use crate::http::extractor::{Json, MastodonAuthExtractor};
use axum::{
    debug_handler,
    extract::{Path, State},
};
use kitsune_error::Result;
use kitsune_mastodon::MastodonMapper;
use kitsune_service::post::PostService;
use kitsune_type::mastodon::status::StatusSource;
use speedy_uuid::Uuid;

#[debug_handler(state = crate::state::Zustand)]
#[utoipa::path(
    get,
    path = "/api/v1/statuses/{id}/source",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = StatusCode::OK, description = "Source of the status", body = StatusSource),
    )
)]
pub async fn get(
    State(mastodon_mapper): State<MastodonMapper>,
    State(post_service): State<PostService>,
    auth_data: Option<MastodonAuthExtractor>,
    Path(id): Path<Uuid>,
) -> Result<Json<StatusSource>> {
    let fetching_account_id = auth_data.map(|auth_data| auth_data.0.account.id);
    let post_source = post_service
        .get_source_by_id(id, fetching_account_id)
        .await?;

    Ok(Json(mastodon_mapper.map(post_source).await?))
}
