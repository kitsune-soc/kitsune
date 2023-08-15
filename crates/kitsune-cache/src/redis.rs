use super::{CacheBackend, CacheResult};
use async_trait::async_trait;
use derive_builder::Builder;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Display, marker::PhantomData, time::Duration};

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct Redis<K, V>
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

impl<K, V> Redis<K, V>
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
    pub fn builder() -> RedisBuilder<K, V> {
        RedisBuilder::default()
    }

    fn compute_key(&self, key: impl Display) -> String {
        format!("{}:{}:{key}", self.namespace, self.prefix)
    }
}

#[async_trait]
impl<K, V> CacheBackend<K, V> for Redis<K, V>
where
    K: Display + Send + Sync + ?Sized,
    V: Serialize + DeserializeOwned + Send + Sync,
{
    #[instrument(skip_all, fields(%key))]
    async fn delete(&self, key: &K) -> CacheResult<()> {
        let mut conn = self.redis_conn.get().await?;
        let key = self.compute_key(key);

        debug!(%key, "Deleting cache entry");
        conn.del(key).await?;

        Ok(())
    }

    #[instrument(skip_all, fields(%key))]
    async fn get(&self, key: &K) -> CacheResult<Option<V>> {
        let mut conn = self.redis_conn.get().await?;
        let key = self.compute_key(key);

        debug!(%key, "Fetching cache entry");
        if let Some(serialised) = conn.get::<_, Option<String>>(&key).await? {
            let mut serialised_bytes = serialised.into_bytes();
            let deserialised = simd_json::from_slice(&mut serialised_bytes)?;
            Ok(Some(deserialised))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip_all, fields(%key))]
    async fn set(&self, key: &K, value: &V) -> CacheResult<()> {
        let mut conn = self.redis_conn.get().await?;
        let key = self.compute_key(key);
        let serialised = simd_json::to_string(value)?;

        debug!(%key, ttl = ?self.ttl, "Setting cache entry");
        #[allow(clippy::cast_possible_truncation)]
        conn.set_ex(key, serialised, self.ttl.as_secs() as usize)
            .await?;

        Ok(())
    }
}

impl<K, V> Clone for Redis<K, V>
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
