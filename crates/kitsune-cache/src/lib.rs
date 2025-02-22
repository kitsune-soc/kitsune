#[macro_use]
extern crate tracing;

use enum_dispatch::enum_dispatch;
use kitsune_error::Result;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Display;
use triomphe::Arc;

pub use self::in_memory::InMemory as InMemoryCache;
pub use self::redis::Redis as RedisCache;

mod in_memory;
mod redis;

pub type ArcCache<K, V> = Arc<AnyCache<K, V>>;

#[enum_dispatch(CacheBackend<K, V>)]
pub enum AnyCache<K, V>
where
    K: Display + Send + Sync + ?Sized,
    V: Clone + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    InMemory(InMemoryCache<K, V>),
    Noop(NoopCache),
    Redis(RedisCache<K, V>),
}

#[enum_dispatch]
#[allow(async_fn_in_trait)] // Because of `enum_dispatch`
pub trait CacheBackend<K, V>: Send + Sync
where
    K: ?Sized,
{
    async fn delete(&self, key: &K) -> Result<()>;
    async fn get(&self, key: &K) -> Result<Option<V>>;
    async fn set(&self, key: &K, value: &V) -> Result<()>;
}

#[derive(Clone)]
pub struct NoopCache;

impl<K, V> CacheBackend<K, V> for NoopCache
where
    K: Send + Sync + ?Sized,
    V: Send + Sync,
{
    async fn delete(&self, _key: &K) -> Result<()> {
        Ok(())
    }

    async fn get(&self, _key: &K) -> Result<Option<V>> {
        Ok(None)
    }

    async fn set(&self, _key: &K, _value: &V) -> Result<()> {
        Ok(())
    }
}
