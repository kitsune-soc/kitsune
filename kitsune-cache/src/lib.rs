#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]

#[macro_use]
extern crate tracing;

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Display, sync::Arc};

pub use self::error::Error;
pub use self::in_memory::InMemory as InMemoryCache;
pub use self::redis::Redis as RedisCache;

mod error;
mod in_memory;
mod redis;

type CacheResult<T, E = Error> = Result<T, E>;

pub type ArcCache<K, V> = Arc<Cache<K, V>>;

#[enum_dispatch(CacheBackend<K, V>)]
pub enum Cache<K, V>
where
    K: Display + Send + Sync + ?Sized,
    V: Clone + DeserializeOwned + Serialize + Send + Sync + 'static,
{
    InMemory(InMemoryCache<K, V>),
    Noop(NoopCache),
    Redis(RedisCache<K, V>),
}

#[async_trait]
#[enum_dispatch]
pub trait CacheBackend<K, V>: Send + Sync
where
    K: ?Sized,
{
    async fn delete(&self, key: &K) -> CacheResult<()>;
    async fn get(&self, key: &K) -> CacheResult<Option<V>>;
    async fn set(&self, key: &K, value: &V) -> CacheResult<()>;
}

#[derive(Clone)]
pub struct NoopCache;

#[async_trait]
impl<K, V> CacheBackend<K, V> for NoopCache
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
