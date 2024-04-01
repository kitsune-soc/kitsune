use diesel_async::pooled_connection::bb8::RunError as DatabasePoolError;
use std::{error::Error as StdError, fmt::Debug};
use thiserror::Error;

pub type BoxError = Box<dyn StdError + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
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
