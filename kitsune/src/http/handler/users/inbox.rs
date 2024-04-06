use crate::{http::extractor::SignedActivity, state::Zustand};
use axum::{debug_handler, extract::State};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use iso8601_timestamp::Timestamp;
use kitsune_activitypub::{process_new_object, update_object, ProcessNewObject};
use kitsune_core::error::HttpError;
use kitsune_db::{
    model::{
        account::Account,
        favourite::NewFavourite,
        follower::NewFollow,
        notification::NewNotification,
        post::{NewPost, Post},
        preference::Preferences,
    },
    post_permission_check::{PermissionCheck, PostPermissionCheckExt},
    schema::{accounts_follows, accounts_preferences, notifications, posts, posts_favourites},
    with_connection,
};
use kitsune_error::{Error, Result};
use kitsune_federation_filter::FederationFilter;
use kitsune_jobs::deliver::accept::DeliverAccept;
use kitsune_service::job::Enqueue;
use kitsune_type::ap::{Activity, ActivityType};
use kitsune_util::try_join;
use speedy_uuid::Uuid;
use std::ops::Not;

async fn accept_activity(state: &Zustand, activity: Activity) -> Result<()> {
    with_connection!(state.db_pool, |db_conn| {
        diesel::update(accounts_follows::table.filter(accounts_follows::url.eq(activity.object())))
            .set(accounts_follows::approved_at.eq(Timestamp::now_utc()))
            .execute(db_conn)
            .await
    })?;

    Ok(())
}

async fn announce_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    let Some(reposted_post) = state.fetcher.fetch_post(activity.object().into()).await? else {
        return Err(HttpError::BadRequest.into());
    };

    with_connection!(state.db_pool, |db_conn| {
        diesel::insert_into(posts::table)
            .values(NewPost {
                id: Uuid::now_v7(),
                account_id: author.id,
                in_reply_to_id: None,
                reposted_post_id: Some(reposted_post.id),
                is_sensitive: false,
                subject: None,
                content: "",
                content_source: "",
                content_lang: kitsune_language::Language::Eng.into(),
                link_preview_url: None,
                visibility: reposted_post.visibility,
                is_local: false,
                url: activity.id.as_str(),
                created_at: None,
            })
            .execute(db_conn)
            .await
    })?;

    Ok(())
}

async fn create_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    if let Some(object) = activity.object.into_object() {
        let process_data = ProcessNewObject::builder()
            .author(&author)
            .db_pool(&state.db_pool)
            .embed_client(state.embed_client.as_ref())
            .fetcher(&state.fetcher)
            .language_detection_config(state.language_detection_config)
            .object(Box::new(object))
            .search_backend(state.service.search.backend())
            .build();
        process_new_object(process_data).await?;
    }

    Ok(())
}

async fn delete_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    with_connection!(state.db_pool, |db_conn| {
        diesel::delete(
            posts::table
                .filter(posts::account_id.eq(author.id))
                .filter(posts::url.eq(activity.object())),
        )
        .execute(db_conn)
        .await
    })?;

    Ok(())
}

async fn follow_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    let Some(followed_user) = state
        .fetcher
        .fetch_account(activity.object().into())
        .await?
    else {
        return Err(HttpError::BadRequest.into());
    };

    let approved_at = followed_user.locked.not().then(Timestamp::now_utc);

    let follow_id = with_connection!(state.db_pool, |db_conn| {
        diesel::insert_into(accounts_follows::table)
            .values(NewFollow {
                id: Uuid::now_v7(),
                account_id: followed_user.id,
                follower_id: author.id,
                approved_at,
                url: activity.id.as_str(),
                notify: false,
                created_at: Some(activity.published),
            })
            .returning(accounts_follows::id)
            .get_result(db_conn)
            .await
    })?;

    if followed_user.local {
        let preferences = with_connection!(state.db_pool, |db_conn| {
            accounts_preferences::table
                .find(followed_user.id)
                .select(Preferences::as_select())
                .get_result(db_conn)
                .await
        })?;

        if (preferences.notify_on_follow && !followed_user.locked)
            || (preferences.notify_on_follow_request && followed_user.locked)
        {
            let notification = if followed_user.locked {
                NewNotification::builder()
                    .receiving_account_id(followed_user.id)
                    .follow_request(author.id)
            } else {
                NewNotification::builder()
                    .receiving_account_id(followed_user.id)
                    .follow(author.id)
            };

            with_connection!(state.db_pool, |db_conn| {
                diesel::insert_into(notifications::table)
                    .values(notification)
                    .on_conflict_do_nothing()
                    .execute(db_conn)
                    .await
            })?;
        }
        state
            .service
            .job
            .enqueue(Enqueue::builder().job(DeliverAccept { follow_id }).build())
            .await?;
    }

    Ok(())
}

