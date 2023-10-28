use crate::{
    consts::default_limit,
    error::{Error, Result},
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    state::Zustand,
};
use axum::{debug_handler, extract::State, routing, Json, Router};
use axum_extra::{either::Either, extract::Query};
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::stream::{FuturesUnordered, TryStreamExt};
use http::StatusCode;
use kitsune_core::{
    consts::API_MAX_LIMIT,
    service::search::{Search, SearchService},
};
use kitsune_db::{
    model::{account::Account, post::Post},
    schema::{accounts, posts},
};
use kitsune_search::SearchIndex;
use kitsune_type::mastodon::SearchResult;
use scoped_futures::ScopedFutureExt;
use serde::Deserialize;
use speedy_uuid::Uuid;
use std::cmp::min;
use utoipa::{IntoParams, ToSchema};

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
    #[serde(default = "default_limit")]
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
    State(search_service): State<SearchService>,
    AuthExtractor(user_data): MastodonAuthExtractor,
    Query(query): Query<SearchQuery>,
) -> Result<Either<Json<SearchResult>, StatusCode>> {
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
        .indices(&indices)
        .max_id(query.max_id)
        .max_results(min(query.limit, API_MAX_LIMIT as u64))
        .min_id(query.min_id)
        .offset(query.offset)
        .query(&query.query)
        .build();
    let results = search_service.search(search).await?;
    // TODO: Find a way to pipeline

    state
        .db_pool()
        .with_connection(|db_conn| {
            async move {
                let mut search_result = SearchResult::default();

                results
                    .iter()
                    .map(|result| async {
                        match result.index {
                            SearchIndex::Account => Either::E1(
                                accounts::table
                                    .find(result.id)
                                    .select(Account::as_select())
                                    .get_result::<Account>(db_conn)
                                    .await,
                            ),
                            SearchIndex::Post => Either::E2(
                                posts::table
                                    .find(result.id)
                                    .select(Post::as_select())
                                    .get_result::<Post>(db_conn)
                                    .await,
                            ),
                        }
                    })
                    .collect::<FuturesUnordered<_>>()
                    .try_collect::<Vec<Either<Account, Post>>>()
                    .await?;

                Ok(search_result)
            }
            .scoped()
        })
        .await;

    for result in results {
        search_result = state
            .db_pool()
            .with_connection(|db_conn| {
                async {
                    match result.index {
                        SearchIndex::Account => {
                            let account = accounts::table
                                .find(result.id)
                                .select(Account::as_select())
                                .get_result::<Account>(db_conn)
                                .await?;

                            search_result
                                .accounts
                                .push(state.mastodon_mapper().map(account).await?);
                        }
                        SearchIndex::Post => {
                            let post = posts::table
                                .find(result.id)
                                .select(Post::as_select())
                                .get_result::<Post>(db_conn)
                                .await?;

                            search_result.statuses.push(
                                state
                                    .mastodon_mapper()
                                    .map((&user_data.account, post))
                                    .await?,
                            );
                        }
                    }

                    Ok::<_, Error>(search_result)
                }
                .scoped()
            })
            .await?;
    }

    Ok(Either::E1(Json(search_result)))
}

pub fn routes() -> Router<Zustand> {
    Router::new().route("/", routing::get(get))
}
