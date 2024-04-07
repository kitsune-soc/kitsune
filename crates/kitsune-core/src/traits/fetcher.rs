use super::Resolver;
use async_trait::async_trait;
use kitsune_db::model::{account::Account, custom_emoji::CustomEmoji, post::Post};
use kitsune_error::Result;
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[derive(Clone, Copy, Debug, TypedBuilder)]
/// Options passed to the fetcher
pub struct AccountFetchOptions<'a> {
    /// Prefetched Webfinger `acct` URI
    #[builder(default, setter(strip_option))]
    pub acct: Option<(&'a str, &'a str)>,

    /// Refetch the account
    ///
    /// This is mainly used to refresh possibly stale actors
    ///
    /// Default: false
    #[builder(default = false)]
    pub refetch: bool,

    /// URL of the account
    pub url: &'a str,
}

impl<'a> From<&'a str> for AccountFetchOptions<'a> {
    fn from(value: &'a str) -> Self {
        Self::builder().url(value).build()
    }
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct PostFetchOptions<'a> {
    /// Call depth of recursive calls of the post fetch logic
    #[builder(default)]
    pub call_depth: u32,

    /// URL of the object
    pub url: &'a str,
}

impl<'a> From<&'a str> for PostFetchOptions<'a> {
    fn from(value: &'a str) -> Self {
        Self::builder().url(value).build()
    }
}

#[async_trait]
pub trait Fetcher: Send + Sync + 'static {
    fn resolver(&self) -> Arc<dyn Resolver>;

    async fn fetch_account(&self, opts: AccountFetchOptions<'_>) -> Result<Option<Account>>;

    async fn fetch_emoji(&self, url: &str) -> Result<Option<CustomEmoji>>;

    async fn fetch_post(&self, opts: PostFetchOptions<'_>) -> Result<Option<Post>>;
}

#[async_trait]
impl Fetcher for Arc<dyn Fetcher> {
    fn resolver(&self) -> Arc<dyn Resolver> {
        (**self).resolver()
    }

    async fn fetch_account(&self, opts: AccountFetchOptions<'_>) -> Result<Option<Account>> {
        (**self).fetch_account(opts).await
    }

    async fn fetch_emoji(&self, url: &str) -> Result<Option<CustomEmoji>> {
        (**self).fetch_emoji(url).await
    }

    async fn fetch_post(&self, opts: PostFetchOptions<'_>) -> Result<Option<Post>> {
        (**self).fetch_post(opts).await
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

    async fn fetch_account(&self, opts: AccountFetchOptions<'_>) -> Result<Option<Account>> {
        for fetcher in self {
            if let Some(account) = fetcher.fetch_account(opts).await? {
                return Ok(Some(account));
            }
        }

        Ok(None)
    }

    async fn fetch_emoji(&self, url: &str) -> Result<Option<CustomEmoji>> {
        for fetcher in self {
            if let Some(emoji) = fetcher.fetch_emoji(url).await? {
                return Ok(Some(emoji));
            }
        }

        Ok(None)
    }

    async fn fetch_post(&self, opts: PostFetchOptions<'_>) -> Result<Option<Post>> {
        for fetcher in self {
            if let Some(post) = fetcher.fetch_post(opts).await? {
                return Ok(Some(post));
            }
        }

        Ok(None)
    }
}
