use crate::{error::Result, mapping::IntoActivity, state::Zustand};
use axum::{
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use uuid::Uuid;

#[debug_handler]
pub async fn get(State(state): State<Zustand>, Path(id): Path<Uuid>) -> Result<Response> {
    let Some(post) = state
        .service
        .post
        .get_by_id(id, None)
        .await?
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(post.into_activity(&state).await?).into_response())
}
