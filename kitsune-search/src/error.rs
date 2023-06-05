use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] diesel::result::Error),

    #[error(transparent)]
    DatabasePool(#[from] diesel_async::pooled_connection::deadpool::PoolError),

    #[cfg(feature = "meilisearch")]
    #[error(transparent)]
    Meilisearch(#[from] meilisearch_sdk::errors::Error),

    #[cfg(feature = "kitsune-search")]
    #[error(transparent)]
    TonicStatus(#[from] tonic::Status),

    #[cfg(feature = "kitsune-search")]
    #[error(transparent)]
    TonicTransport(#[from] tonic::transport::Error),
}
