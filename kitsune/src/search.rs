use crate::{
    db::model::{account, post},
    error::Result,
};
use async_trait::async_trait;
use futures_util::stream;
use kitsune_search_proto::{
    common::SearchIndex,
    index::{
        add_index_request::IndexEntity, index_client::IndexClient, AddAccountIndex,
        AddIndexRequest, AddPostIndex, RemoveIndexRequest, ResetRequest,
    },
    search::{search_client::SearchClient, SearchRequest, SearchResponse},
};
use std::future;
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;

pub enum SearchItem {
    Account(account::Model),
    Post(post::Model),
}

impl From<account::Model> for SearchItem {
    fn from(account: account::Model) -> Self {
        Self::Account(account)
    }
}

impl From<post::Model> for SearchItem {
    fn from(post: post::Model) -> Self {
        Self::Post(post)
    }
}

#[async_trait]
pub trait SearchService: Clone + Send + 'static {
    /// Add an item to the index
    async fn add_to_index<I>(&mut self, item: I) -> Result<()>
    where
        I: Into<SearchItem> + Send;

    /// Remove an item from the index
    async fn remove_from_index<I>(&mut self, item: I) -> Result<()>
    where
        I: Into<SearchItem> + Send;

    /// Reset a search index
    ///
    /// **WARNING**: This is a major destructive operation
    async fn reset_index(&mut self, index: SearchIndex) -> Result<()>;

    /// Search through a search index
    async fn search(
        &mut self,
        index: SearchIndex,
        query: String,
        max_results: u64,
        offset: u64,
        min_id: Option<Uuid>,
        max_id: Option<Uuid>,
    ) -> Result<SearchResponse>;
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
    async fn add_to_index<I>(&mut self, item: I) -> Result<()>
    where
        I: Into<SearchItem> + Send,
    {
        let request = match item.into() {
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
            .add(stream::once(future::ready(request)))
            .await?;

        Ok(())
    }

    async fn remove_from_index<I>(&mut self, item: I) -> Result<()>
    where
        I: Into<SearchItem> + Send,
    {
        let request = match item.into() {
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
            .remove(stream::once(future::ready(request)))
            .await?;

        Ok(())
    }

    async fn reset_index(&mut self, index: SearchIndex) -> Result<()> {
        let request = ResetRequest {
            index: index.into(),
        };
        self.indexer.reset(request).await?;

        Ok(())
    }

    async fn search(
        &mut self,
        index: SearchIndex,
        query: String,
        max_results: u64,
        offset: u64,
        min_id: Option<Uuid>,
        max_id: Option<Uuid>,
    ) -> Result<SearchResponse> {
        let request = SearchRequest {
            index: index.into(),
            query,
            max_results,
            offset,
            max_id: min_id.as_ref().map(|id| id.as_bytes().to_vec()),
            min_id: max_id.as_ref().map(|id| id.as_bytes().to_vec()),
        };

        Ok(self.searcher.search(request).await?.into_inner())
    }
}

/// Dummy search service
///
/// Always returns `Ok(())`/an empty list
#[derive(Clone)]
pub struct NoopSearchService;

#[async_trait]
impl SearchService for NoopSearchService {
    async fn add_to_index<I>(&mut self, _item: I) -> Result<()>
    where
        I: Into<SearchItem> + Send,
    {
        Ok(())
    }

    async fn remove_from_index<I>(&mut self, _item: I) -> Result<()>
    where
        I: Into<SearchItem> + Send,
    {
        Ok(())
    }

    async fn reset_index(&mut self, _index: SearchIndex) -> Result<()> {
        Ok(())
    }

    async fn search(
        &mut self,
        _index: SearchIndex,
        _query: String,
        _max_results: u64,
        _offset: u64,
        _min_id: Option<Uuid>,
        _max_id: Option<Uuid>,
    ) -> Result<SearchResponse> {
        Ok(SearchResponse { result: Vec::new() })
    }
}
