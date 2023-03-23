use crate::{
    error::Result,
    http::extractor::MastodonAuthExtractor,
    mapping::MastodonMapper,
    service::account::{AccountService, GetPosts},
};
use axum::{
    extract::{Path, Query, State},
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
    max_id: Option<Uuid>,
    min_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[utoipa::path(
    get,
    path = "/api/v1/accounts/{id}/statuses",
    params(GetQuery),
    responses(
        (status = 200, description = "Statuses by the user", body = Vec<Status>),
    )
)]
pub async fn get(
    State(account): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    Path(account_id): Path<Uuid>,
    auth_data: Option<MastodonAuthExtractor>,
    Query(query): Query<GetQuery>,
) -> Result<Json<Vec<Status>>> {
    let fetching_account_id = auth_data.map(|user_data| user_data.0.account.id);

    let mut get_posts = GetPosts::builder().account_id(account_id).clone();
    if let Some(fetching_account_id) = fetching_account_id {
        get_posts.fetching_account_id(fetching_account_id);
    }
    if let Some(max_id) = query.max_id {
        get_posts.max_id(max_id);
    }
    if let Some(min_id) = query.min_id {
        get_posts.min_id(min_id);
    }

    let get_posts = get_posts.build().unwrap();
    let limit = min(query.limit, MAX_LIMIT);

    let statuses: Vec<Status> = account
        .get_posts(get_posts)
        .await?
        .take(limit)
        .and_then(|post| mastodon_mapper.map(post))
        .try_collect()
        .await?;

    Ok(Json(statuses))
}
