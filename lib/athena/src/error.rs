use std::error::Error as StdError;
use thiserror::Error;

pub type BoxError = Box<dyn StdError + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ContextRepository(BoxError),

    #[error(transparent)]
    Redis(#[from] redis::RedisError),

    #[error(transparent)]
    RedisPool(#[from] deadpool_redis::PoolError),

    #[error(transparent)]
    SimdJson(#[from] simd_json::Error),

    #[error(transparent)]
    Uuid(#[from] speedy_uuid::Error),
}
