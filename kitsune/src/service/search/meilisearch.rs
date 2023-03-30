use super::{Result, SearchIndex, SearchItem, SearchResult, SearchService};
use async_trait::async_trait;
use meilisearch_sdk::{indexes::Index, Client};
use uuid::Uuid;

pub struct MeiliSearchService {
    client: Client,
}

impl MeiliSearchService {
    #[must_use]
    pub fn new(host: &str, api_key: &str) -> Self {
        Self {
            client: Client::new(host, api_key),
        }
    }

    fn get_index(&self, index: SearchIndex) -> Index {
        match index {
            SearchIndex::Account => self.client.index("accounts"),
            SearchIndex::Post => self.client.index("posts"),
        }
    }
}

#[async_trait]
impl SearchService for MeiliSearchService {
    async fn add_to_index(&self, item: SearchItem) -> Result<()> {
        self.get_index(item.index())
            .add_documents(&[item], Some("id"))
            .await?
            .wait_for_completion(&self.client, None, None)
            .await?;

        Ok(())
    }

    async fn remove_from_index(&self, item: SearchItem) -> Result<()> {
        match item {
            SearchItem::Account(account) => {
                self.get_index(SearchIndex::Account)
                    .delete_document(account.id)
                    .await?
            }
            SearchItem::Post(post) => {
                self.get_index(SearchIndex::Post)
                    .delete_document(post.id)
                    .await?
            }
        }
        .wait_for_completion(&self.client, None, None)
        .await?;

        Ok(())
    }

    async fn reset_index(&self, index: SearchIndex) -> Result<()> {
        self.get_index(index)
            .delete_all_documents()
            .await?
            .wait_for_completion(&self.client, None, None)
            .await?;

        Ok(())
    }

    async fn search(
        &self,
        index: SearchIndex,
        query: String,
        max_results: u64,
        offset: u64,
        min_id: Option<Uuid>,
        max_id: Option<Uuid>,
    ) -> Result<Vec<SearchResult>> {
        let filter = format!(
            "id > {} AND id < {}",
            min_id.unwrap_or_else(Uuid::nil),
            max_id.unwrap_or_else(Uuid::max)
        );

        #[allow(clippy::cast_possible_truncation)]
        let results = self
            .get_index(index)
            .search()
            .with_query(&query)
            .with_filter(&filter)
            .with_sort(&["id:desc"])
            .with_offset(offset as usize)
            .with_limit(max_results as usize)
            .execute::<SearchResult>()
            .await?;

        Ok(results.hits.into_iter().map(|item| item.result).collect())
    }
}
