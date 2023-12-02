use crate::{
    consts::default_limit,
    error::{Error, Result},
    http::{
        extractor::{AuthExtractor, MastodonAuthExtractor},
        pagination::{LinkHeader, PaginatedJsonResponse},
    },
    state::{AccountService, Zustand},
};
use axum::{
    debug_handler,
    extract::{OriginalUri, State},
    routing, Json, Router,
};
use axum_extra::extract::Query;
use futures_util::{TryFutureExt, TryStreamExt};
use kitsune_mastodon::MastodonMapper;
use kitsune_service::{account::GetFollowRequests, url::UrlService};
use kitsune_type::mastodon::Account;
use serde::Deserialize;
use speedy_uuid::Uuid;
use utoipa::IntoParams;

pub mod accept;
pub mod reject;

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
        (status = 200, description = "List of accounts requesting a follow", body = Vec<Account>)
    ),
)]
pub async fn get(
    State(account_service): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    State(url_service): State<UrlService>,
    OriginalUri(original_uri): OriginalUri,
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
        .map_err(Error::from)
        .and_then(|acc| mastodon_mapper.map(acc).map_err(Error::from))
        .try_collect()
        .await?;

    let link_header = LinkHeader::new(
        &accounts,
        query.limit,
        &url_service.base_url(),
        original_uri.path(),
        |a| a.id,
    );

    Ok((link_header, Json(accounts)))
}

pub fn routes() -> Router<Zustand> {
    Router::new()
        .route("/", routing::get(get))
        .route("/:id/authorize", routing::post(accept::post))
        .route("/:id/reject", routing::post(reject::post))
}
