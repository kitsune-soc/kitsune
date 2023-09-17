use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use kitsune_core::{
    mapping::MastodonMapper,
    service::post::{PostService, UnrepostPost},
};
use kitsune_type::mastodon::Status;
use speedy_uuid::Uuid;

#[debug_handler(state = Zustand)]
#[utoipa::path(
    delete,
    path = "/api/v1/statuses/{id}/unreblog",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = StatusCode::OK, description = "Status was unboosted or was already not boosted", body = Status),
        (status = StatusCode::NOT_FOUND, description = "Status does not exist or is private")
    )
)]
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
