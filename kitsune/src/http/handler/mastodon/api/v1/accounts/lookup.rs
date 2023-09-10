use crate::{error::Result, http::extractor::MastodonAuthExtractor};
use axum::{
    debug_handler,
    extract::{Query, State},
    Json,
};
use kitsune_core::{
    error::ApiError,
    mapping::MastodonMapper,
    service::account::{AccountService, GetUser},
};
use kitsune_type::mastodon::Account;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct LookupQuery {
    acct: String,
}

#[debug_handler(state = crate::state::AppState)]
#[utoipa::path(
    get,
    path = "/api/v1/accounts/lookup",
    security(
        ("oauth_token" = [])
    ),
    params(LookupQuery),
    responses(
        (status = 200, description = "Return the account that goes by the acct structure", body = Account),
        (status = 404, description = "The account isn't known and has to be looked up via Webfinger"),
    ),
)]
pub async fn get(
    State(account_service): State<AccountService>,
    State(mastodon_mapper): State<MastodonMapper>,
    _: MastodonAuthExtractor,
    Query(query): Query<LookupQuery>,
) -> Result<Json<Account>> {
    let (username, domain) = match query.acct.split_once('@') {
        Some((username, domain)) => (username, Some(domain)),
        None => (query.acct.as_str(), None),
    };

    let get_user = GetUser::builder()
        .username(username)
        .domain(domain)
        .use_webfinger(false)
        .build();

    let account = account_service
        .get(get_user)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(mastodon_mapper.map(account).await?))
}
