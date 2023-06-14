use crate::{
    error::Result, http::responder::ActivityPubJson, mapping::IntoActivity, state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
};
use kitsune_type::ap::Activity;
use uuid::Uuid;

#[debug_handler]
pub async fn get(
    State(state): State<Zustand>,
    Path(id): Path<Uuid>,
) -> Result<ActivityPubJson<Activity>> {
    let post = state.service.post.get_by_id(id, None).await?;
    Ok(ActivityPubJson(post.into_activity(&state).await?))
}
