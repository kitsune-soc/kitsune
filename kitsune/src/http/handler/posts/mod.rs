use crate::{
    error::{ApiError, Result},
    mapping::IntoObject,
    state::Zustand,
};
use axum::{debug_handler, extract::Path, extract::State, response::Html, routing, Json, Router};
use kitsune_type::ap::Object;
use uuid::Uuid;

mod activity;

#[debug_handler]
async fn get(State(state): State<Zustand>, Path(id): Path<Uuid>) -> Result<Json<Object>> {
    let post = state
        .service
        .post
        .get_by_id(id, None)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(post.into_object(&state).await?))
}

#[debug_handler]
async fn get_html(State(state): State<Zustand>, Path(id): Path<Uuid>) -> Result<Html<String>> {
    let post = state
        .service
        .post
        .get_by_id(id, None)
        .await?
        .ok_or(ApiError::NotFound)?;

    todo!();
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:id", routing::get(get))
        .route("/:id/activity", routing::get(activity::get))
}
