use ahash::AHashSet;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use futures_util::{FutureExt, TryFutureExt, TryStreamExt, stream::FuturesUnordered};
use garde::Validate;
use kitsune_core::{consts::API_MAX_LIMIT, traits::Fetcher};
use kitsune_db::{
    PgPool,
    model::{Account, Post},
    schema::{accounts, posts},
    with_connection,
};
use kitsune_derive::kitsune_service;
use kitsune_error::Result;
use kitsune_search::{SearchBackend, SearchIndex};
use speedy_uuid::Uuid;
use triomphe::Arc;
use typed_builder::TypedBuilder;
use url::Url;

pub enum SearchResult {
    Account(Account),
    Post(Post),
}

#[derive(TypedBuilder, Validate)]
pub struct Search<'a> {
    #[garde(skip)]
    indices: AHashSet<SearchIndex>,
    #[garde(skip)]
    query: &'a str,
    #[garde(range(max = API_MAX_LIMIT as u64))]
    max_results: u64,
    #[garde(skip)]
    offset: u64,
    #[garde(skip)]
    min_id: Option<Uuid>,
    #[garde(skip)]
    max_id: Option<Uuid>,
}

#[kitsune_service]
pub struct SearchService {
    db_pool: PgPool,
    fetcher: Arc<dyn Fetcher>,
    search_backend: kitsune_search::AnySearchBackend,
}

impl SearchService {
    #[must_use]
    pub fn backend(&self) -> &kitsune_search::AnySearchBackend {
        &self.search_backend
    }

    pub async fn search(&self, search: Search<'_>) -> Result<Vec<SearchResult>> {
        search.validate()?;

        let mut results = Vec::new();

        // TODO: Add Webfinger-based handle resolver

        if let Ok(searched_url) = Url::parse(search.query) {
            match self
                .fetcher
                .fetch_account(searched_url.as_str().into())
                .await
            {
                Ok(Some(account)) => results.push(SearchResult::Account(account)),
                Ok(None) => debug!("no account found"),
                Err(error) => {
                    debug!(?error, "couldn't fetch actor via url");
                }
            }

            match self.fetcher.fetch_post(searched_url.as_str().into()).await {
                Ok(Some(post)) => results.push(SearchResult::Post(post)),
                Ok(None) => debug!("no post found"),
                Err(error) => {
                    debug!(?error, "couldn't fetch object via url");
                }
            }
        }

        let result_references = search
            .indices
            .into_iter()
            .map(|index| {
                self.search_backend.search(
                    index,
                    search.query,
                    search.max_results,
                    search.offset,
                    search.min_id,
                    search.max_id,
                )
            })
            .collect::<FuturesUnordered<_>>()
            .try_concat()
            .await?;

        let search_backend_results = with_connection!(self.db_pool, |db_conn| {
            result_references
                .iter()
                .map(|result| match result.index {
                    SearchIndex::Account => accounts::table
                        .find(result.id)
                        .select(Account::as_select())
                        .get_result::<Account>(db_conn)
                        .map_ok(SearchResult::Account)
                        .left_future(),
                    SearchIndex::Post => posts::table
                        .find(result.id)
                        .select(Post::as_select())
                        .get_result::<Post>(db_conn)
                        .map_ok(SearchResult::Post)
                        .right_future(),
                })
                .collect::<FuturesUnordered<_>>()
                .try_collect::<Vec<SearchResult>>()
                .await
        })?;

        results.extend(search_backend_results);

        Ok(results)
    }
}
