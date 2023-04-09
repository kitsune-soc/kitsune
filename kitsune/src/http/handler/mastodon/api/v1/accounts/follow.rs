use crate::{
    error::{ApiError, Result},
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::MastodonMapper,
    service::account::{AccountService, Follow},
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
#[utoipa::path(
    post,
    path = "/api/v1/accounts/{id}/follow",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = 200, description = "Followed user successfully", body = Relationship)
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

    let follow = Follow::builder()
        .account_id(id)
        .follower_id(user_data.account.id)
        .build();
    let follow_accounts = account_service.follow(follow).await?;

    Ok(Json(
        mastodon_mapper
            .map((&follow_accounts.0, &follow_accounts.1))
            .await?,
    ))
}
