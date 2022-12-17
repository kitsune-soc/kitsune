use crate::{error::Result, http::extractor::AuthExtactor, mapping::IntoMastodon, state::Zustand};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;

pub async fn get(
    State(state): State<Zustand>,
    AuthExtactor(user): AuthExtactor,
) -> Result<Response> {
    if let Some(user) = user {
        Ok(Json(user.account.into_mastodon(&state).await?).into_response())
    } else {
        Ok(StatusCode::UNAUTHORIZED.into_response())
    }
}
