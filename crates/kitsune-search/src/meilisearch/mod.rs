use self::http_client::HttpClient;
use super::{Result, SearchBackend, SearchIndex, SearchItem, SearchResultReference};
use kitsune_derive::kitsune_service;
use meilisearch_sdk::{client::Client, indexes::Index, settings::Settings};
use serde::Deserialize;
use speedy_uuid::Uuid;
use strum::IntoEnumIterator;

mod http_client;

#[derive(Deserialize)]
struct MeilisearchResult {
    id: Uuid,
}

#[kitsune_service(omit_builder)]
pub struct MeiliSearchService {
    client: Client<HttpClient>,
}

impl MeiliSearchService {
    /// Connect to the Meilisearch instance and initialise the indices
    ///
    /// # Errors
    ///
    /// - Failed to connect to the instance
    pub async fn new(host: &str, api_key: &str) -> Result<Self> {
        let http_client = HttpClient {
            inner: kitsune_http_client::Client::builder()
                .content_length_limit(None)
                .build(),
        };
        let service = Self::from(__MeiliSearchService__Inner {
            client: Client::new_with_client(host, Some(api_key), http_client),
        });

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

    fn get_index(&self, index: SearchIndex) -> Index<HttpClient> {
        self.client.index(index.as_ref())
    }
}

impl SearchBackend for MeiliSearchService {
    #[cfg_attr(not(coverage), instrument(skip_all))]
    async fn add_to_index(&self, item: SearchItem) -> Result<()> {
        self.get_index(item.index())
            .add_documents(&[item], Some("id"))
            .await?
            .wait_for_completion(&self.client, None, None)
            .await?;

        Ok(())
    }

    #[cfg_attr(not(coverage), instrument(skip_all))]
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

    #[cfg_attr(not(coverage), instrument(skip(self)))]
    async fn reset_index(&self, index: SearchIndex) -> Result<()> {
        self.get_index(index)
            .delete_all_documents()
            .await?
            .wait_for_completion(&self.client, None, None)
            .await?;

        Ok(())
    }

    #[cfg_attr(not(coverage), instrument(skip(self)))]
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

    #[cfg_attr(not(coverage), instrument(skip_all))]
    async fn update_in_index(&self, item: SearchItem) -> Result<()> {
        self.get_index(item.index())
            .add_or_update(&[item], Some("id"))
            .await?
            .wait_for_completion(&self.client, None, None)
            .await?;

        Ok(())
    }
}
