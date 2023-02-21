use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::MastodonMapper,
};
use axum::{extract::State, Json};
use kitsune_type::mastodon::Account;

pub async fn get(
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user): MastodonAuthExtractor,
) -> Result<Json<Account>> {
    Ok(Json(mastodon_mapper.map(user.account).await?))
}
