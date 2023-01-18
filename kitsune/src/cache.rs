use crate::error::CacheError;
use async_trait::async_trait;
use derive_builder::Builder;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Display, marker::PhantomData, time::Duration};

type CacheResult<T, E = CacheError> = Result<T, E>;

#[async_trait]
pub trait Cache<K, V>
where
    K: ?Sized,
{
    async fn delete(&self, key: &K) -> CacheResult<()>;
    async fn get(&self, key: &K) -> CacheResult<Option<V>>;
    async fn set(&self, key: &K, value: &V) -> CacheResult<()>;
}

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct RedisCache<K, V>
where
    K: ?Sized,
{
    #[builder(default = "\"DEFAULT-REDIS-CACHER\".into()")]
    namespace: String,
    #[builder(setter(into))]
    prefix: String,
    redis_conn: deadpool_redis::Pool,
    ttl: Duration,

    // Type phantom data
    #[builder(setter(skip))]
    _key: PhantomData<K>,
    #[builder(setter(skip))]
    _value: PhantomData<V>,
}

impl<K, V> RedisCache<K, V>
where
    K: ?Sized,
{
    #[allow(clippy::missing_panics_doc)] // All invariants covered. Won't panic.
    pub fn new<P>(redis_conn: deadpool_redis::Pool, prefix: P, ttl: Duration) -> Self
    where
        P: Into<String>,
    {
        Self::builder()
            .redis_conn(redis_conn)
            .prefix(prefix)
            .ttl(ttl)
            .build()
            .unwrap()
    }

    #[must_use]
    pub fn builder() -> RedisCacheBuilder<K, V> {
        RedisCacheBuilder::default()
    }

    fn compute_key(&self, key: impl Display) -> String {
        format!("{}:{}:{key}", self.namespace, self.prefix)
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for RedisCache<K, V>
where
    K: Display + Send + Sync + ?Sized,
    V: Serialize + DeserializeOwned + Send + Sync,
{
    #[instrument(skip_all)]
    async fn delete(&self, key: &K) -> CacheResult<()> {
        let mut conn = self.redis_conn.get().await?;
        let key = self.compute_key(key);

        debug!(%key, "Deleting cache entry");
        conn.del(key).await?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn get(&self, key: &K) -> CacheResult<Option<V>> {
        let mut conn = self.redis_conn.get().await?;
        let key = self.compute_key(key);

        debug!(%key, "Fetching cache entry");
        if let Some(serialised) = conn.get::<_, Option<String>>(&key).await? {
            let deserialised = serde_json::from_str(&serialised)?;
            Ok(Some(deserialised))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip_all)]
    async fn set(&self, key: &K, value: &V) -> CacheResult<()> {
        let mut conn = self.redis_conn.get().await?;
        let key = self.compute_key(key);
        let serialised = serde_json::to_string(value)?;

        debug!(%key, ttl = ?self.ttl, "Setting cache entry");
        #[allow(clippy::cast_possible_truncation)]
        conn.set_ex(key, serialised, self.ttl.as_secs() as usize)
            .await?;

        Ok(())
    }
}

impl<K, V> Clone for RedisCache<K, V>
where
    K: ?Sized,
{
    fn clone(&self) -> Self {
        Self {
            namespace: self.namespace.clone(),
            prefix: self.prefix.clone(),
            redis_conn: self.redis_conn.clone(),
            ttl: self.ttl,
            _key: PhantomData,
            _value: PhantomData,
        }
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
