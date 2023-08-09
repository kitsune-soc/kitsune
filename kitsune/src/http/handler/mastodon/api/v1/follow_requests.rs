use crate::{
    consts::API_DEFAULT_LIMIT,
    error::{ApiError, Result},
    http::{
        extractor::{AuthExtractor, MastodonAuthExtractor},
        pagination::{Link, PaginatedJsonResponse},
    },
    mapping::MastodonMapper,
    service::{
        account::{AccountService, FollowRequest, GetFollowRequests},
        url::UrlService,
    },
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::{Path, State},
    routing, Json, Router,
};
use axum_extra::extract::Query;
use futures_util::TryStreamExt;
use kitsune_type::mastodon::{relationship::Relationship, Account};
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
    #[serde(default = "default_limit")]
    limit: usize,
}

#[debug_handler(state = crate::state::Zustand)]
#[utoipa::path(
    get,
    path = "/api/v1/follow_requests",
    security(
        ("oauth_token" = [])
    ),
    params(GetQuery),
    responses(
        (status = 200, description = "List of accounts requesting a follow", body = Relationship)
    ),
)]
pub async fn get(
    State(account_service): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    State(url_service): State<UrlService>,
    Query(query): Query<GetQuery>,
    AuthExtractor(user_data): MastodonAuthExtractor,
) -> Result<PaginatedJsonResponse<Account>> {
    let get_follow_requests = GetFollowRequests::builder()
        .account_id(user_data.account.id)
        .limit(query.limit)
        .since_id(query.since_id)
        .max_id(query.max_id)
        .build();

    let accounts: Vec<Account> = account_service
        .get_follow_requests(get_follow_requests)
        .await?
        .and_then(|acc| mastodon_mapper.map(acc))
        .try_collect()
        .await?;

    let base_url = url_service.base_url();
    let link = if accounts.is_empty() {
        None
    } else {
        let next = (
            String::from("next"),
            format!(
                "{}/api/v1/follow_requests?max_id={}",
                base_url,
                accounts.last().unwrap().id
            ),
        );
        let prev = (
            String::from("prev"),
            format!(
                "{}/api/v1/follow_requests?since_id={}",
                base_url,
                accounts.first().unwrap().id
            ),
        );
        if accounts.len() >= query.limit && query.limit > 0 {
            Some(Link(vec![next, prev]))
        } else {
            Some(Link(vec![prev]))
        }
    };

    Ok((link, Json(accounts)))
}

#[debug_handler(state = crate::state::Zustand)]
#[utoipa::path(
    post,
    path = "/api/v1/follow_requests/{id}/authorize",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = 200, description = "Follow request accepted", body = Relationship),
        (status = 404, description = "No pending follow request from that account ID")
    ),
)]
pub async fn accept(
    State(account_service): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<Json<Relationship>> {
    if user_data.account.id == id {
        return Err(ApiError::BadRequest.into());
    }

    let follow_request = FollowRequest::builder()
        .account_id(user_data.account.id)
        .follower_id(id)
        .build();

    let follow_accounts = account_service
        .accept_follow_request(follow_request)
        .await?;

    if let Some(follow_accounts) = follow_accounts {
        Ok(Json(
            mastodon_mapper
                .map((&follow_accounts.0, &follow_accounts.1))
                .await?,
        ))
    } else {
        Err(ApiError::BadRequest.into())
    }
}

#[debug_handler(state = crate::state::Zustand)]
#[utoipa::path(
    post,
    path = "/api/v1/follow_requests/{id}/reject",
    security(
        ("oauth_token" = [])
    ),
    responses(
        (status = 200, description = "Follow request rejected", body = Relationship),
        (status = 404, description = "No pending follow request from that account ID")
    ),
)]
pub async fn reject(
    State(account_service): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Path(id): Path<Uuid>,
) -> Result<Json<Relationship>> {
    if user_data.account.id == id {
        return Err(ApiError::BadRequest.into());
    }

    let follow_request = FollowRequest::builder()
        .account_id(user_data.account.id)
        .follower_id(id)
        .build();

    let follow_accounts = account_service
        .reject_follow_request(follow_request)
        .await?;

    if let Some(follow_accounts) = follow_accounts {
        Ok(Json(
            mastodon_mapper
                .map((&follow_accounts.0, &follow_accounts.1))
                .await?,
        ))
    } else {
        Err(ApiError::BadRequest.into())
    }
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/", routing::get(get))
        .route("/:id/authorize", routing::post(accept))
        .route("/:id/reject", routing::post(reject))
}
