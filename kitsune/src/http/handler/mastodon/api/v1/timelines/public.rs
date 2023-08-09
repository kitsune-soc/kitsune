use crate::{
    consts::API_DEFAULT_LIMIT,
    error::Result,
    http::{
        extractor::{AuthExtractor, MastodonAuthExtractor},
        pagination::{Link, PaginatedJsonResponse},
    },
    mapping::MastodonMapper,
    service::{
        timeline::{GetPublic, TimelineService},
        url::UrlService,
    },
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
    since_id: Option<Uuid>,
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
    State(url_service): State<UrlService>,
    Query(query): Query<GetQuery>,
    user_data: Option<MastodonAuthExtractor>,
) -> Result<PaginatedJsonResponse<Status>> {
    let get_public = GetPublic::builder()
        .only_local(query.local)
        .only_remote(query.remote)
        .max_id(query.max_id)
        .since_id(query.since_id)
        .min_id(query.min_id)
        .limit(query.limit)
        .build();

    let mut statuses: Vec<Status> = timeline
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

    if query.min_id.is_some() {
        statuses.reverse();
    }

    let base_url = url_service.base_url();
    let link = if statuses.is_empty() {
        None
    } else {
        let next = (
            "next",
            format!(
                "{}/api/v1/timelines/public?limit={}&max_id={}",
                base_url,
                query.limit,
                statuses.last().unwrap().id
            ),
        );
        let prev = (
            "prev",
            format!(
                "{}/api/v1/timelines/public?limit={}&min_id={}",
                base_url,
                query.limit,
                statuses.first().unwrap().id
            ),
        );
        if statuses.len() >= query.limit && query.limit > 0 {
            Some(Link(vec![next, prev]))
        } else {
            Some(Link(vec![prev]))
        }
    };

    Ok((link, Json(statuses)))
}
