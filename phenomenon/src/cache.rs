use crate::error::CacheError;
use derive_builder::Builder;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Display, marker::PhantomData, time::Duration};

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct Cacher<K, V>
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

impl<K, V> Cacher<K, V>
where
    K: Display + ?Sized,
    V: Serialize + DeserializeOwned,
{
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

    pub fn builder() -> CacherBuilder<K, V> {
        CacherBuilder::default()
    }

    fn compute_key(&self, key: impl Display) -> String {
        format!("{}:{}:{key}", self.namespace, self.prefix)
    }

    #[instrument(skip_all)]
    pub async fn delete(&self, key: &K) -> Result<(), CacheError> {
        let mut conn = self.redis_conn.get().await?;
        let key = self.compute_key(key);

        debug!(%key, "Deleting cache entry");
        conn.del(key).await?;

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        let mut conn = self.redis_conn.get().await?;
        let key = self.compute_key(key);

        debug!(%key, "Fetching cache entry");
        if let Some(serialised) = conn.get::<_, Option<String>>(key).await? {
            let deserialised = serde_json::from_str(&serialised)?;
            Ok(Some(deserialised))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip_all)]
    pub async fn set(&self, key: &K, value: &V) -> Result<(), CacheError> {
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

impl<K, V> Clone for Cacher<K, V>
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
