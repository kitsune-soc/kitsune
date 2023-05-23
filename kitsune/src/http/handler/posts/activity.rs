use crate::{
    error::Result, http::responder::ActivityPubJson, mapping::IntoActivity, state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

#[debug_handler]
pub async fn get(State(state): State<Zustand>, Path(id): Path<Uuid>) -> Result<Response> {
    let post = state.service.post.get_by_id(id, None).await?;
    Ok(ActivityPubJson(post.into_activity(&state).await?).into_response())
}
