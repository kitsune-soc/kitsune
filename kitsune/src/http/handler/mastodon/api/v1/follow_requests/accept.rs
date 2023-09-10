use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
};
use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use kitsune_core::{
    error::ApiError,
    mapping::MastodonMapper,
    service::account::{AccountService, FollowRequest},
};
use kitsune_type::mastodon::relationship::Relationship;
use speedy_uuid::Uuid;

#[debug_handler(state = crate::state::AppState)]
#[utoipa::path(
    post,
    path = "/api/v1/follow_requests/{id}/authorize",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = 200, description = "Follow request accepted", body = Relationship),
        (status = 404, description = "No pending follow request from that account ID")
    ),
)]
pub async fn post(
    State(account_service): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<Json<Relationship>> {
    if user_data.account.id == id {
        return Err(ApiError::BadRequest.into());
    }

    let follow_request = FollowRequest::builder()
        .account_id(user_data.account.id)
        .follower_id(id)
        .build();

    let follow_accounts = account_service
        .accept_follow_request(follow_request)
        .await?;

    if let Some(follow_accounts) = follow_accounts {
        Ok(Json(
            mastodon_mapper
                .map((&follow_accounts.0, &follow_accounts.1))
                .await?,
        ))
    } else {
        Err(ApiError::BadRequest.into())
    }
}
