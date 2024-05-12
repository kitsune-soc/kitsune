use super::Store;
use crate::state::LoginState;
use kitsune_error::{kitsune_error, ErrorType, Result};
use moka::sync::Cache;

#[derive(Clone)]
pub struct InMemory {
    inner: Cache<String, LoginState>,
}

impl InMemory {
    pub fn new(size: u64) -> Self {
        Self {
            inner: Cache::new(size),
        }
    }
}

impl Store for InMemory {
    async fn get_and_remove(&self, key: &str) -> Result<LoginState> {
        self.inner
            .remove(key)
            .ok_or_else(|| kitsune_error!(type = ErrorType::BadRequest, "missing login state"))
    }

    async fn set(&self, key: &str, value: LoginState) -> Result<()> {
        self.inner.insert(key.to_string(), value);
        Ok(())
    }
}
