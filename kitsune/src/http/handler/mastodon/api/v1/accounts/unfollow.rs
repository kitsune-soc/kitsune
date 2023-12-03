use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
};
use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use kitsune_core::error::HttpError;
use kitsune_mastodon::MastodonMapper;
use kitsune_service::account::{AccountService, Unfollow};
use kitsune_type::mastodon::relationship::Relationship;
use speedy_uuid::Uuid;

#[debug_handler(state = crate::state::Zustand)]
pub async fn post(
    State(account_service): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<Json<Relationship>> {
    if user_data.account.id == id {
        return Err(HttpError::BadRequest.into());
    }

    let unfollow = Unfollow::builder()
        .account_id(id)
        .follower_id(user_data.account.id)
        .build();
    let unfollow_accounts = account_service.unfollow(unfollow).await?;

    Ok(Json(
        mastodon_mapper
            .map((&unfollow_accounts.0, &unfollow_accounts.1))
            .await?,
    ))
}
