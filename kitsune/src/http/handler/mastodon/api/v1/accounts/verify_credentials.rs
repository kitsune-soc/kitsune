use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
};
use axum::{extract::State, Json};
use kitsune_mastodon::MastodonMapper;
use kitsune_type::mastodon::Account;

#[utoipa::path(
    get,
    path = "/api/v1/accounts/verify_credentials",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = 200, description = "Account of the logged in user", body = Account),
        (status = StatusCode::BAD_REQUEST, description = "The request was invalid"),
    )
)]
pub async fn get(
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user): MastodonAuthExtractor,
) -> Result<Json<Account>> {
    Ok(Json(mastodon_mapper.map(user.account).await?))
}
