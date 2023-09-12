use crate::{error::Result, http::responder::ActivityPubJson, state::AppState};
use axum::{
    debug_handler,
    extract::{Path, State},
};
use kitsune_core::mapping::IntoActivity;
use kitsune_type::ap::Activity;
use speedy_uuid::Uuid;

#[debug_handler]
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<ActivityPubJson<Activity>> {
    let post = state.service().post.get_by_id(id, None).await?;
    Ok(ActivityPubJson(post.into_activity(&state.core).await?))
}
