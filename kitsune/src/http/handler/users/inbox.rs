use crate::{
    activitypub::{handle_attachments, handle_mentions},
    error::{Error, Result},
    event::{post::EventType, PostEvent},
    http::extractor::SignedActivity,
    job::deliver::accept::DeliverAccept,
    service::{federation_filter::FederationFilterService, job::Enqueue},
    state::Zustand,
};
use axum::{debug_handler, extract::State};
use futures_util::{future::OptionFuture, FutureExt};
use kitsune_type::ap::{Activity, ActivityType};
use std::ops::Not;
use time::OffsetDateTime;
use uuid::Uuid;

async fn accept_activity(state: &Zustand, activity: Activity) -> Result<()> {
    let Some(follow_activity) = AccountsFollowers::find()
        .filter(accounts_followers::Column::Url.eq(activity.object()))
        .one(&state.db_conn)
        .await?
    else {
        return Ok(());
    };

    let mut follow_activity: accounts_followers::ActiveModel = follow_activity.into();
    follow_activity.approved_at = ActiveValue::Set(Some(OffsetDateTime::now_utc()));
    follow_activity.update(&state.db_conn).await?;

    Ok(())
}

async fn create_activity(
    state: &Zustand,
    author: accounts::Model,
    activity: Activity,
) -> Result<()> {
    if let Some(object) = activity.object.into_object() {
        let in_reply_to_id = OptionFuture::from(
            object
                .in_reply_to
                .as_ref()
                .map(|post_url| state.fetcher.fetch_object(post_url)),
        )
        .await
        .transpose()?
        .map(|in_reply_to| in_reply_to.id);

        let new_post = state
            .db_conn
            .transaction(|tx| {
                async move {
                    let visibility = Visibility::from_activitypub(&author, &object).unwrap();
                    let new_post = Posts::insert(
                        posts::Model {
                            id: Uuid::now_v7(),
                            account_id: author.id,
                            in_reply_to_id,
                            reposted_post_id: None,
                            subject: object.summary,
                            content: object.content,
                            is_sensitive: object.sensitive,
                            visibility,
                            is_local: false,
                            url: object.id,
                            created_at: object.published,
                            updated_at: OffsetDateTime::now_utc(),
                        }
                        .into_active_model(),
                    )
                    .exec(tx)
                    .await?;

                    handle_attachments(tx, &author, new_post.last_insert_id, object.attachment)
                        .await?;
                    handle_mentions(tx, &author, new_post.last_insert_id, &object.tag).await?;

                    Ok::<_, Error>(new_post)
                }
                .boxed()
            })
            .await?;

        state
            .event_emitter
            .post
            .emit(PostEvent {
                r#type: EventType::Create,
                post_id: new_post.last_insert_id,
            })
            .await
            .map_err(Error::Event)?;
    }

    Ok(())
}

async fn delete_activity(
    state: &Zustand,
    author: accounts::Model,
    activity: Activity,
) -> Result<()> {
    let Some((post_id,)): Option<(Uuid,)> = Posts::find()
        .filter(posts::Column::AccountId.eq(author.id))
        .filter(posts::Column::Url.eq(activity.object()))
        .select_only()
        .column(posts::Column::Id)
        .into_tuple()
        .one(&state.db_conn)
        .await?
    else {
        return Ok(())
    };

    Posts::delete_by_id(post_id).exec(&state.db_conn).await?;

    state
        .event_emitter
        .post
        .emit(PostEvent {
            r#type: EventType::Delete,
            post_id,
        })
        .await
        .map_err(Error::Event)?;

    Ok(())
}

async fn follow_activity(
    state: &Zustand,
    author: accounts::Model,
    activity: Activity,
) -> Result<()> {
    let followed_user = state.fetcher.fetch_actor(activity.object().into()).await?;
    let approved_at = followed_user.locked.not().then(OffsetDateTime::now_utc);

    let insert_result = AccountsFollowers::insert(
        accounts_followers::Model {
            id: Uuid::now_v7(),
            account_id: followed_user.id,
            follower_id: author.id,
            approved_at,
            url: activity.id,
            created_at: activity.published,
            updated_at: OffsetDateTime::now_utc(),
        }
        .into_active_model(),
    )
    .exec(&state.db_conn)
    .await?;

    if followed_user.local {
        state
            .service
            .job
            .enqueue(
                Enqueue::builder()
                    .job(DeliverAccept {
                        follow_id: insert_result.last_insert_id,
                    })
                    .build(),
            )
            .await?;
    }

    Ok(())
}

async fn like_activity(state: &Zustand, author: accounts::Model, activity: Activity) -> Result<()> {
    let permission_check = PermissionCheck::builder()
        .fetching_account_id(Some(author.id))
        .build()
        .unwrap();

    let Some(post) = Posts::find()
        .filter(posts::Column::Url.eq(activity.object()))
        .add_permission_checks(permission_check)
        .one(&state.db_conn)
        .await?
    else {
        return Ok(());
    };

    PostsFavourites::insert(
        posts_favourites::Model {
            id: Uuid::now_v7(),
            account_id: author.id,
            post_id: post.id,
            url: activity.id,
            created_at: OffsetDateTime::now_utc(),
        }
        .into_active_model(),
    )
    .exec_without_returning(&state.db_conn)
    .await?;

    Ok(())
}

async fn reject_activity(
    state: &Zustand,
    author: accounts::Model,
    activity: Activity,
) -> Result<()> {
    AccountsFollowers::delete_many()
        .filter(accounts_followers::Column::AccountId.eq(author.id))
        .filter(accounts_followers::Column::Url.eq(activity.object()))
        .exec(&state.db_conn)
        .await?;

    Ok(())
}

async fn undo_activity(state: &Zustand, author: accounts::Model, activity: Activity) -> Result<()> {
    // An undo activity can apply for likes and follows
    PostsFavourites::delete_many()
        .filter(posts_favourites::Column::AccountId.eq(author.id))
        .filter(posts_favourites::Column::Url.eq(activity.object()))
        .exec(&state.db_conn)
        .await?;

    AccountsFollowers::delete_many()
        .filter(accounts_followers::Column::FollowerId.eq(author.id))
        .filter(accounts_followers::Column::Url.eq(activity.object()))
        .exec(&state.db_conn)
        .await?;

    Ok(())
}

/// It's fine that the extractor doesn't check for "activity author == object author" since the logic
/// of this inbox implementation attributes the contents of the object to the activity author
///
/// Since the extractor validates "request signer == activity author", it is safe to assume that the object author is the activity author.
/// There aren't really any scenarios where this could be used for any nefarious purposes since a user would have *much* bigger problems than
/// getting someone elses post attributed to them.
#[debug_handler(state = Zustand)]
pub async fn post(
    State(state): State<Zustand>,
    State(federation_filter): State<FederationFilterService>,
    SignedActivity(author, activity): SignedActivity,
) -> Result<()> {
    increment_counter!("received_activities");

    if !federation_filter.is_entity_allowed(&activity)? {
        return Ok(());
    }

    match activity.r#type {
        ActivityType::Accept => accept_activity(&state, activity).await,
        ActivityType::Announce => todo!(),
        ActivityType::Block => todo!(),
        ActivityType::Create => create_activity(&state, author, activity).await,
        ActivityType::Delete => delete_activity(&state, author, activity).await,
        ActivityType::Follow => follow_activity(&state, author, activity).await,
        ActivityType::Like => like_activity(&state, author, activity).await,
        ActivityType::Reject => reject_activity(&state, author, activity).await,
        ActivityType::Undo => undo_activity(&state, author, activity).await,
        ActivityType::Update => todo!(),
    }
}
