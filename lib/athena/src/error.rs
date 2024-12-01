use std::error::Error as StdError;
use thiserror::Error;

pub type BoxError = Box<dyn StdError + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ContextRepository(BoxError),

    #[cfg(feature = "redis")]
    #[error(transparent)]
    Redis(#[from] fred::error::Error),

    #[cfg(feature = "redis")]
    #[error(transparent)]
    SonicRs(#[from] sonic_rs::Error),

    #[error(transparent)]
    Uuid(#[from] speedy_uuid::Error),
}
