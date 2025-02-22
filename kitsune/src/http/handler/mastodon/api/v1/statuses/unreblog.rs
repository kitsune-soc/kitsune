use crate::http::extractor::{AuthExtractor, MastodonAuthExtractor};
use axum::{
    Json, debug_handler,
    extract::{Path, State},
};
use kitsune_error::Result;
use kitsune_mastodon::MastodonMapper;
use kitsune_service::post::{PostService, UnrepostPost};
use kitsune_type::mastodon::Status;
use speedy_uuid::Uuid;

#[debug_handler(state = crate::state::Zustand)]
pub async fn post(
    State(mastodon_mapper): State<MastodonMapper>,
    State(post): State<PostService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<Json<Status>> {
    let unrepost_post = UnrepostPost::builder()
        .account_id(user_data.account.id)
        .post_id(id)
        .build();

    let status = mastodon_mapper
        .map((&user_data.account, post.unrepost(unrepost_post).await?))
        .await?;

    Ok(Json(status))
}
