use diesel_async::pooled_connection::bb8;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] diesel::result::Error),

    #[error(transparent)]
    DatabasePool(#[from] bb8::RunError),

    #[cfg(feature = "meilisearch")]
    #[error(transparent)]
    Meilisearch(#[from] meilisearch_sdk::errors::Error),
}
