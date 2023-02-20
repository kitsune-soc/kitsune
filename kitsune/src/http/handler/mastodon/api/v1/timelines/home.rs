use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::IntoMastodon,
    service::timeline::{GetHome, TimelineService},
    state::Zustand,
};
use axum::{
    extract::{Query, State},
    Json,
};
use futures_util::{StreamExt, TryStreamExt};
use kitsune_type::mastodon::Status;
use serde::Deserialize;
use std::cmp::min;
use uuid::Uuid;

const MAX_LIMIT: usize = 40;

fn default_limit() -> usize {
    20
}

#[derive(Deserialize)]
pub struct GetQuery {
    max_id: Option<Uuid>,
    min_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    limit: usize,
}

pub async fn get(
    State(state): State<Zustand>,
    State(timeline): State<TimelineService>,
    Query(query): Query<GetQuery>,
    AuthExtractor(user_data): MastodonAuthExtractor,
) -> Result<Json<Vec<Status>>> {
    let mut get_home = GetHome::builder()
        .fetching_account_id(user_data.account.id)
        .clone();

    if let Some(max_id) = query.max_id {
        get_home.max_id(max_id);
    }
    if let Some(min_id) = query.min_id {
        get_home.min_id(min_id);
    }

    let limit = min(query.limit, MAX_LIMIT);
    let get_home = get_home.build().unwrap();

    let statuses: Vec<Status> = timeline
        .get_home(get_home)
        .await?
        .take(limit)
        .and_then(|post| post.into_mastodon(&state))
        .try_collect()
        .await?;

    Ok(Json(statuses))
}
