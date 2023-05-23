use crate::error::SearchError;
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use kitsune_db::model::{account::Account as DbAccount, post::Post as DbPost};
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use uuid::Uuid;

mod grpc;
#[cfg(feature = "meilisearch")]
mod meilisearch;
mod sql;

#[cfg(feature = "meilisearch")]
pub use self::meilisearch::MeiliSearchService;
pub use self::{grpc::GrpcSearchService, sql::SqlSearchService};

type Result<T, E = SearchError> = std::result::Result<T, E>;

#[derive(Clone)]
#[enum_dispatch(SearchBackend)]
pub enum SearchService {
    Grpc(GrpcSearchService),
    #[cfg(feature = "meilisearch")]
    Meilisearch(MeiliSearchService),
    Noop(NoopSearchService),
    Sql(SqlSearchService),
}

#[derive(Deserialize, Serialize)]
pub struct Account {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub username: String,
    pub note: Option<String>,
    /// Timestamp of the creation expressed in seconds since the Unix epoch
    pub created_at: u64,
}

impl From<DbAccount> for Account {
    fn from(value: DbAccount) -> Self {
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

impl From<DbPost> for Post {
    fn from(value: DbPost) -> Self {
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

impl From<DbAccount> for SearchItem {
    fn from(account: DbAccount) -> Self {
        Self::Account(account.into())
    }
}

impl From<DbPost> for SearchItem {
    fn from(post: DbPost) -> Self {
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
#[enum_dispatch]
pub trait SearchBackend: Send + Sync {
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

/// Dummy search service
///
/// Always returns `Ok(())`/an empty list
#[derive(Clone)]
pub struct NoopSearchService;

#[async_trait]
impl SearchBackend for NoopSearchService {
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
