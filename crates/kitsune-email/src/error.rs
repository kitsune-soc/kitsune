use diesel_async::pooled_connection::bb8::RunError as DatabasePoolError;
use miette::Diagnostic;
use std::{
    error::Error as StdError,
    fmt::{Debug, Display},
};
use thiserror::Error;

pub type BoxError = Box<dyn StdError + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    #[error(transparent)]
    Address(#[from] lettre::address::AddressError),

    #[error(transparent)]
    DatabasePool(#[from] DatabasePoolError),

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error(transparent)]
    Lettre(#[from] lettre::error::Error),

    #[error(transparent)]
    Templating(#[from] askama::Error),

    #[error(transparent)]
    Transport(BoxError),

    #[error(transparent)]
    RenderParsing(#[from] mrml::prelude::parser::Error),

    #[error(transparent)]
    Rendering(#[from] mrml::prelude::render::Error),
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
