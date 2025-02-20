use super::CacheBackend;
use fred::{clients::Pool, interfaces::KeysInterface, types::Expiration};
use kitsune_error::Result;
use serde::{Serialize, de::DeserializeOwned};
use std::{fmt::Display, marker::PhantomData, time::Duration};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Redis<K, V>
where
    K: ?Sized,
{
    conn_pool: Pool,
    #[builder(default = "DEFAULT-REDIS-CACHER".into())]
    namespace: String,
    #[builder(setter(into))]
    prefix: String,
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
    pub fn new<P>(conn_pool: Pool, prefix: P, ttl: Duration) -> Self
    where
        P: Into<String>,
    {
        Self::builder()
            .conn_pool(conn_pool)
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
        let () = self.conn_pool.del(key).await?;

        Ok(())
    }

    #[instrument(skip_all, fields(%key))]
    async fn get(&self, key: &K) -> Result<Option<V>> {
        let key = self.compute_key(key);

        debug!(%key, "Fetching cache entry");
        match self.conn_pool.get::<Option<String>, _>(&key).await? {
            Some(serialised) => {
                let deserialised = sonic_rs::from_slice(serialised.as_bytes())?;
                Ok(Some(deserialised))
            }
            _ => Ok(None),
        }
    }

    #[instrument(skip_all, fields(%key))]
    async fn set(&self, key: &K, value: &V) -> Result<()> {
        let key = self.compute_key(key);
        let serialised = sonic_rs::to_string(value)?;

        debug!(%key, ttl = ?self.ttl, "Setting cache entry");
        let () = self
            .conn_pool
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
            conn_pool: self.conn_pool.clone(),
            ttl: self.ttl,
            _key: PhantomData,
            _value: PhantomData,
        }
    }
}
