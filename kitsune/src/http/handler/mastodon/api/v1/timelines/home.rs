use crate::{
    consts::API_DEFAULT_LIMIT,
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::MastodonMapper,
    service::timeline::{GetHome, TimelineService},
};
use axum::{
    extract::{Query, State},
    Json,
};
use futures_util::TryStreamExt;
use kitsune_type::mastodon::Status;
use serde::Deserialize;
use speedy_uuid::Uuid;
use utoipa::IntoParams;

fn default_limit() -> usize {
    API_DEFAULT_LIMIT
}

#[derive(Deserialize, IntoParams)]
pub struct GetQuery {
    max_id: Option<Uuid>,
    min_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[utoipa::path(
    get,
    path = "/api/v1/timelines/home",
    security(
        ("oauth_token" = [])
    ),
    params(GetQuery),
    responses(
        (status = 200, description = "Current home timeline", body = Vec<Status>),
    ),
)]
pub async fn get(
    State(mastodon_mapper): State<MastodonMapper>,
    State(timeline): State<TimelineService>,
    Query(query): Query<GetQuery>,
    AuthExtractor(user_data): MastodonAuthExtractor,
) -> Result<Json<Vec<Status>>> {
    let get_home = GetHome::builder()
        .fetching_account_id(user_data.account.id)
        .max_id(query.max_id)
        .min_id(query.min_id)
        .limit(query.limit)
        .build();

    let statuses: Vec<Status> = timeline
        .get_home(get_home)
        .await?
        .and_then(|post| mastodon_mapper.map((&user_data.account, post)))
        .try_collect()
        .await?;

    Ok(Json(statuses))
}
