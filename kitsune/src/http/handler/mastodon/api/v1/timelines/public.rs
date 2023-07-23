use crate::{
    consts::API_DEFAULT_LIMIT,
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::MastodonMapper,
    service::timeline::{GetPublic, TimelineService},
};
use axum::{
    extract::{Query, State},
    Json,
};
use futures_util::{FutureExt, TryStreamExt};
use kitsune_type::mastodon::Status;
use serde::Deserialize;
use speedy_uuid::Uuid;
use utoipa::IntoParams;

fn default_limit() -> usize {
    API_DEFAULT_LIMIT
}

#[derive(Deserialize, IntoParams)]
pub struct GetQuery {
    #[serde(default)]
    local: bool,
    #[serde(default)]
    remote: bool,
    max_id: Option<Uuid>,
    min_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[utoipa::path(
    get,
    path = "/api/v1/timelines/public",
    params(GetQuery),
    responses(
        (status = 200, description = "Current public timeline", body = Vec<Status>),
    ),
)]
pub async fn get(
    State(mastodon_mapper): State<MastodonMapper>,
    State(timeline): State<TimelineService>,
    Query(query): Query<GetQuery>,
    user_data: Option<MastodonAuthExtractor>,
) -> Result<Json<Vec<Status>>> {
    let get_public = GetPublic::builder()
        .only_local(query.local)
        .only_remote(query.remote)
        .max_id(query.max_id)
        .min_id(query.min_id)
        .limit(query.limit)
        .build();

    let statuses: Vec<Status> = timeline
        .get_public(get_public)
        .await?
        .and_then(|post| {
            if let Some(AuthExtractor(ref user_data)) = user_data {
                mastodon_mapper
                    .map((&user_data.account, post))
                    .left_future()
            } else {
                mastodon_mapper.map(post).right_future()
            }
        })
        .try_collect()
        .await?;

    Ok(Json(statuses))
}
