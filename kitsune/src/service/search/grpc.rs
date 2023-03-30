use super::{Result, SearchIndex, SearchItem, SearchResult, SearchService};
use async_trait::async_trait;
use futures_util::stream;
use kitsune_search_proto::{
    common::SearchIndex as GrpcSearchIndex,
    index::{
        add_index_request::IndexEntity, index_client::IndexClient, AddAccountIndex,
        AddIndexRequest, AddPostIndex, RemoveIndexRequest, ResetRequest,
    },
    search::{search_client::SearchClient, SearchRequest, SearchResult as GrpcSearchResult},
};
use std::future;
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;

/// Search service
///
/// Connects to the `kitsune-search` backend via gRPC
#[derive(Clone)]
pub struct GrpcSearchService {
    searcher: SearchClient<Channel>,
    indexer: IndexClient<Channel>,
}

impl GrpcSearchService {
    pub async fn connect(index_endpoint: &str, search_endpoints: &[String]) -> Result<Self> {
        let index_channel = Endpoint::from_shared(index_endpoint.to_string())?
            .connect()
            .await?;

        let search_endpoints: Vec<Endpoint> = search_endpoints
            .iter()
            .cloned()
            .map(Endpoint::from_shared)
            .collect::<Result<_, tonic::transport::Error>>()?;
        let search_channel = Channel::balance_list(search_endpoints.into_iter());

        Ok(Self {
            searcher: SearchClient::new(search_channel),
            indexer: IndexClient::new(index_channel),
        })
    }
}

#[async_trait]
impl SearchService for GrpcSearchService {
    #[instrument(skip_all)]
    async fn add_to_index(&self, item: SearchItem) -> Result<()> {
        let request = match item {
            SearchItem::Account(account) => AddIndexRequest {
                index_entity: Some(IndexEntity::Account(AddAccountIndex {
                    id: account.id.as_bytes().to_vec(),
                    display_name: account.display_name,
                    username: account.username,
                    description: account.note,
                })),
            },
            SearchItem::Post(post) => AddIndexRequest {
                index_entity: Some(IndexEntity::Post(AddPostIndex {
                    id: post.id.as_bytes().to_vec(),
                    subject: post.subject,
                    content: post.content,
                })),
            },
        };

        self.indexer
            .clone()
            .add(stream::once(future::ready(request)))
            .await?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn remove_from_index(&self, item: SearchItem) -> Result<()> {
        let request = match item {
            SearchItem::Account(account) => RemoveIndexRequest {
                index: GrpcSearchIndex::from(SearchIndex::Account).into(),
                id: account.id.as_bytes().to_vec(),
            },
            SearchItem::Post(post) => RemoveIndexRequest {
                index: GrpcSearchIndex::from(SearchIndex::Post).into(),
                id: post.id.as_bytes().to_vec(),
            },
        };

        self.indexer
            .clone()
            .remove(stream::once(future::ready(request)))
            .await?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn reset_index(&self, index: SearchIndex) -> Result<()> {
        let request = ResetRequest {
            index: GrpcSearchIndex::from(index).into(),
        };
        self.indexer.clone().reset(request).await?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn search(
        &self,
        index: SearchIndex,
        query: String,
        max_results: u64,
        offset: u64,
        min_id: Option<Uuid>,
        max_id: Option<Uuid>,
    ) -> Result<Vec<SearchResult>> {
        let request = SearchRequest {
            index: GrpcSearchIndex::from(index).into(),
            query,
            max_results,
            offset,
            max_id: min_id.as_ref().map(|id| id.as_bytes().to_vec()),
            min_id: max_id.as_ref().map(|id| id.as_bytes().to_vec()),
        };

        let results = self
            .searcher
            .clone()
            .search(request)
            .await?
            .into_inner()
            .results
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(results)
    }
}

impl From<SearchIndex> for GrpcSearchIndex {
    fn from(value: SearchIndex) -> Self {
        match value {
            SearchIndex::Account => GrpcSearchIndex::Account,
            SearchIndex::Post => GrpcSearchIndex::Post,
        }
    }
}

impl From<GrpcSearchResult> for SearchResult {
    fn from(value: GrpcSearchResult) -> Self {
        let id = Uuid::from_bytes(
            value
                .id
                .try_into()
                .expect("Received non-UUID from search service"),
        );

        Self { id }
    }
}
