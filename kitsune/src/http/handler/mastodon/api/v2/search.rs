use crate::{
    error::{Error, Result},
    http::extractor::{AuthExtractor, MastodonAuthExtractor},
    state::AppState,
};
use axum::{debug_handler, extract::State, routing, Json, Router};
use axum_extra::{either::Either, extract::Query};
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use http::StatusCode;
use kitsune_core::{
    consts::{API_DEFAULT_LIMIT, API_MAX_LIMIT},
    error::Error as CoreError,
};
use kitsune_db::{
    model::{account::Account, post::Post},
    schema::{accounts, posts},
};
use kitsune_search::{SearchBackend, SearchIndex, SearchService};
use kitsune_type::mastodon::SearchResult;
use scoped_futures::ScopedFutureExt;
use serde::Deserialize;
use speedy_uuid::Uuid;
use std::cmp::min;
use url::Url;
use utoipa::{IntoParams, ToSchema};

fn default_page_limit() -> u64 {
    API_DEFAULT_LIMIT as u64
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

#[debug_handler(state = AppState)]
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
    State(state): State<AppState>,
    State(search): State<SearchService>,
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

    // TODO: Find a way to pipeline
    let mut search_result = SearchResult::default();
    for index in indices {
        let results = search
            .search(
                index,
                query.query.clone(),
                min(query.limit, API_MAX_LIMIT as u64),
                query.offset,
                query.min_id,
                query.max_id,
            )
            .await
            .map_err(CoreError::from)?;

        for result in results {
            search_result = state
                .db_pool()
                .with_connection(|db_conn| {
                    async {
                        match index {
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

                                search_result
                                    .statuses
                                    .push(state.mastodon_mapper().map(post).await?);
                            }
                        }

                        Ok::<_, Error>(search_result)
                    }
                    .scoped()
                })
                .await?;
        }
    }

    if Url::parse(&query.query).is_ok() {
        match state
            .fetcher()
            .fetch_actor(query.query.as_str().into())
            .await
        {
            Ok(account) => search_result
                .accounts
                .insert(0, state.mastodon_mapper().map(account).await?),
            Err(error) => debug!(?error, "couldn't fetch actor via url"),
        }

        match state.fetcher().fetch_object(query.query.as_str()).await {
            Ok(post) => search_result.statuses.insert(
                0,
                state
                    .mastodon_mapper()
                    .map((&user_data.account, post))
                    .await?,
            ),
            Err(error) => debug!(?error, "couldn't fetch object via url"),
        }
    }

    Ok(Either::E1(Json(search_result)))
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/", routing::get(get))
}
