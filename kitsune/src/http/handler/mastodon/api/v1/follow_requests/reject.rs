use crate::http::extractor::{AuthExtractor, MastodonAuthExtractor};
use axum::{
    Json, debug_handler,
    extract::{Path, State},
};
use kitsune_error::{ErrorType, Result, bail};
use kitsune_mastodon::MastodonMapper;
use kitsune_service::account::{AccountService, FollowRequest};
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
        bail!(type = ErrorType::BadRequest, "user tried to reject a follow to themselves");
    }

    let follow_request = FollowRequest::builder()
        .account_id(user_data.account.id)
        .follower_id(id)
        .build();

    let follow_accounts = account_service
        .reject_follow_request(follow_request)
        .await?;

    if let Some(follow_accounts) = follow_accounts {
        Ok(Json(
            mastodon_mapper
                .map((&follow_accounts.0, &follow_accounts.1))
                .await?,
        ))
    } else {
        bail!(type = ErrorType::BadRequest, "follow request wasn't found in the database");
    }
}
