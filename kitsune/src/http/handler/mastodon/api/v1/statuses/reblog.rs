use crate::http::extractor::{AgnosticForm, AuthExtractor, MastodonAuthExtractor};
use axum::{
    Json, debug_handler,
    extract::{Path, State},
};
use kitsune_error::Result;
use kitsune_mastodon::MastodonMapper;
use kitsune_service::post::{PostService, RepostPost};
use kitsune_type::mastodon::{Status, status::Visibility};
use serde::Deserialize;
use speedy_uuid::Uuid;

#[derive(Deserialize)]
pub struct RepostBody {
    #[serde(default)]
    visibility: Visibility,
}

#[debug_handler(state = crate::state::Zustand)]
pub async fn post(
    State(mastodon_mapper): State<MastodonMapper>,
    State(post): State<PostService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
    AgnosticForm(body): AgnosticForm<RepostBody>,
) -> Result<Json<Status>> {
    let repost_post = RepostPost::builder()
        .account_id(user_data.account.id)
        .post_id(id)
        .visibility(body.visibility.into())
        .build();

    let status = mastodon_mapper
        .map((&user_data.account, post.repost(repost_post).await?))
        .await?;

    Ok(Json(status))
}
