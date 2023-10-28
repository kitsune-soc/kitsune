use crate::{activitypub::Fetcher, consts::API_MAX_LIMIT, error::Result};
use garde::Validate;
use kitsune_search::{SearchBackend, SearchIndex, SearchResult};
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;
use url::Url;

#[derive(TypedBuilder, Validate)]
pub struct Search<'a> {
    #[garde(skip)]
    indices: &'a [SearchIndex],
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

#[derive(Clone, TypedBuilder)]
pub struct SearchService {
    fetcher: Fetcher,
    search_backend: kitsune_search::Search,
}

impl SearchService {
    #[must_use]
    pub fn backend(&self) -> &kitsune_search::Search {
        &self.search_backend
    }

    pub async fn search(&self, search: Search<'_>) -> Result<Vec<SearchResult>> {
        search.validate(&())?;

        let mut results = Vec::new();

        // TODO: Add Webfinger-based handle resolver

        if let Ok(searched_url) = Url::parse(search.query) {
            match self.fetcher.fetch_actor(searched_url.as_str().into()).await {
                Ok(account) => results.push(SearchResult {
                    index: SearchIndex::Account,
                    id: account.id,
                }),
                Err(error) => debug!(?error, "couldn't fetch actor via url"),
            }

            match self.fetcher.fetch_object(searched_url.as_str()).await {
                Ok(post) => results.push(SearchResult {
                    index: SearchIndex::Post,
                    id: post.id,
                }),
                Err(error) => debug!(?error, "couldn't fetch object via url"),
            }
        }

        for index in search.indices {
            results.extend(
                self.search_backend
                    .search(
                        *index,
                        search.query,
                        search.max_results,
                        search.offset,
                        search.min_id,
                        search.max_id,
                    )
                    .await?,
            );
        }

        Ok(results)
    }
}
