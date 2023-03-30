use super::{Result, SearchIndex, SearchItem, SearchResult, SearchService};
use async_trait::async_trait;
use meilisearch_sdk::{indexes::Index, settings::Settings, Client};
use uuid::Uuid;

pub struct MeiliSearchService {
    client: Client,
}

impl MeiliSearchService {
    pub async fn new(host: &str, api_key: &str) -> Result<Self> {
        let client = Client::new(host, api_key);

        client
            .create_index("accounts", Some("id"))
            .await?
            .wait_for_completion(&client, None, None)
            .await?;

        client
            .create_index("posts", Some("id"))
            .await?
            .wait_for_completion(&client, None, None)
            .await?;

        Ok(Self { client })
    }

    async fn get_index(&self, index: SearchIndex) -> Result<Index> {
        let settings = Settings::new()
            .with_filterable_attributes(["created_at"])
            .with_sortable_attributes(["id"]);

        let index = match index {
            SearchIndex::Account => self.client.index("accounts"),
            SearchIndex::Post => self.client.index("posts"),
        };
        index
            .set_settings(&settings)
            .await?
            .wait_for_completion(&self.client, None, None)
            .await?;

        Ok(index)
    }
}

#[async_trait]
impl SearchService for MeiliSearchService {
    async fn add_to_index(&self, item: SearchItem) -> Result<()> {
        self.get_index(item.index())
            .await?
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
                    .await?
                    .delete_document(account.id)
                    .await?
            }
            SearchItem::Post(post) => {
                self.get_index(SearchIndex::Post)
                    .await?
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
            .await?
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
        let mut filter = String::new();
        if let Some(min_id) = min_id {
            let (created_at_secs, _) = min_id.get_timestamp().unwrap().to_unix();
            filter.push_str("created_at > ");
            filter.push_str(&created_at_secs.to_string());
        }

        if let Some(max_id) = max_id {
            let (created_at_secs, _) = max_id.get_timestamp().unwrap().to_unix();
            if !filter.is_empty() {
                filter.push_str(" AND");
            }

            filter.push_str("created_at < ");
            filter.push_str(&created_at_secs.to_string());
        }

        #[allow(clippy::cast_possible_truncation)]
        let results = self
            .get_index(index)
            .await?
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
