use super::Storage;
use crate::{
    error::{Error, Result},
    state::LoginState,
};
use moka::sync::Cache;

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

impl Storage for InMemory {
    async fn get_and_remove(&self, key: &str) -> Result<LoginState> {
        self.inner.remove(key).ok_or(Error::MissingLoginState)
    }

    async fn set(&self, key: &str, value: LoginState) -> Result<()> {
        self.inner.insert(key.to_string(), value);
        Ok(())
    }
}
