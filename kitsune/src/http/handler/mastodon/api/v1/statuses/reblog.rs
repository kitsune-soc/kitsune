use crate::{
    error::Result,
    http::extractor::{AuthExtractor, FormOrJson, MastodonAuthExtractor},
    mapping::MastodonMapper,
    service::post::{BoostPost, PostService},
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
pub struct BoostBody {
    #[serde(default)]
    visibility: Visibility,
}

#[debug_handler(state = crate::state::Zustand)]
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
    FormOrJson(body): FormOrJson<BoostBody>,
) -> Result<Json<Status>> {
    let boost_post = BoostPost::builder()
        .account_id(user_data.account.id)
        .post_id(id)
        .visibility(body.visibility.into())
        .build()
        .unwrap();

    let status = mastodon_mapper.map(post.boost(boost_post).await?).await?;

    Ok(Json(status))
}
