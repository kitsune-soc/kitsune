#[macro_use]
extern crate tracing;

use enum_dispatch::enum_dispatch;
use kitsune_db::model::{account::Account as DbAccount, post::Post as DbPost};
use kitsune_error::Result;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;
use strum::{AsRefStr, EnumIter};

#[cfg(feature = "meilisearch")]
mod meilisearch;
mod sql;

#[cfg(feature = "meilisearch")]
pub use self::meilisearch::MeiliSearchService;
pub use self::sql::SearchService as SqlSearchService;

#[derive(Clone)]
#[enum_dispatch(SearchBackend)]
pub enum AnySearchBackend {
    #[cfg(feature = "meilisearch")]
    Meilisearch(MeiliSearchService),
    Noop(NoopSearchService),
    Sql(SqlSearchService),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(AsRefStr, Clone, Copy, Debug, EnumIter, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[strum(serialize_all = "snake_case")]
pub enum SearchIndex {
    Account,
    Post,
}

#[derive(Clone, Copy)]
pub struct SearchResultReference {
    pub index: SearchIndex,
    pub id: Uuid,
}

#[enum_dispatch]
#[allow(async_fn_in_trait)] // Because of `enum_dispatch`
pub trait SearchBackend: Send + Sync {
    /// Add an item to the index
    async fn add_to_index(&self, item: SearchItem) -> Result<()>;

    /// Remove an item from the index
    async fn remove_from_index(&self, item: &SearchItem) -> Result<()>;

    /// Reset a search index
    ///
    /// **WARNING**: This is a major destructive operation
    async fn reset_index(&self, index: SearchIndex) -> Result<()>;

    /// Search through a search index
    async fn search(
        &self,
        index: SearchIndex,
        query: &str,
        max_results: u64,
        offset: u64,
        min_id: Option<Uuid>,
        max_id: Option<Uuid>,
    ) -> Result<Vec<SearchResultReference>>;

    /// Update an entity inside the search index
    ///
    /// This is a separate function even though most search engines have immutable documents (meaning updating documents is actually: delete -> create).
    /// That's due to the fact that there are in fact *some* search engines that support updating documents.
    async fn update_in_index(&self, item: SearchItem) -> Result<()>;
}

/// Dummy search service
///
/// Always returns `Ok(())`/an empty list
#[derive(Clone)]
pub struct NoopSearchService;

impl SearchBackend for NoopSearchService {
    async fn add_to_index(&self, _item: SearchItem) -> Result<()> {
        Ok(())
    }

    async fn remove_from_index(&self, _item: &SearchItem) -> Result<()> {
        Ok(())
    }

    async fn reset_index(&self, _index: SearchIndex) -> Result<()> {
        Ok(())
    }

    async fn search(
        &self,
        _index: SearchIndex,
        _query: &str,
        _max_results: u64,
        _offset: u64,
        _min_id: Option<Uuid>,
        _max_id: Option<Uuid>,
    ) -> Result<Vec<SearchResultReference>> {
        Ok(Vec::new())
    }

    async fn update_in_index(&self, _item: SearchItem) -> Result<()> {
        Ok(())
    }
}
