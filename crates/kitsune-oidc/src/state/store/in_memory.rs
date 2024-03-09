use super::Store;
use crate::{
    error::{Error, Result},
    state::LoginState,
};
use moka::future::Cache;

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
        self.inner.remove(key).await.ok_or(Error::MissingLoginState)
    }

    async fn set(&self, key: &str, value: LoginState) -> Result<()> {
        self.inner.insert(key.to_string(), value).await;
        Ok(())
    }
}
