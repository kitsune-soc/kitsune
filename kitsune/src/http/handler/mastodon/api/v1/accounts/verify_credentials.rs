use crate::http::extractor::{AuthExtractor, MastodonAuthExtractor};
use axum::{extract::State, Json};
use kitsune_error::Result;
use kitsune_mastodon::MastodonMapper;
use kitsune_type::mastodon::Account;

pub async fn get(
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user): MastodonAuthExtractor,
) -> Result<Json<Account>> {
    Ok(Json(mastodon_mapper.map(user.account).await?))
}