async fn like_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    let permission_check = PermissionCheck::builder()
        .fetching_account_id(Some(author.id))
        .build();

    with_connection!(state.db_pool, |db_conn| {
        let post = posts::table
            .filter(posts::url.eq(activity.object()))
            .add_post_permission_check(permission_check)
            .select(Post::as_select())
            .get_result::<Post>(db_conn)
            .await?;

        diesel::insert_into(posts_favourites::table)
            .values(NewFavourite {
                id: Uuid::now_v7(),
                account_id: author.id,
                post_id: post.id,
                url: activity.id,
                created_at: Some(Timestamp::now_utc()),
            })
            .execute(db_conn)
            .await?;

        Ok::<_, Error>(())
    })?;

    Ok(())
}

async fn reject_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    with_connection!(state.db_pool, |db_conn| {
        diesel::delete(
            accounts_follows::table.filter(
                accounts_follows::account_id
                    .eq(author.id)
                    .and(accounts_follows::url.eq(activity.object())),
            ),
        )
        .execute(db_conn)
        .await
    })?;

    Ok(())
}

async fn undo_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    with_connection!(state.db_pool, |db_conn| {
        // An undo activity can apply for likes and follows and announces
        let favourite_delete_fut = diesel::delete(
            posts_favourites::table.filter(
                posts_favourites::account_id
                    .eq(author.id)
                    .and(posts_favourites::url.eq(activity.object())),
            ),
        )
        .execute(db_conn);

        let follow_delete_fut = diesel::delete(
            accounts_follows::table.filter(
                accounts_follows::follower_id
                    .eq(author.id)
                    .and(accounts_follows::url.eq(activity.object())),
            ),
        )
        .execute(db_conn);

        let repost_delete_fut = diesel::delete(
            posts::table.filter(
                posts::url
                    .eq(activity.object())
                    .and(posts::account_id.eq(author.id)),
            ),
        )
        .execute(db_conn);

        try_join!(favourite_delete_fut, follow_delete_fut, repost_delete_fut)
    })?;

    Ok(())
}

async fn update_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    if let Some(object) = activity.object.into_object() {
        let process_data = ProcessNewObject::builder()
            .author(&author)
            .db_pool(&state.db_pool)
            .embed_client(state.embed_client.as_ref())
            .fetcher(&state.fetcher)
            .language_detection_config(state.language_detection_config)
            .object(Box::new(object))
            .search_backend(state.service.search.backend())
            .build();

        update_object(process_data).await?;
    }

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
    State(federation_filter): State<FederationFilter>,
    SignedActivity(author, activity): SignedActivity,
) -> Result<()> {
    let counter = counter!("received_activities");
    counter.increment(1);

    if !federation_filter.is_entity_allowed(&activity)? {
        return Ok(());
    }

    match activity.r#type {
        ActivityType::Accept => accept_activity(&state, activity).await,
        ActivityType::Announce => announce_activity(&state, author, activity).await,
        ActivityType::Block => todo!(),
        ActivityType::Create => create_activity(&state, author, activity).await,
        ActivityType::Delete => delete_activity(&state, author, activity).await,
        ActivityType::Follow => follow_activity(&state, author, activity).await,
        ActivityType::Like => like_activity(&state, author, activity).await,
        ActivityType::Reject => reject_activity(&state, author, activity).await,
        ActivityType::Undo => undo_activity(&state, author, activity).await,
        ActivityType::Update => update_activity(&state, author, activity).await,
    }
}
