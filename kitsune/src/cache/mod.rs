use crate::error::CacheError;
use async_trait::async_trait;
use std::{ops::Deref, sync::Arc};

mod in_memory;
mod redis;

pub use self::in_memory::InMemoryCache;
pub use self::redis::RedisCache;

type CacheResult<T, E = CacheError> = Result<T, E>;

pub type ArcCache<K, V> = Arc<dyn Cache<K, V>>;

#[async_trait]
pub trait Cache<K, V>: Send + Sync
where
    K: ?Sized,
{
    async fn delete(&self, key: &K) -> CacheResult<()>;
    async fn get(&self, key: &K) -> CacheResult<Option<V>>;
    async fn set(&self, key: &K, value: &V) -> CacheResult<()>;
}

#[async_trait]
impl<K, V> Cache<K, V> for Arc<dyn Cache<K, V> + Send + Sync>
where
    K: Send + Sync + ?Sized,
    V: Send + Sync,
{
    async fn delete(&self, key: &K) -> CacheResult<()> {
        self.deref().delete(key).await
    }

    async fn get(&self, key: &K) -> CacheResult<Option<V>> {
        self.deref().get(key).await
    }

    async fn set(&self, key: &K, value: &V) -> CacheResult<()> {
        self.deref().set(key, value).await
    }
}

#[derive(Clone)]
pub struct NoopCache;

#[async_trait]
impl<K, V> Cache<K, V> for NoopCache
where
    K: Send + Sync + ?Sized,
    V: Send + Sync,
{
    async fn delete(&self, _key: &K) -> CacheResult<()> {
        Ok(())
    }

    async fn get(&self, _key: &K) -> CacheResult<Option<V>> {
        Ok(None)
    }

    async fn set(&self, _key: &K, _value: &V) -> CacheResult<()> {
        Ok(())
    }
}
