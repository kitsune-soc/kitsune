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

pub mod follow;
pub mod relationships;
pub mod statuses;
pub mod unfollow;
pub mod verify_credentials;

#[utoipa::path(
    get,
    path = "/api/v1/accounts/{id}",
    responses(
        (status = 200, description = "Account information", body = Account),
        (status = StatusCode::NOT_FOUND, description = "No account with that ID exists"),
    )
)]
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
        .route("/:id/follow", routing::post(follow::post))
        .route("/:id/statuses", routing::get(statuses::get))
        .route("/:id/unfollow", routing::post(unfollow::post))
        .route("/relationships", routing::get(relationships::get))
        .route("/verify_credentials", routing::get(verify_credentials::get))
}
