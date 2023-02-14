use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::IntoMastodon,
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

#[debug_handler(state = Zustand)]
pub async fn post(
    State(state): State<Zustand>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let post = state
        .service
        .post
        .favourite(id, user_data.account.id)
        .await?;

    Ok(Json(post.into_mastodon(&state).await?).into_response())
}
