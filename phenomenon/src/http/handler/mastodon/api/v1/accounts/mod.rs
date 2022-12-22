use crate::{db::model::account, error::Result, mapping::IntoMastodon, state::Zustand};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use http::StatusCode;
use sea_orm::EntityTrait;
use uuid::Uuid;

mod verify_credentials;

async fn get(State(state): State<Zustand>, Path(id): Path<Uuid>) -> Result<Response> {
    let Some(account) = account::Entity::find_by_id(id).one(&state.db_conn).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(account.into_mastodon(&state).await?).into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:id", routing::get(get))
        .route("/verify_credentials", routing::get(verify_credentials::get))
}
