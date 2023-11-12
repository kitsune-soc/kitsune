use kitsune_db::model::{account::Account, custom_emoji::CustomEmoji, post::Post};
use serde::{Deserialize, Serialize};
use std::future::Future;
use typed_builder::TypedBuilder;

#[derive(Clone, Debug, TypedBuilder)]
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

/// Description of a resolved account
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountResource {
    /// The `self` link (the account's URI)
    pub uri: String,
    /// The username part of the canonical `acct:` URI
    pub username: String,
    /// The host component of the canonical `acct:` URI
    pub domain: String,
}

pub trait Fetcher {
    type Error;

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

pub trait Resolver {
    type Error;

    fn resolve_account(
        &self,
        username: &str,
        domain: &str,
    ) -> impl Future<Output = Result<Option<AccountResource>, Self::Error>> + Send;
}
