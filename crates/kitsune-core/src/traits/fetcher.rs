use crate::error::BoxError;
use kitsune_db::model::{account::Account, custom_emoji::CustomEmoji, post::Post};
use std::future::Future;
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

pub trait Fetcher: Send + Sync + 'static {
    type Error: Into<BoxError>;

    fn fetch_account(
        &self,
        opts: AccountFetchOptions<'_>,
    ) -> impl Future<Output = Result<Option<Account>, Self::Error>> + Send;

    fn fetch_emoji(
        &self,
        url: &str,
    ) -> impl Future<Output = Result<Option<CustomEmoji>, Self::Error>> + Send;

    fn fetch_post(
        &self,
        url: &str,
    ) -> impl Future<Output = Result<Option<Post>, Self::Error>> + Send;
}

impl<T> Fetcher for [T]
where
    T: Fetcher,
{
    type Error = BoxError;

    async fn fetch_account(
        &self,
        opts: AccountFetchOptions<'_>,
    ) -> Result<Option<Account>, Self::Error> {
        for fetcher in self {
            if let Some(account) = fetcher.fetch_account(opts).await.map_err(Into::into)? {
                return Ok(Some(account));
            }
        }

        Ok(None)
    }

    async fn fetch_emoji(&self, url: &str) -> Result<Option<CustomEmoji>, Self::Error> {
        for fetcher in self {
            if let Some(emoji) = fetcher.fetch_emoji(url).await.map_err(Into::into)? {
                return Ok(Some(emoji));
            }
        }

        Ok(None)
    }

    async fn fetch_post(&self, url: &str) -> Result<Option<Post>, Self::Error> {
        for fetcher in self {
            if let Some(post) = fetcher.fetch_post(url).await.map_err(Into::into)? {
                return Ok(Some(post));
            }
        }

        Ok(None)
    }
}
