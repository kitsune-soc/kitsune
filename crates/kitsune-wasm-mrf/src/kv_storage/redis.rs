use super::BoxError;
use miette::IntoDiagnostic;
use redis::{aio::ConnectionManager, AsyncCommands};

const POOL_SIZE: usize = 5;
const REDIS_NAMESPACE: &str = "MRF-KV-STORE";

pub struct RedisBackend {
    pool: multiplex_pool::Pool<ConnectionManager>,
}

impl RedisBackend {
    pub async fn from_client(client: redis::Client) -> miette::Result<Self> {
        let pool = multiplex_pool::Pool::from_producer(
            || client.get_connection_manager(),
            POOL_SIZE,
            multiplex_pool::RoundRobinStrategy::default(),
        )
        .await
        .into_diagnostic()?;

        Ok(Self { pool })
    }
}

impl super::Backend for RedisBackend {
    type Bucket = RedisBucketBackend;

    async fn open(&self, module_name: &str, name: &str) -> Result<Self::Bucket, BoxError> {
        Ok(RedisBucketBackend {
            name: format!("{REDIS_NAMESPACE}:{module_name}:{name}"),
            pool: self.pool.clone(),
        })
    }
}

pub struct RedisBucketBackend {
    name: String,
    pool: multiplex_pool::Pool<ConnectionManager>,
}

impl super::BucketBackend for RedisBucketBackend {
    async fn exists(&self, key: &str) -> Result<bool, BoxError> {
        self.pool
            .get()
            .hexists(&self.name, key)
            .await
            .map_err(Into::into)
    }

    async fn delete(&self, key: &str) -> Result<(), BoxError> {
        self.pool
            .get()
            .hdel(&self.name, key)
            .await
            .map_err(Into::into)
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, BoxError> {
        self.pool
            .get()
            .hget(&self.name, key)
            .await
            .map_err(Into::into)
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<(), BoxError> {
        self.pool
            .get()
            .hset(&self.name, key, value)
            .await
            .map_err(Into::into)
    }
}
