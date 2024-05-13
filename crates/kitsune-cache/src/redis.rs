use super::CacheBackend;
use fred::{clients::RedisPool, interfaces::KeysInterface, types::Expiration};
use kitsune_error::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Display, marker::PhantomData, time::Duration};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Redis<K, V>
where
    K: ?Sized,
{
    #[builder(default = "DEFAULT-REDIS-CACHER".into())]
    namespace: String,
    #[builder(setter(into))]
    prefix: String,
    redis_conn: RedisPool,
    ttl: Duration,

    // Type phantom data
    #[builder(default, setter(skip))]
    _key: PhantomData<K>,
    #[builder(default, setter(skip))]
    _value: PhantomData<V>,
}

impl<K, V> Redis<K, V>
where
    K: ?Sized,
{
    pub fn new<P>(redis_conn: RedisPool, prefix: P, ttl: Duration) -> Self
    where
        P: Into<String>,
    {
        Self::builder()
            .redis_conn(redis_conn)
            .prefix(prefix)
            .ttl(ttl)
            .build()
    }

    fn compute_key(&self, key: impl Display) -> String {
        format!("{}:{}:{key}", self.namespace, self.prefix)
    }
}

impl<K, V> CacheBackend<K, V> for Redis<K, V>
where
    K: Display + Send + Sync + ?Sized,
    V: Serialize + DeserializeOwned + Send + Sync,
{
    #[instrument(skip_all, fields(%key))]
    async fn delete(&self, key: &K) -> Result<()> {
        let key = self.compute_key(key);

        debug!(%key, "Deleting cache entry");
        self.redis_conn.del(key).await?;

        Ok(())
    }

    #[instrument(skip_all, fields(%key))]
    async fn get(&self, key: &K) -> Result<Option<V>> {
        let key = self.compute_key(key);

        debug!(%key, "Fetching cache entry");
        if let Some(serialised) = self.redis_conn.get::<Option<String>, _>(&key).await? {
            let mut serialised_bytes = serialised.into_bytes();
            let deserialised = simd_json::from_slice(&mut serialised_bytes)?;
            Ok(Some(deserialised))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip_all, fields(%key))]
    async fn set(&self, key: &K, value: &V) -> Result<()> {
        let key = self.compute_key(key);
        let serialised = simd_json::to_string(value)?;

        debug!(%key, ttl = ?self.ttl, "Setting cache entry");
        self.redis_conn
            .set(
                key,
                serialised,
                Some(Expiration::EX(self.ttl.as_secs() as i64)),
                None,
                false,
            )
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
