use crate::{
    activitypub::{process_new_object, ProcessNewObject},
    error::{Error, Result},
    event::{post::EventType, PostEvent},
    http::extractor::SignedActivity,
    job::deliver::accept::DeliverAccept,
    service::{federation_filter::FederationFilterService, job::Enqueue},
    state::Zustand,
};
use axum::{debug_handler, extract::State};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use kitsune_db::{
    model::{
        account::Account,
        favourite::NewFavourite,
        follower::NewFollow,
        post::{NewPost, Post},
    },
    post_permission_check::{PermissionCheck, PostPermissionCheckExt},
    schema::{accounts_follows, posts, posts_favourites},
};
use kitsune_type::ap::{Activity, ActivityType};
use std::ops::Not;
use time::OffsetDateTime;
use uuid::Uuid;

async fn accept_activity(state: &Zustand, activity: Activity) -> Result<()> {
    let mut db_conn = state.db_conn.get().await?;
    diesel::update(accounts_follows::table.filter(accounts_follows::url.eq(activity.object())))
        .set(accounts_follows::approved_at.eq(OffsetDateTime::now_utc()))
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

async fn announce_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    let reposted_post = state.fetcher.fetch_object(activity.object()).await?;

    let mut db_conn = state.db_conn.get().await?;
    diesel::insert_into(posts::table)
        .values(NewPost {
            id: Uuid::now_v7(),
            account_id: author.id,
            in_reply_to_id: None,
            reposted_post_id: Some(reposted_post.id),
            is_sensitive: false,
            subject: None,
            content: "",
            link_preview_url: None,
            visibility: reposted_post.visibility,
            is_local: false,
            url: activity.id.as_str(),
            created_at: None,
        })
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

async fn create_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    if let Some(object) = activity.object.into_object() {
        let mut db_conn = state.db_conn.get().await?;
        let process_data = ProcessNewObject::builder()
            .author(author)
            .db_conn(&mut db_conn)
            .embed_client(state.embed_client.as_ref())
            .fetcher(&state.fetcher)
            .object(object)
            .search_service(&state.service.search)
            .build();
        let new_post = process_new_object(process_data).await?;

        state
            .event_emitter
            .post
            .emit(PostEvent {
                r#type: EventType::Create,
                post_id: new_post.id,
            })
            .await
            .map_err(Error::Event)?;
    }

    Ok(())
}

async fn delete_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    let mut db_conn = state.db_conn.get().await?;
    let post_id = posts::table
        .filter(posts::account_id.eq(author.id))
        .filter(posts::url.eq(activity.object()))
        .select(posts::id)
        .get_result(&mut db_conn)
        .await?;

    diesel::delete(posts::table.find(post_id))
        .execute(&mut db_conn)
        .await?;

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

async fn follow_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    let followed_user = state.fetcher.fetch_actor(activity.object().into()).await?;
    let approved_at = followed_user.locked.not().then(OffsetDateTime::now_utc);

    let mut db_conn = state.db_conn.get().await?;
    let follow_id = diesel::insert_into(accounts_follows::table)
        .values(NewFollow {
            id: Uuid::now_v7(),
            account_id: followed_user.id,
            follower_id: author.id,
            approved_at,
            url: activity.id.as_str(),
            created_at: Some(activity.published),
        })
        .returning(accounts_follows::id)
        .get_result(&mut db_conn)
        .await?;

    if followed_user.local {
        state
            .service
            .job
            .enqueue(Enqueue::builder().job(DeliverAccept { follow_id }).build())
            .await?;
    }

    Ok(())
}

async fn like_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    let mut db_conn = state.db_conn.get().await?;
    let permission_check = PermissionCheck::builder()
        .fetching_account_id(Some(author.id))
        .build()
        .unwrap();

    let post = posts::table
        .filter(posts::url.eq(activity.object()))
        .add_post_permission_check(permission_check)
        .select(Post::as_select())
        .get_result::<Post>(&mut db_conn)
        .await?;

    diesel::insert_into(posts_favourites::table)
        .values(NewFavourite {
            id: Uuid::now_v7(),
            account_id: author.id,
            post_id: post.id,
            url: activity.id,
            created_at: Some(OffsetDateTime::now_utc()),
        })
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

async fn reject_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    let mut db_conn = state.db_conn.get().await?;
    diesel::delete(
        accounts_follows::table.filter(
            accounts_follows::account_id
                .eq(author.id)
                .and(accounts_follows::url.eq(activity.object())),
        ),
    )
    .execute(&mut db_conn)
    .await?;

    Ok(())
}

async fn undo_activity(state: &Zustand, author: Account, activity: Activity) -> Result<()> {
    let mut db_conn = state.db_conn.get().await?;
    // An undo activity can apply for likes and follows and announces
    diesel::delete(
        posts_favourites::table.filter(
            posts_favourites::account_id
                .eq(author.id)
                .and(posts_favourites::url.eq(activity.object())),
        ),
    )
    .execute(&mut db_conn)
    .await?;

    diesel::delete(
        accounts_follows::table.filter(
            accounts_follows::follower_id
                .eq(author.id)
                .and(accounts_follows::url.eq(activity.object())),
        ),
    )
    .execute(&mut db_conn)
    .await?;

    diesel::delete(
        posts::table.filter(
            posts::url
                .eq(activity.object())
                .and(posts::account_id.eq(author.id)),
        ),
    )
    .execute(&mut db_conn)
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
    #[cfg(feature = "metrics")]
    increment_counter!("received_activities");

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
        ActivityType::Update => todo!(),
    }
}
