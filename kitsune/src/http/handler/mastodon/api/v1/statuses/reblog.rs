use crate::{
    error::Result,
    http::extractor::{AuthExtractor, FormOrJson, MastodonAuthExtractor},
    mapping::MastodonMapper,
    service::post::{PostService, RepostPost},
};
use axum::{
    debug_handler,
    extract::{Path, State},
    Json,
};
use kitsune_type::mastodon::{status::Visibility, Status};
use serde::Deserialize;
use speedy_uuid::Uuid;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct RepostBody {
    #[serde(default)]
    visibility: Visibility,
}

#[debug_handler(state = kitsune_core::state::Zustand)]
#[utoipa::path(
    post,
    path = "/api/v1/statuses/{id}/reblog",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = StatusCode::OK, description = "The new status referencing the reblog", body = Status),
        (status = StatusCode::NOT_FOUND, description = "Requested status doesn't exist"),
    )
)]
pub async fn post(
    State(mastodon_mapper): State<MastodonMapper>,
    State(post): State<PostService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
    FormOrJson(body): FormOrJson<RepostBody>,
) -> Result<Json<Status>> {
    let repost_post = RepostPost::builder()
        .account_id(user_data.account.id)
        .post_id(id)
        .visibility(body.visibility.into())
        .build()
        .unwrap();

    let status = mastodon_mapper
        .map((&user_data.account, post.repost(repost_post).await?))
        .await?;

    Ok(Json(status))
}
