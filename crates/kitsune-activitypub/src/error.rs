use std::convert::Infallible;

use diesel_async::pooled_connection::deadpool::PoolError as DatabasePoolError;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Instance is blocked")]
    BlockedInstance,

    #[error(transparent)]
    Cache(#[from] kitsune_cache::Error),

    #[error(transparent)]
    DatabasePool(#[from] DatabasePoolError),

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error(transparent)]
    Embed(#[from] kitsune_embed::Error),

    #[error(transparent)]
    HttpClient(#[from] kitsune_http_client::Error),

    #[error("Invalid ActivityPub document")]
    InvalidDocument,

    #[error("Invalid ActivityPub response")]
    InvalidResponse,

    #[error(transparent)]
    InvalidUri(#[from] http::uri::InvalidUri),

    #[error("Missing host")]
    MissingHost,

    #[error(transparent)]
    Search(#[from] kitsune_search::Error),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}

impl From<Infallible> for Error {
    fn from(value: Infallible) -> Self {
        unreachable!();
    }
}

impl<E> From<kitsune_db::PoolError<E>> for Error
where
    E: Into<Error>,
{
    fn from(value: kitsune_db::PoolError<E>) -> Self {
        match value {
            kitsune_db::PoolError::Pool(err) => err.into(),
            kitsune_db::PoolError::User(err) => err.into(),
        }
    }
}
