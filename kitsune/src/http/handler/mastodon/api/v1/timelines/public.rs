use crate::{
    error::Result,
    mapping::MastodonMapper,
    service::timeline::{GetPublic, TimelineService},
};
use axum::{
    extract::{Query, State},
    Json,
};
use futures_util::{StreamExt, TryStreamExt};
use kitsune_type::mastodon::Status;
use serde::Deserialize;
use std::cmp::min;
use utoipa::IntoParams;
use uuid::Uuid;

const MAX_LIMIT: usize = 40;

fn default_limit() -> usize {
    20
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
) -> Result<Json<Vec<Status>>> {
    let mut get_public = GetPublic::builder()
        .only_local(query.local)
        .only_remote(query.remote)
        .clone();

    if let Some(max_id) = query.max_id {
        get_public.max_id(max_id);
    }
    if let Some(min_id) = query.min_id {
        get_public.min_id(min_id);
    }

    let limit = min(query.limit, MAX_LIMIT);
    let get_public = get_public.build().unwrap();

    let statuses: Vec<Status> = timeline
        .get_public(get_public)
        .await?
        .take(limit)
        .and_then(|post| mastodon_mapper.map(post))
        .try_collect()
        .await?;

    Ok(Json(statuses))
}
