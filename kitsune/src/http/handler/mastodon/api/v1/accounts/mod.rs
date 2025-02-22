use axum::{
    Json,
    extract::{Path, State},
};
use kitsune_error::{ErrorType, Result, kitsune_error};
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

pub async fn get(
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
