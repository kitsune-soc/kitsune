use crate::{
    db::model::post::{self, Visibility},
    error::Result,
    mapping::IntoObject,
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::Path,
    extract::State,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use http::StatusCode;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

mod activity;

#[debug_handler]
async fn get(State(state): State<Zustand>, Path(id): Path<Uuid>) -> Result<Response> {
    let Some(post) = post::Entity::find_by_id(id)
        .filter(post::Column::Visibility.is_in([Visibility::Public, Visibility::Unlisted]))
        .one(&state.db_conn)
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
