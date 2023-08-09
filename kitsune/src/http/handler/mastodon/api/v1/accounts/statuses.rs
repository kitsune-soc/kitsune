use crate::{
    consts::API_DEFAULT_LIMIT,
    error::Result,
    http::{
        extractor::{AuthExtractor, MastodonAuthExtractor},
        pagination::{Link, PaginatedJsonResponse},
    },
    mapping::MastodonMapper,
    service::{
        account::{AccountService, GetPosts},
        url::UrlService,
    },
};
use axum::{
    extract::{Path, Query, State},
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
    max_id: Option<Uuid>,
    since_id: Option<Uuid>,
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
    State(url_service): State<UrlService>,
    Path(account_id): Path<Uuid>,
    auth_data: Option<MastodonAuthExtractor>,
    Query(query): Query<GetQuery>,
    user_data: Option<MastodonAuthExtractor>,
) -> Result<PaginatedJsonResponse<Status>> {
    let fetching_account_id = auth_data.map(|user_data| user_data.0.account.id);

    let get_posts = GetPosts::builder()
        .account_id(account_id)
        .fetching_account_id(fetching_account_id)
        .max_id(query.max_id)
        .since_id(query.since_id)
        .min_id(query.min_id)
        .limit(query.limit)
        .build();

    let mut statuses: Vec<Status> = account
        .get_posts(get_posts)
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
                "{}/api/v1/accounts/{}/statuses?limit={}&max_id={}",
                base_url,
                account_id,
                query.limit,
                statuses.last().unwrap().id
            ),
        );
        let prev = (
            "prev",
            format!(
                "{}/api/v1/accounts/{}/statuses?limit={}&min_id={}",
                base_url,
                account_id,
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
