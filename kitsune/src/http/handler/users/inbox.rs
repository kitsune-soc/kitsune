use crate::{error::Result, http::extractor::SignedActivity, state::Zustand};
use axum::{debug_handler, extract::State};
use chrono::Utc;
use kitsune_db::{
    custom::Visibility,
    entity::{
        accounts, accounts_followers, posts,
        prelude::{Accounts, Posts},
    },
};
use kitsune_type::ap::{Activity, ActivityType, Object};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use uuid::Uuid;

async fn create_activity(state: &Zustand, activity: Activity) -> Result<()> {
    let account = Accounts::find()
        .filter(accounts::Column::Url.eq(activity.rest.attributed_to().unwrap()))
        .one(&state.db_conn)
        .await?
        .unwrap();
    let visibility = Visibility::from_activitypub(&account, &activity);

    match activity.object.into_object() {
        Some(Object::Note(note)) => {
            let in_reply_to_id = if let Some(in_reply_to) = note.rest.in_reply_to {
                let note = state.fetcher.fetch_note(&in_reply_to).await?;
                Some(note.id)
            } else {
                None
            };

            posts::Model {
                id: Uuid::now_v7(),
                account_id: account.id,
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
            // TODO: Handle rest of the cases
        }
    }

    Ok(())
}

async fn delete_activity(state: &Zustand, activity: Activity) -> Result<()> {
    let account = Accounts::find()
        .filter(accounts::Column::Url.eq(activity.rest.attributed_to().unwrap()))
        .one(&state.db_conn)
        .await?
        .unwrap();

    Posts::delete(posts::ActiveModel {
        account_id: ActiveValue::Set(account.id),
        url: ActiveValue::Set(activity.object().to_string()),
        ..Default::default()
    })
    .exec(&state.db_conn)
    .await?;

    Ok(())
}

async fn follow_activity(state: &Zustand, activity: Activity) -> Result<()> {
    let account = Accounts::find()
        .filter(accounts::Column::Url.eq(activity.rest.attributed_to().unwrap()))
        .one(&state.db_conn)
        .await?
        .unwrap();

    let followed_user = state.fetcher.fetch_actor(activity.object()).await?;

    accounts_followers::Model {
        id: Uuid::now_v7(),
        account_id: followed_user.id,
        follower_id: account.id,
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
    SignedActivity(activity): SignedActivity,
) -> Result<()> {
    increment_counter!("received_activities");
    // TODO: Insert activity into database

    match activity.r#type {
        ActivityType::Accept => todo!(),
        ActivityType::Announce => todo!(),
        ActivityType::Block => todo!(),
        ActivityType::Create => create_activity(&state, activity).await,
        ActivityType::Delete => delete_activity(&state, activity).await,
        ActivityType::Follow => follow_activity(&state, activity).await,
        ActivityType::Like => todo!(),
        ActivityType::Reject => todo!(),
        ActivityType::Undo => todo!(),
        ActivityType::Update => todo!(),
    }
}
