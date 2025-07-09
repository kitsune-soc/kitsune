use crate::http::extractor::MastodonAuthExtractor;
use axum::{
    Json, debug_handler,
    extract::{Query, State},
};
use kitsune_error::{ErrorType, Result, kitsune_error};
use kitsune_mastodon::MastodonMapper;
use kitsune_service::account::{AccountService, GetUser};
use kitsune_type::mastodon::Account;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LookupQuery {
    acct: String,
}

#[debug_handler(state = crate::state::Zustand)]
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
        .use_resolver(false)
        .build();

    let account = account_service
        .get(get_user)
        .await?
        .ok_or_else(|| kitsune_error!(type = ErrorType::NotFound, "account not found"))?;

    Ok(Json(mastodon_mapper.map(account).await?))
}
