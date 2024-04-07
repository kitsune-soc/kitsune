use async_trait::async_trait;
use kitsune_error::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[async_trait]
pub trait Resolver: Send + Sync + 'static {
    async fn resolve_account(
        &self,
        username: &str,
        domain: &str,
    ) -> Result<Option<AccountResource>>;
}

#[async_trait]
impl Resolver for Arc<dyn Resolver> {
    async fn resolve_account(
        &self,
        username: &str,
        domain: &str,
    ) -> Result<Option<AccountResource>> {
        (**self).resolve_account(username, domain).await
    }
}

#[async_trait]
impl<T> Resolver for Vec<T>
where
    T: Resolver,
{
    async fn resolve_account(
        &self,
        username: &str,
        domain: &str,
    ) -> Result<Option<AccountResource>> {
        for resolver in self {
            if let Some(resource) = resolver.resolve_account(username, domain).await? {
                return Ok(Some(resource));
            }
        }

        Ok(None)
    }
}
