use crate::error::BoxError;
use serde::{Deserialize, Serialize};
use std::future::Future;

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

pub trait Resolver: Send + Sync + 'static {
    type Error: Into<BoxError>;

    fn resolve_account(
        &self,
        username: &str,
        domain: &str,
    ) -> impl Future<Output = Result<Option<AccountResource>, Self::Error>> + Send;
}
