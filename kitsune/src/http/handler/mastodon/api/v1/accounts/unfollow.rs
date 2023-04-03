use crate::{
    error::{ApiError, Result},
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::MastodonMapper,
    service::account::{AccountService, Unfollow},
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use kitsune_type::mastodon::relationship::Relationship;
use uuid::Uuid;

#[debug_handler(state = Zustand)]
pub async fn post(
    State(account_service): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<Json<Relationship>> {
    if user_data.account.id == id {
        return Err(ApiError::BadRequest.into());
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
