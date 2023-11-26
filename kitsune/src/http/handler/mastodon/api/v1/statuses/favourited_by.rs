use crate::{
    consts::default_limit,
    error::Result,
    http::{
        extractor::MastodonAuthExtractor,
        pagination::{LinkHeader, PaginatedJsonResponse},
    },
};
use axum::{
    debug_handler,
    extract::{OriginalUri, Path, State},
    Json,
};
use axum_extra::extract::Query;
use futures_util::TryStreamExt;
use kitsune_mastodon::MastodonMapper;
use kitsune_service::{
    post::{GetAccountsInteractingWithPost, PostService},
    url::UrlService,
};
use kitsune_type::mastodon::Account;
use serde::Deserialize;
use speedy_uuid::Uuid;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct GetQuery {
    min_id: Option<Uuid>,
    max_id: Option<Uuid>,
    since_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    limit: usize,
}

#[debug_handler(state = crate::state::Zustand)]
#[utoipa::path(
    get,
    path = "/api/v1/statuses/{id}/favourited_by",
    security(
        ("oauth_token" = [])
    ),
    params(GetQuery),
    responses(
        (status = 200, description = "List of accounts that favourited the status", body = Vec<Account>)
    ),
)]
pub async fn get(
    State(post_service): State<PostService>,
    State(mastodon_mapper): State<MastodonMapper>,
    State(url_service): State<UrlService>,
    OriginalUri(original_uri): OriginalUri,
    Query(query): Query<GetQuery>,
    user_data: Option<MastodonAuthExtractor>,
    Path(id): Path<Uuid>,
) -> Result<PaginatedJsonResponse<Account>> {
    let fetching_account_id = user_data.map(|user_data| user_data.0.account.id);
    let get_favourites = GetAccountsInteractingWithPost::builder()
        .post_id(id)
        .fetching_account_id(fetching_account_id)
        .limit(query.limit)
        .since_id(query.since_id)
        .min_id(query.min_id)
        .max_id(query.max_id)
        .build();

    let accounts: Vec<Account> = post_service
        .favourited_by(get_favourites)
        .await?
        .and_then(|acc| mastodon_mapper.map(acc))
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
