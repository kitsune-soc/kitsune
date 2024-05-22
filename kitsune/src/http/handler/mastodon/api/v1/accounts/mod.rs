use crate::state::Zustand;
use axum::{
    extract::{Path, State},
    routing, Json, Router,
};
use kitsune_error::{kitsune_error, ErrorType, Result};
use kitsune_mastodon::MastodonMapper;
use kitsune_service::account::AccountService;
use kitsune_type::mastodon;
use speedy_uuid::Uuid;

pub mod follow;
pub mod lookup;
pub mod relationships;
pub mod statuses;
pub mod unfollow;
pub mod update_credentials;
pub mod verify_credentials;

async fn get(
    State(account_service): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    Path(id): Path<Uuid>,
) -> Result<Json<mastodon::Account>> {
    let account = account_service
        .get_by_id(id)
        .await?
        .ok_or_else(|| kitsune_error!(type = ErrorType::NotFound, "account not found"))?;

    Ok(Json(mastodon_mapper.map(account).await?))
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/:id", routing::get(get))
        .route("/:id/follow", routing::post(follow::post))
        .route("/:id/statuses", routing::get(statuses::get))
        .route("/:id/unfollow", routing::post(unfollow::post))
        .route("/lookup", routing::get(lookup::get))
        .route("/relationships", routing::get(relationships::get))
        .route(
            "/update_credentials",
            routing::patch(update_credentials::patch),
        )
        .route("/verify_credentials", routing::get(verify_credentials::get))
}
