use crate::{
    db::entity::{post, user},
    error::Result,
    http::extractor::SignedActivity,
    state::State,
};
use axum::{debug_handler, Extension};
use chrono::Utc;
use phenomenon_model::ap::{Activity, ActivityType, Object};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use uuid::Uuid;

async fn create_activity(state: &State, activity: Activity) -> Result<()> {
    let user = user::Entity::find()
        .filter(user::Column::Url.eq(activity.rest.attributed_to().unwrap()))
        .one(&state.db_conn)
        .await?
        .unwrap();

    match activity.object.into_object() {
        Some(Object::Note(note)) => {
            post::Model {
                id: Uuid::new_v4(),
                user_id: user.id,
                subject: note.subject,
                content: note.content,
                url: note.rest.id,
                created_at: note.rest.published_at,
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

async fn delete_activity(state: &State, activity: Activity) -> Result<()> {
    let user = user::Entity::find()
        .filter(user::Column::Url.eq(activity.rest.attributed_to().unwrap()))
        .one(&state.db_conn)
        .await?
        .unwrap();

    if let Some(url) = activity.object.into_string() {
        post::Entity::delete(post::ActiveModel {
            user_id: ActiveValue::Set(user.id),
            url: ActiveValue::Set(url),
            ..Default::default()
        })
        .exec(&state.db_conn)
        .await?;
    }

    Ok(())
}

#[debug_handler]
pub async fn post(
    Extension(state): Extension<State>,
    SignedActivity(activity): SignedActivity,
) -> Result<()> {
    // TODO: Insert activity into database

    match activity.r#type {
        ActivityType::Announce => todo!(),
        ActivityType::Create => create_activity(&state, activity).await,
        ActivityType::Delete => delete_activity(&state, activity).await,
        ActivityType::Follow => todo!(),
        ActivityType::Like => todo!(),
        ActivityType::Undo => todo!(),
        ActivityType::Update => todo!(),
    }
}
