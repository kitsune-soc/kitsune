use crate::http::extractor::{AuthExtractor, MastodonAuthExtractor};
use axum::{
    Json, debug_handler,
    extract::{Path, State},
};
use kitsune_error::Result;
use kitsune_mastodon::MastodonMapper;
use kitsune_service::post::PostService;
use kitsune_type::mastodon::Status;
use speedy_uuid::Uuid;

#[debug_handler(state = crate::state::Zustand)]
pub async fn post(
    State(mastodon_mapper): State<MastodonMapper>,
    State(post): State<PostService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<Json<Status>> {
    let post = post.unfavourite(id, user_data.account.id).await?;

    Ok(Json(mastodon_mapper.map((&user_data.account, post)).await?))
}
