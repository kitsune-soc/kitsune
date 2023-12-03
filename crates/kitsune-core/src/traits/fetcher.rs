use super::Resolver;
use crate::error::BoxError;
use async_trait::async_trait;
use kitsune_db::model::{account::Account, custom_emoji::CustomEmoji, post::Post};
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[derive(Clone, Copy, Debug, TypedBuilder)]
/// Options passed to the fetcher
pub struct AccountFetchOptions<'a> {
    /// Prefetched WebFinger `acct` URI
    #[builder(default, setter(strip_option))]
    pub acct: Option<(&'a str, &'a str)>,

    /// Refetch the ActivityPub entity
    ///
    /// This is mainly used to refresh possibly stale actors
    ///
    /// Default: false
    #[builder(default = false)]
    pub refetch: bool,

    /// URL of the ActivityPub entity
    pub url: &'a str,
}

impl<'a> From<&'a str> for AccountFetchOptions<'a> {
    fn from(value: &'a str) -> Self {
        Self::builder().url(value).build()
    }
}

#[async_trait]
pub trait Fetcher: Send + Sync + 'static {
    fn resolver(&self) -> Arc<dyn Resolver>;

    async fn fetch_account(
        &self,
        opts: AccountFetchOptions<'_>,
    ) -> Result<Option<Account>, BoxError>;

    async fn fetch_emoji(&self, url: &str) -> Result<Option<CustomEmoji>, BoxError>;

    async fn fetch_post(&self, url: &str) -> Result<Option<Post>, BoxError>;
}

#[async_trait]
impl Fetcher for Arc<dyn Fetcher> {
    fn resolver(&self) -> Arc<dyn Resolver> {
        (**self).resolver()
    }

    async fn fetch_account(
        &self,
        opts: AccountFetchOptions<'_>,
    ) -> Result<Option<Account>, BoxError> {
        (**self).fetch_account(opts).await
    }

    async fn fetch_emoji(&self, url: &str) -> Result<Option<CustomEmoji>, BoxError> {
        (**self).fetch_emoji(url).await
    }

    async fn fetch_post(&self, url: &str) -> Result<Option<Post>, BoxError> {
        (**self).fetch_post(url).await
    }
}

#[async_trait]
impl<T> Fetcher for Vec<T>
where
    T: Fetcher,
{
    fn resolver(&self) -> Arc<dyn Resolver> {
        Arc::new(self.iter().map(Fetcher::resolver).collect::<Vec<_>>())
    }

    async fn fetch_account(
        &self,
        opts: AccountFetchOptions<'_>,
    ) -> Result<Option<Account>, BoxError> {
        for fetcher in self {
            if let Some(account) = fetcher.fetch_account(opts).await? {
                return Ok(Some(account));
            }
        }

        Ok(None)
    }

    async fn fetch_emoji(&self, url: &str) -> Result<Option<CustomEmoji>, BoxError> {
        for fetcher in self {
            if let Some(emoji) = fetcher.fetch_emoji(url).await? {
                return Ok(Some(emoji));
            }
        }

        Ok(None)
    }

    async fn fetch_post(&self, url: &str) -> Result<Option<Post>, BoxError> {
        for fetcher in self {
            if let Some(post) = fetcher.fetch_post(url).await? {
                return Ok(Some(post));
            }
        }

        Ok(None)
    }
}
