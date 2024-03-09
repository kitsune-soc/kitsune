use std::fmt::{Debug, Display};

use diesel_async::pooled_connection::bb8;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Cache(#[from] kitsune_cache::Error),

    #[error(transparent)]
    DatabasePool(#[from] bb8::RunError),

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error(transparent)]
    Embed(#[from] kitsune_embed::Error),

    #[error(transparent)]
    Service(#[from] kitsune_service::error::Error),
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
