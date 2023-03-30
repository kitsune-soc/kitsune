use crate::error::SearchError;
use async_trait::async_trait;
use kitsune_db::entity::{accounts, posts};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, sync::Arc};
use strum::EnumIter;
use uuid::Uuid;

mod grpc;
mod meilisearch;
mod sql;

pub use self::{grpc::GrpcSearchService, meilisearch::MeiliSearchService, sql::SqlSearchService};

pub type ArcSearchService = Arc<dyn SearchService>;

type Result<T, E = SearchError> = std::result::Result<T, E>;

#[derive(Deserialize, Serialize)]
pub struct Account {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub username: String,
    pub note: Option<String>,
    /// Timestamp of the creation expressed in seconds since the Unix epoch
    pub created_at: u64,
}

impl From<accounts::Model> for Account {
    fn from(value: accounts::Model) -> Self {
        let (created_at_secs, _) = value.id.get_timestamp().unwrap().to_unix();
        Self {
            id: value.id,
            display_name: value.display_name,
            username: value.username,
            note: value.note,
            created_at: created_at_secs,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Post {
    pub id: Uuid,
    pub subject: Option<String>,
    pub content: String,
    /// Timestamp of the creation expressed in seconds since the Unix epoch
    pub created_at: u64,
}

impl From<posts::Model> for Post {
    fn from(value: posts::Model) -> Self {
        let (created_at_secs, _) = value.id.get_timestamp().unwrap().to_unix();
        Self {
            id: value.id,
            subject: value.subject,
            content: value.content,
            created_at: created_at_secs,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum SearchItem {
    Account(Account),
    Post(Post),
}

impl SearchItem {
    #[must_use]
    pub fn index(&self) -> SearchIndex {
        match self {
            Self::Account(..) => SearchIndex::Account,
            Self::Post(..) => SearchIndex::Post,
        }
    }
}

impl From<accounts::Model> for SearchItem {
    fn from(account: accounts::Model) -> Self {
        Self::Account(account.into())
    }
}

impl From<posts::Model> for SearchItem {
    fn from(post: posts::Model) -> Self {
        Self::Post(post.into())
    }
}

#[derive(Clone, Copy, EnumIter)]
pub enum SearchIndex {
    Account,
    Post,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
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
