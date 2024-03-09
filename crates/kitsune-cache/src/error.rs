use redis::RedisError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Redis(#[from] RedisError),

    #[error(transparent)]
    SimdJson(#[from] simd_json::Error),
}
