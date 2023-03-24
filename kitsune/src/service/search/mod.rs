use crate::error::Result;
use async_trait::async_trait;
use kitsune_db::entity::{accounts, posts};
use std::{ops::Deref, sync::Arc};
use uuid::Uuid;

mod grpc;
mod sql;

pub use self::grpc::GrpcSearchService;
pub use self::sql::SqlSearchService;

pub type ArcSearchService = Arc<dyn SearchService>;

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

#[derive(Clone, Copy)]
pub enum SearchIndex {
    Account,
    Post,
}

#[derive(Clone, Copy)]
pub struct SearchResult {
    pub id: Uuid,
}

#[async_trait]
pub trait SearchService: Send + Sync {
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
