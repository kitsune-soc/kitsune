use super::Store;
use crate::state::LoginState;
use kitsune_error::{kitsune_error, ErrorType, Result};
use scc::HashCache;
use triomphe::Arc;

#[derive(Clone)]
pub struct InMemory {
    inner: Arc<HashCache<String, LoginState>>,
}

impl InMemory {
    pub fn new(size: usize) -> Self {
        Self {
            inner: Arc::new(HashCache::with_capacity(size / 2, size)),
        }
    }
}

impl Store for InMemory {
    async fn get_and_remove(&self, key: &str) -> Result<LoginState> {
        self.inner
            .remove_async(key)
            .await
            .map(|(_key, value)| value)
            .ok_or_else(|| kitsune_error!(type = ErrorType::BadRequest, "missing login state"))
    }

    async fn set(&self, key: &str, value: LoginState) -> Result<()> {
        let _ = self.inner.put_async(key.to_string(), value).await;
        Ok(())
    }
}
