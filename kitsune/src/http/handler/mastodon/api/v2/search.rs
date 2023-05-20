use crate::{
    error::Result,
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    service::search::{SearchBackend, SearchIndex, SearchService},
    state::Zustand,
};
use axum::{
    debug_handler,
    extract::State,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_extra::extract::Query;
use http::StatusCode;
use kitsune_type::mastodon::SearchResult;
use serde::Deserialize;
use url::Url;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

fn default_page_limit() -> u64 {
    40
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum SearchType {
    Accounts,
    Hashtags,
    Statuses,
}

#[derive(Deserialize, IntoParams)]
struct SearchQuery {
    #[serde(rename = "q")]
    query: String,
    r#type: Option<SearchType>,
    /*#[serde(default)]
    resolve: bool,*/
    max_id: Option<Uuid>,
    min_id: Option<Uuid>,
    #[serde(default = "default_page_limit")]
    limit: u64,
    #[serde(default)]
    offset: u64,
}

#[debug_handler(state = Zustand)]
#[utoipa::path(
    get,
    path = "/api/v2/search",
    security(
        ("oauth_token" = [])
    ),
    params(SearchQuery),
    responses(
        (status = 200, description = "Search results", body = SearchResult),
    ),
)]
async fn get(
    State(state): State<Zustand>,
    State(search): State<SearchService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Query(query): Query<SearchQuery>,
) -> Result<Response> {
    let indices = if let Some(r#type) = query.r#type {
        let index = match r#type {
            SearchType::Accounts => SearchIndex::Account,
            SearchType::Statuses => SearchIndex::Post,
            SearchType::Hashtags => return Ok(StatusCode::BAD_REQUEST.into_response()),
        };

        vec![index]
    } else {
        vec![SearchIndex::Account, SearchIndex::Post]
    };

    let mut search_result = SearchResult::default();
    for index in indices {
        let results = search
            .search(
                index,
                query.query.clone(),
                query.limit,
                query.offset,
                query.min_id,
                query.max_id,
            )
            .await?;

        for result in results {
            match index {
                SearchIndex::Account => {
                    let account = Accounts::find_by_id(result.id)
                        .one(&state.db_conn)
                        .await?
                        .expect("[Bug] Account indexed in search not in database");

                    search_result
                        .accounts
                        .push(state.mastodon_mapper.map(account).await?);
                }
                SearchIndex::Post => {
                    let post = Posts::find_by_id(result.id)
                        .one(&state.db_conn)
                        .await?
                        .expect("[Bug] Post indexed in search not in database");

                    search_result
                        .statuses
                        .push(state.mastodon_mapper.map(post).await?);
                }
            }
        }
    }

    if Url::parse(&query.query).is_ok() {
        if let Ok(account) = state.fetcher.fetch_actor(query.query.as_str().into()).await {
            search_result
                .accounts
                .insert(0, state.mastodon_mapper.map(account).await?);
        }

        if let Ok(post) = state.fetcher.fetch_object(query.query.as_str()).await {
            search_result.statuses.insert(
                0,
                state
                    .mastodon_mapper
                    .map((&user_data.account, post))
                    .await?,
            );
        }
    }

    Ok(Json(search_result).into_response())
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
