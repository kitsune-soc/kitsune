use crate::{db::model::user, error::Result, mapping::IntoMastodon, state::Zustand};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use http::StatusCode;
use sea_orm::EntityTrait;
use uuid::Uuid;

mod verify_credentials;

async fn get_handler(State(state): State<Zustand>, Path(id): Path<Uuid>) -> Result<Response> {
    let Some(user) = user::Entity::find_by_id(id).one(&state.db_conn).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(user.into_mastodon(&state).await?).into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:id", get(get_handler))
        .route("/verify_credentials", get(verify_credentials::get))
}
