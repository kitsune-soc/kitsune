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
    ) -> impl Future<Output = Result<Account, Self::Error>> + Send;

    fn fetch_emoji(
        &self,
        url: &str,
    ) -> impl Future<Output = Result<CustomEmoji, Self::Error>> + Send;

    fn fetch_post(&self, url: &str) -> impl Future<Output = Result<Post, Self::Error>> + Send;
}

// TODO: How can we implement this trait for an array?
// Probably by changing the return type to a `Result<Option>` and then just fatally failing on actual errors
// Would make the most sense.
