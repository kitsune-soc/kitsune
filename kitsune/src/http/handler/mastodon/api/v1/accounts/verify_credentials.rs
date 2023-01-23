use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::IntoMastodon,
    state::Zustand,
};
use axum::{extract::State, Json};
use kitsune_type::mastodon::Account;

pub async fn get(
    State(state): State<Zustand>,
    AuthExtractor(user): MastodonAuthExtractor,
) -> Result<Json<Account>> {
    Ok(Json(user.account.into_mastodon(&state).await?))
}
