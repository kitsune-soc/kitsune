use crate::{
    consts::default_limit,
    error::Result,
    http::{
        extractor::{AuthExtractor, MastodonAuthExtractor},
        pagination::{LinkHeader, PaginatedJsonResponse},
    },
};
use axum::{
    extract::{OriginalUri, Query, State},
    Json,
};
use futures_util::TryStreamExt;
use kitsune_core::{
    mapping::MastodonMapper,
    service::{
        timeline::{GetHome, TimelineService},
        url::UrlService,
    },
};
use kitsune_type::mastodon::Status;
use serde::Deserialize;
use speedy_uuid::Uuid;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct GetQuery {
    max_id: Option<Uuid>,
    since_id: Option<Uuid>,
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
    State(url_service): State<UrlService>,
    OriginalUri(original_uri): OriginalUri,
    Query(query): Query<GetQuery>,
    AuthExtractor(user_data): MastodonAuthExtractor,
) -> Result<PaginatedJsonResponse<Status>> {
    let get_home = GetHome::builder()
        .fetching_account_id(user_data.account.id)
        .max_id(query.max_id)
        .since_id(query.since_id)
        .min_id(query.min_id)
        .limit(query.limit)
        .build();

    let mut statuses: Vec<Status> = timeline
        .get_home(get_home)
        .await?
        .and_then(|post| mastodon_mapper.map((&user_data.account, post)))
        .try_collect()
        .await?;

    if query.min_id.is_some() {
        statuses.reverse();
    }

    let link_header = LinkHeader::new(
        &statuses,
        query.limit,
        &url_service.base_url(),
        original_uri.path(),
        |s| s.id,
    );

    Ok((link_header, Json(statuses)))
}
