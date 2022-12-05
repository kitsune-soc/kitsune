use deadpool_redis::Connection;
use redis::{AsyncCommands, ErrorKind, FromRedisValue, RedisError, RedisResult, ToRedisArgs};
use std::time::Duration;

pub struct Cacher {
    prefix: String,
    redis_conn: deadpool_redis::Pool,
    ttl: Duration,
}

// TODO: Make `get`/`set` generic over serde's traits to allow for more complex data to be cached
impl Cacher {
    pub fn new(redis_conn: deadpool_redis::Pool, prefix: String, ttl: Duration) -> Self {
        Self {
            prefix,
            redis_conn,
            ttl,
        }
    }

    fn compute_key(&self, key: &str) -> String {
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

    pub async fn delete(&self, key: &str) -> RedisResult<()> {
        let mut conn = self.get_connection().await?;
        conn.del(self.compute_key(key)).await?;
        Ok(())
    }

    pub async fn get<V>(&self, key: &str) -> RedisResult<Option<V>>
    where
        V: FromRedisValue,
    {
        let mut conn = self.get_connection().await?;
        conn.get(self.compute_key(key)).await
    }

    pub async fn set<V>(&self, key: &str, value: V) -> RedisResult<()>
    where
        V: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.get_connection().await?;
        #[allow(clippy::cast_possible_truncation)]
        conn.set_ex(self.compute_key(key), value, self.ttl.as_secs() as usize)
            .await?;
        Ok(())
    }
}
