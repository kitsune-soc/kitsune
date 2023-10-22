use core::fmt;
use diesel_async::pooled_connection::deadpool::PoolError;
use std::error::Error as StdError;

pub type BoxError = Box<dyn StdError + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct EnumConversionError(pub i32);

impl fmt::Display for EnumConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Attempted to convert integer to enum. Got invalid value \"{}\"",
            self.0
        )
    }
}

impl StdError for EnumConversionError {}

#[derive(Debug)]
pub struct IsoCodeConversionError(pub String);

impl fmt::Display for IsoCodeConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Attempted to convert string to ISO code. Got invalid value \"{}\"",
            self.0
        )
    }
}

impl StdError for IsoCodeConversionError {}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Blocking(#[from] kitsune_blocking::Error),

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error(transparent)]
    DieselConnection(#[from] diesel::result::ConnectionError),

    #[error(transparent)]
    Migration(BoxError),

    #[error(transparent)]
    Pool(#[from] PoolError),
}
