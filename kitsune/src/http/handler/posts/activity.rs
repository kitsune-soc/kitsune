use crate::{error::Result, http::responder::ActivityPubJson};
use axum::{
    debug_handler,
    extract::{Path, State},
};
use kitsune_core::{mapping::IntoActivity, state::Zustand};
use kitsune_type::ap::Activity;
use speedy_uuid::Uuid;

#[debug_handler]
pub async fn get(
    State(state): State<Zustand>,
    Path(id): Path<Uuid>,
) -> Result<ActivityPubJson<Activity>> {
    let post = state.service.post.get_by_id(id, None).await?;
    Ok(ActivityPubJson(post.into_activity(&state).await?))
}
