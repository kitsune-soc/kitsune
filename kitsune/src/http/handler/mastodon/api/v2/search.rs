use crate::{
    consts::default_limit,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    state::Zustand,
};
use axum::{debug_handler, extract::State, Json};
use axum_extra::{either::Either, extract::Query};
use http::StatusCode;
use kitsune_core::consts::API_MAX_LIMIT;
use kitsune_error::Result;
use kitsune_mastodon::MastodonMapper;
use kitsune_search::SearchIndex;
use kitsune_service::search::{Search, SearchResult, SearchService};
use kitsune_type::mastodon::SearchResult as MastodonSearchResult;
use serde::Deserialize;
use speedy_uuid::Uuid;
use std::cmp::min;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchType {
    Accounts,
    Hashtags,
    Statuses,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    #[serde(rename = "q")]
    query: String,
    r#type: Option<SearchType>,
    /*#[serde(default)]
    resolve: bool,*/
    max_id: Option<Uuid>,
    min_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    limit: u64,
    #[serde(default)]
    offset: u64,
}

#[debug_handler(state = Zustand)]
pub async fn get(
    State(search_service): State<SearchService>,
    State(mastodon_mapper): State<MastodonMapper>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Query(query): Query<SearchQuery>,
) -> Result<Either<Json<MastodonSearchResult>, StatusCode>> {
    let indices = if let Some(r#type) = query.r#type {
        let index = match r#type {
            SearchType::Accounts => SearchIndex::Account,
            SearchType::Statuses => SearchIndex::Post,
            SearchType::Hashtags => return Ok(Either::E2(StatusCode::BAD_REQUEST)),
        };

        vec![index]
    } else {
        vec![SearchIndex::Account, SearchIndex::Post]
    };

    let search = Search::builder()
        .indices(indices.into_iter().collect())
        .max_id(query.max_id)
        .max_results(min(query.limit, API_MAX_LIMIT as u64))
        .min_id(query.min_id)
        .offset(query.offset)
        .query(&query.query)
        .build();
    let results = search_service.search(search).await?;

    let mut search_result = MastodonSearchResult::default();
    for result in results {
        match result {
            SearchResult::Account(account) => search_result
                .accounts
                .push(mastodon_mapper.map(account).await?),
            SearchResult::Post(post) => search_result
                .statuses
                .push(mastodon_mapper.map((&user_data.account, post)).await?),
        }
    }

    Ok(Either::E1(Json(search_result)))
}
