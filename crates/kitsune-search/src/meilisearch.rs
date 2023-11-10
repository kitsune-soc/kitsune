use super::{Result, SearchBackend, SearchIndex, SearchItem, SearchResultReference};
use meilisearch_sdk::{indexes::Index, settings::Settings, Client};
use serde::Deserialize;
use speedy_uuid::Uuid;
use strum::IntoEnumIterator;

#[derive(Deserialize)]
struct MeilisearchResult {
    id: Uuid,
}

#[derive(Clone)]
pub struct MeiliSearchService {
    client: Client,
}

impl MeiliSearchService {
    /// Connect to the Meilisearch instance and initialise the indices
    ///
    /// # Errors
    ///
    /// - Failed to connect to the instance
    #[allow(clippy::missing_panics_doc)]
    pub async fn new(host: &str, api_key: &str) -> Result<Self> {
        let service = Self {
            client: Client::new(host, Some(api_key)),
        };
        let settings = Settings::new()
            .with_filterable_attributes(["created_at"])
            .with_sortable_attributes(["id"]);

        for index in SearchIndex::iter() {
            service
                .get_index(index)
                .set_settings(&settings)
                .await?
                .wait_for_completion(&service.client, None, None)
                .await?;
        }

        Ok(service)
    }

    fn get_index(&self, index: SearchIndex) -> Index {
        match index {
            SearchIndex::Account => self.client.index("accounts"),
            SearchIndex::Post => self.client.index("posts"),
        }
    }
}

impl SearchBackend for MeiliSearchService {
    #[instrument(skip_all)]
    async fn add_to_index(&self, item: SearchItem) -> Result<()> {
        self.get_index(item.index())
            .add_documents(&[item], Some("id"))
            .await?
            .wait_for_completion(&self.client, None, None)
            .await?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn remove_from_index(&self, item: &SearchItem) -> Result<()> {
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

    #[instrument(skip(self))]
    async fn reset_index(&self, index: SearchIndex) -> Result<()> {
        self.get_index(index)
            .delete_all_documents()
            .await?
            .wait_for_completion(&self.client, None, None)
            .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn search(
        &self,
        index: SearchIndex,
        query: &str,
        max_results: u64,
        offset: u64,
        min_id: Option<Uuid>,
        max_id: Option<Uuid>,
    ) -> Result<Vec<SearchResultReference>> {
        let min_timestamp = min_id.map_or(u64::MIN, |id| {
            let (created_at_secs, _) = id.get_timestamp().unwrap().to_unix();
            created_at_secs
        });
        let max_timestamp = max_id.map_or(u64::MAX, |id| {
            let (created_at_secs, _) = id.get_timestamp().unwrap().to_unix();
            created_at_secs
        });

        let filter = format!("created_at > {min_timestamp} AND created_at < {max_timestamp}");
        #[allow(clippy::cast_possible_truncation)]
        let results = self
            .get_index(index)
            .search()
            .with_query(query)
            .with_filter(&filter)
            .with_sort(&["id:desc"])
            .with_offset(offset as usize)
            .with_limit(max_results as usize)
            .execute::<MeilisearchResult>()
            .await?;

        Ok(results
            .hits
            .into_iter()
            .map(|item| SearchResultReference {
                index,
                id: item.result.id,
            })
            .collect())
    }

    #[instrument(skip_all)]
    async fn update_in_index(&self, item: SearchItem) -> Result<()> {
        self.get_index(item.index())
            .add_or_update(&[item], Some("id"))
            .await?
            .wait_for_completion(&self.client, None, None)
            .await?;

        Ok(())
    }
}
