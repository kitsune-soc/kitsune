use crate::{http::extractor::SignedActivity, state::State};
use axum::{debug_handler, Extension};
use phenomenon_model::ap::ActivityType;

#[debug_handler]
pub async fn post(Extension(_state): Extension<State>, SignedActivity(activity): SignedActivity) {
    // TODO: Insert activity into database

    match activity.r#type {
        ActivityType::Announce => todo!(),
        ActivityType::Create => todo!(),
        ActivityType::Delete => todo!(),
        ActivityType::Follow => todo!(),
        ActivityType::Like => todo!(),
        ActivityType::Undo => todo!(),
        ActivityType::Update => todo!(),
    }
}
