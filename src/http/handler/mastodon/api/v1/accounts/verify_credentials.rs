use crate::{error::Result, http::extractor::AuthExtactor, mapping::IntoMastodon, state::State};
use axum::{
    response::{IntoResponse, Response},
    Extension, Json,
};
use http::StatusCode;

pub async fn get(
    Extension(state): Extension<State>,
    AuthExtactor(user): AuthExtactor,
) -> Result<Response> {
    if let Some(user) = user {
        Ok(Json(user.into_mastodon(&state).await?).into_response())
    } else {
        Ok(StatusCode::UNAUTHORIZED.into_response())
    }
}
