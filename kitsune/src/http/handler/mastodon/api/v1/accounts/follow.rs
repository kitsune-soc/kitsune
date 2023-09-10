use crate::{
    error::Result,
    http::extractor::{AuthExtractor, FormOrJson, MastodonAuthExtractor},
};
use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use kitsune_core::{
    error::ApiError,
    mapping::MastodonMapper,
    service::account::{AccountService, Follow},
};
use kitsune_type::mastodon::relationship::Relationship;
use serde::Deserialize;
use speedy_uuid::Uuid;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct FollowBody {
    #[serde(default)]
    notify: bool,
}

#[debug_handler(state = crate::state::AppState)]
#[utoipa::path(
    post,
    path = "/api/v1/accounts/{id}/follow",
    security(
        ("oauth_token" = [])
    ),
    request_body = FollowBody,
    responses(
        (status = 200, description = "Followed user successfully", body = Relationship)
    ),
)]
pub async fn post(
    State(account_service): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
    FormOrJson(follow_body): FormOrJson<FollowBody>,
) -> Result<Json<Relationship>> {
    if user_data.account.id == id {
        return Err(ApiError::BadRequest.into());
    }

    let follow = Follow::builder()
        .account_id(id)
        .follower_id(user_data.account.id)
        .build();
    let follow_accounts = account_service.follow(follow, follow_body.notify).await?;

    Ok(Json(
        mastodon_mapper
            .map((&follow_accounts.0, &follow_accounts.1))
            .await?,
    ))
}
