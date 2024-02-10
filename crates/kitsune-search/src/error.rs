use diesel_async::pooled_connection::bb8;
use miette::Diagnostic;
use std::fmt::{Debug, Display};
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] diesel::result::Error),

    #[error(transparent)]
    DatabasePool(#[from] bb8::RunError),

    #[cfg(feature = "meilisearch")]
    #[error(transparent)]
    Meilisearch(#[from] meilisearch_sdk::errors::Error),
}

impl<E> From<kitsune_db::PoolError<E>> for Error
where
    E: Into<Error> + Debug + Display,
{
    fn from(value: kitsune_db::PoolError<E>) -> Self {
        match value {
            kitsune_db::PoolError::Pool(err) => err.into(),
            kitsune_db::PoolError::User(err) => err.into(),
        }
    }
}
