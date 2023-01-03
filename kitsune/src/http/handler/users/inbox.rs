use crate::{
    db::model::{
        account, follow,
        post::{self, Visibility},
    },
    error::Result,
    http::extractor::SignedActivity,
    state::Zustand,
};
use axum::{debug_handler, extract::State};
use chrono::Utc;
use kitsune_type::ap::{Activity, ActivityType, Object};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use uuid::Uuid;

async fn create_activity(state: &Zustand, activity: Activity) -> Result<()> {
    let account = account::Entity::find()
        .filter(account::Column::Url.eq(activity.rest.attributed_to().unwrap()))
        .one(&state.db_conn)
        .await?
        .unwrap();
    let visibility = Visibility::from_activitypub(&account, &activity);

    match activity.object.into_object() {
        Some(Object::Note(note)) => {
            post::Model {
                id: Uuid::now_v7(),
                account_id: account.id,
                subject: note.subject,
                content: note.content,
                is_sensitive: note.rest.sensitive,
                visibility,
                url: note.rest.id,
                created_at: note.rest.published,
                updated_at: Utc::now(),
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
    let account = account::Entity::find()
        .filter(account::Column::Url.eq(activity.rest.attributed_to().unwrap()))
        .one(&state.db_conn)
        .await?
        .unwrap();

    if let Some(url) = activity.object.into_string() {
        post::Entity::delete(post::ActiveModel {
            account_id: ActiveValue::Set(account.id),
            url: ActiveValue::Set(url),
            ..Default::default()
        })
        .exec(&state.db_conn)
        .await?;
    }

    Ok(())
}

async fn follow_activity(state: &Zustand, activity: Activity) -> Result<()> {
    let account = account::Entity::find()
        .filter(account::Column::Url.eq(activity.rest.attributed_to().unwrap()))
        .one(&state.db_conn)
        .await?
        .unwrap();

    if let Some(url) = activity.object.into_string() {
        let followed_user = state.fetcher.fetch_actor(&url).await?;

        follow::Model {
            id: Uuid::now_v7(),
            account_id: followed_user.id,
            follower_id: account.id,
            approved_at: None,
            url: activity.rest.id,
            created_at: activity.rest.published,
            updated_at: Utc::now(),
        }
        .into_active_model()
        .insert(&state.db_conn)
        .await?;
    }

    Ok(())
}

#[debug_handler]
pub async fn post(
    State(state): State<Zustand>,
    SignedActivity(activity): SignedActivity,
) -> Result<()> {
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
