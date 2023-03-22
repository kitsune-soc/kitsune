use crate::{error::Result, mapping::MastodonMapper, state::Zustand};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use http::StatusCode;
use kitsune_db::entity::prelude::Accounts;
use sea_orm::{DatabaseConnection, EntityTrait};
use uuid::Uuid;

mod relationships;
mod statuses;
mod verify_credentials;

async fn get(
    State(db_conn): State<DatabaseConnection>,
    State(mastodon_mapper): State<MastodonMapper>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let Some(account) = Accounts::find_by_id(id).one(&db_conn).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Json(mastodon_mapper.map(account).await?).into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:id", routing::get(get))
        .route("/:id/statuses", routing::get(statuses::get))
        .route("/relationships", routing::get(relationships::get))
        .route("/verify_credentials", routing::get(verify_credentials::get))
}
