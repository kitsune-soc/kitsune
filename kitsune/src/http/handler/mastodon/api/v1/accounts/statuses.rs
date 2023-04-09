use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    mapping::MastodonMapper,
    service::account::{AccountService, GetPosts},
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use futures_util::{FutureExt, StreamExt, TryStreamExt};
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
    security(
        (),
        ("oauth_token" = [])
    ),
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
    user_data: Option<MastodonAuthExtractor>,
) -> Result<Json<Vec<Status>>> {
    let fetching_account_id = auth_data.map(|user_data| user_data.0.account.id);

    let get_posts = GetPosts::builder()
        .account_id(account_id)
        .fetching_account_id(fetching_account_id)
        .max_id(query.max_id)
        .min_id(query.min_id)
        .build();
    let limit = min(query.limit, MAX_LIMIT);

    let statuses: Vec<Status> = account
        .get_posts(get_posts)
        .await?
        .take(limit)
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
