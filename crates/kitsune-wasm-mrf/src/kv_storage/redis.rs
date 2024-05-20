use super::BoxError;
use fred::{clients::RedisPool, interfaces::HashesInterface};
use kitsune_derive::typed_builder::TypedBuilder;

const REDIS_NAMESPACE: &str = "MRF-KV-STORE";

#[derive(TypedBuilder)]
pub struct RedisBackend {
    pool: RedisPool,
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
    pool: RedisPool,
}

impl super::BucketBackend for RedisBucketBackend {
    async fn exists(&self, key: &str) -> Result<bool, BoxError> {
        self.pool.hexists(&self.name, key).await.map_err(Into::into)
    }

    async fn delete(&self, key: &str) -> Result<(), BoxError> {
        self.pool.hdel(&self.name, key).await.map_err(Into::into)
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, BoxError> {
        self.pool.hget(&self.name, key).await.map_err(Into::into)
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<(), BoxError> {
        self.pool
            .hset(&self.name, (key, value))
            .await
            .map_err(Into::into)
    }
}
