use crate::error::Result;
use async_trait::async_trait;
use futures_util::stream;
use kitsune_db::entity::{accounts, posts};
use kitsune_search_proto::{
    common::SearchIndex,
    index::{
        add_index_request::IndexEntity, index_client::IndexClient, AddAccountIndex,
        AddIndexRequest, AddPostIndex, RemoveIndexRequest, ResetRequest,
    },
    search::{search_client::SearchClient, SearchRequest, SearchResult},
};
use std::{future, ops::Deref, sync::Arc};
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;

pub enum SearchItem {
    Account(accounts::Model),
    Post(posts::Model),
}

impl From<accounts::Model> for SearchItem {
    fn from(account: accounts::Model) -> Self {
        Self::Account(account)
    }
}

impl From<posts::Model> for SearchItem {
    fn from(post: posts::Model) -> Self {
        Self::Post(post)
    }
}

#[async_trait]
pub trait SearchService {
    /// Add an item to the index
    async fn add_to_index(&self, item: SearchItem) -> Result<()>;

    /// Remove an item from the index
    async fn remove_from_index(&self, item: SearchItem) -> Result<()>;

    /// Reset a search index
    ///
    /// **WARNING**: This is a major destructive operation
    async fn reset_index(&self, index: SearchIndex) -> Result<()>;

    /// Search through a search index
    async fn search(
        &self,
        index: SearchIndex,
        query: String,
        max_results: u64,
        offset: u64,
        min_id: Option<Uuid>,
        max_id: Option<Uuid>,
    ) -> Result<Vec<SearchResult>>;
}

#[async_trait]
impl SearchService for Arc<dyn SearchService + Send + Sync> {
    async fn add_to_index(&self, item: SearchItem) -> Result<()> {
        self.deref().add_to_index(item).await
    }

    async fn remove_from_index(&self, item: SearchItem) -> Result<()> {
        self.deref().remove_from_index(item).await
    }

    async fn reset_index(&self, index: SearchIndex) -> Result<()> {
        self.deref().reset_index(index).await
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
        self.deref()
            .search(index, query, max_results, offset, min_id, max_id)
            .await
    }
}

/// Search service
///
/// Connects to the `kitsune-search` backend via gRPC
#[derive(Clone)]
pub struct GrpcSearchService {
    searcher: SearchClient<Channel>,
    indexer: IndexClient<Channel>,
}

impl GrpcSearchService {
    pub async fn new(index_endpoint: &str, search_endpoints: &[String]) -> Result<Self> {
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
                index: SearchIndex::Account.into(),
                id: account.id.as_bytes().to_vec(),
            },
            SearchItem::Post(post) => RemoveIndexRequest {
                index: SearchIndex::Post.into(),
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
            index: index.into(),
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
            index: index.into(),
            query,
            max_results,
            offset,
            max_id: min_id.as_ref().map(|id| id.as_bytes().to_vec()),
            min_id: max_id.as_ref().map(|id| id.as_bytes().to_vec()),
        };

        Ok(self
            .searcher
            .clone()
            .search(request)
            .await?
            .into_inner()
            .results)
    }
}

/// Dummy search service
///
/// Always returns `Ok(())`/an empty list
#[derive(Clone)]
pub struct NoopSearchService;

#[async_trait]
impl SearchService for NoopSearchService {
    async fn add_to_index(&self, _item: SearchItem) -> Result<()> {
        Ok(())
    }

    async fn remove_from_index(&self, _item: SearchItem) -> Result<()> {
        Ok(())
    }

    async fn reset_index(&self, _index: SearchIndex) -> Result<()> {
        Ok(())
    }

    async fn search(
        &self,
        _index: SearchIndex,
        _query: String,
        _max_results: u64,
        _offset: u64,
        _min_id: Option<Uuid>,
        _max_id: Option<Uuid>,
    ) -> Result<Vec<SearchResult>> {
        Ok(Vec::new())
    }
}
