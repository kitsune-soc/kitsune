use crate::{error::Result, http::extractor::SignedActivity, state::Zustand};
use axum::{debug_handler, extract::State};
use chrono::Utc;
use futures_util::future::OptionFuture;
use kitsune_db::{
    custom::Visibility,
    entity::{accounts, accounts_followers, posts, prelude::Posts},
};
use kitsune_type::ap::{Activity, ActivityType, Object};
use sea_orm::{ActiveModelTrait, ActiveValue, EntityTrait, IntoActiveModel};
use uuid::Uuid;

async fn create_activity(
    state: &Zustand,
    author: accounts::Model,
    activity: Activity,
) -> Result<()> {
    let visibility = Visibility::from_activitypub(&author, &activity);

    match activity.object.into_object() {
        Some(Object::Note(note)) => {
            let in_reply_to_id = OptionFuture::from(
                note.rest
                    .in_reply_to
                    .as_ref()
                    .map(|post_url| state.fetcher.fetch_note(post_url)),
            )
            .await
            .transpose()?
            .map(|in_reply_to| in_reply_to.id);

            posts::Model {
                id: Uuid::now_v7(),
                account_id: author.id,
                in_reply_to_id,
                subject: note.subject,
                content: note.content,
                is_sensitive: note.rest.sensitive,
                visibility,
                is_local: false,
                url: note.rest.id,
                created_at: note.rest.published.into(),
                updated_at: Utc::now().into(),
            }
            .into_active_model()
            .insert(&state.db_conn)
            .await?;
        }
        None | Some(Object::Person(..)) => {
            // Right now, we refuse to save anything but a note
            // If we receive a user or just a URL to a resource, we don't care
        }
    }

    Ok(())
}

async fn delete_activity(
    state: &Zustand,
    author: accounts::Model,
    activity: Activity,
) -> Result<()> {
    Posts::delete(posts::ActiveModel {
        account_id: ActiveValue::Set(author.id),
        url: ActiveValue::Set(activity.object().to_string()),
        ..Default::default()
    })
    .exec(&state.db_conn)
    .await?;

    Ok(())
}

async fn follow_activity(
    state: &Zustand,
    author: accounts::Model,
    activity: Activity,
) -> Result<()> {
    let followed_user = state.fetcher.fetch_actor(activity.object().into()).await?;

    accounts_followers::Model {
        id: Uuid::now_v7(),
        account_id: followed_user.id,
        follower_id: author.id,
        approved_at: None,
        url: activity.rest.id,
        created_at: activity.rest.published.into(),
        updated_at: Utc::now().into(),
    }
    .into_active_model()
    .insert(&state.db_conn)
    .await?;

    Ok(())
}

#[debug_handler]
pub async fn post(
    State(state): State<Zustand>,
    SignedActivity(author, activity): SignedActivity,
) -> Result<()> {
    increment_counter!("received_activities");

    match activity.r#type {
        ActivityType::Accept => todo!(),
        ActivityType::Announce => todo!(),
        ActivityType::Block => todo!(),
        ActivityType::Create => create_activity(&state, author, activity).await,
        ActivityType::Delete => delete_activity(&state, author, activity).await,
        ActivityType::Follow => follow_activity(&state, author, activity).await,
        ActivityType::Like => todo!(),
        ActivityType::Reject => todo!(),
        ActivityType::Undo => todo!(),
        ActivityType::Update => todo!(),
    }
}
