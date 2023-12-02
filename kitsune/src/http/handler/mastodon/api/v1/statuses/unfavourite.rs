use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    state::PostService,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use kitsune_mastodon::MastodonMapper;
use kitsune_type::mastodon::Status;
use speedy_uuid::Uuid;

#[debug_handler(state = crate::state::Zustand)]
#[utoipa::path(
    post,
    path = "/api/v1/statuses/{id}/unfavourite",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = 200, description = "The status with updated information regarding like status"),
    )
)]
pub async fn post(
    State(mastodon_mapper): State<MastodonMapper>,
    State(post): State<PostService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<Json<Status>> {
    let post = post.unfavourite(id, user_data.account.id).await?;

    Ok(Json(mastodon_mapper.map((&user_data.account, post)).await?))
}
