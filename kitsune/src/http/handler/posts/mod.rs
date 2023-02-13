use crate::{error::Result, mapping::IntoObject, state::Zustand};
use axum::{
    debug_handler,
    extract::Path,
    extract::State,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use http::StatusCode;
use uuid::Uuid;

mod activity;

#[debug_handler]
async fn get(State(state): State<Zustand>, Path(id): Path<Uuid>) -> Result<Response> {
    let Some(post) = state
        .service
        .post
        .get_by_id(id, None)
        .await?
    else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(post.into_object(&state).await?).into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:id", routing::get(get))
        .route("/:id/activity", routing::get(activity::get))
}
