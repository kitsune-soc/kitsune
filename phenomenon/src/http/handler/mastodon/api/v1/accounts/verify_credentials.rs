use crate::{error::Result, http::extractor::AuthExtactor, mapping::IntoMastodon, state::Zustand};
use axum::{extract::State, Json};
use phenomenon_type::mastodon::Account;

pub async fn get(
    State(state): State<Zustand>,
    AuthExtactor(user): AuthExtactor,
) -> Result<Json<Account>> {
    Ok(Json(user.account.into_mastodon(&state).await?))
}
