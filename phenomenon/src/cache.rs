use deadpool_redis::Connection;
use redis::{AsyncCommands, ErrorKind, RedisError, RedisResult};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Display, marker::PhantomData, time::Duration};

pub struct Cacher<K, V> {
    prefix: String,
    redis_conn: deadpool_redis::Pool,
    ttl: Duration,

    // Type phantom data
    _key: PhantomData<K>,
    _value: PhantomData<V>,
}

impl<K, V> Cacher<K, V>
where
    K: Display,
    V: Serialize + DeserializeOwned,
{
    pub fn new(redis_conn: deadpool_redis::Pool, prefix: String, ttl: Duration) -> Self {
        Self {
            prefix,
            redis_conn,
            ttl,
            _key: PhantomData,
            _value: PhantomData,
        }
    }

    fn compute_key(&self, key: impl Display) -> String {
        // TODO: Make namespace configurable
        format!("DEFAULT-REDIS-CACHER:{}:{key}", self.prefix)
    }

    async fn get_connection(&self) -> RedisResult<Connection> {
        self.redis_conn.get().await.map_err(|err| {
            RedisError::from((
                ErrorKind::IoError,
                "Failed to obtain connection from pool",
                err.to_string(),
            ))
        })
    }

    pub async fn delete(&self, key: K) -> RedisResult<()> {
        let mut conn = self.get_connection().await?;
        conn.del(self.compute_key(key)).await?;
        Ok(())
    }

    pub async fn get(&self, key: K) -> RedisResult<Option<V>> {
        let mut conn = self.get_connection().await?;
        if let Some(serialised) = conn.get::<_, Option<String>>(self.compute_key(key)).await? {
            let deserialised = serde_json::from_str(&serialised).map_err(|err| {
                RedisError::from((
                    ErrorKind::IoError,
                    "Failed to deserialise data",
                    err.to_string(),
                ))
            })?;
            Ok(Some(deserialised))
        } else {
            Ok(None)
        }
    }

    pub async fn set(&self, key: K, value: &V) -> RedisResult<()> {
        let mut conn = self.get_connection().await?;
        let serialised = serde_json::to_string(value).map_err(|err| {
            RedisError::from((
                ErrorKind::IoError,
                "Failed to serialise data",
                err.to_string(),
            ))
        })?;

        #[allow(clippy::cast_possible_truncation)]
        conn.set_ex(
            self.compute_key(key),
            serialised,
            self.ttl.as_secs() as usize,
        )
        .await?;

        Ok(())
    }
}
