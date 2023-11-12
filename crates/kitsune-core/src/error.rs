use std::error::Error as StdError;
use thiserror::Error;

pub type BoxError = Box<dyn StdError + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Bad request")]
    BadRequest,

    #[error("Email already taken")]
    EmailTaken,

    #[error("Internal server error")]
    InternalServerError,

    #[error("Invalid captcha")]
    InvalidCaptcha,

    #[error("Missing host")]
    MissingHost,

    #[error("Not found")]
    NotFound,

    #[error("Registrations closed")]
    RegistrationsClosed,

    #[error("Unauthorised")]
    Unauthorised,

    #[error("Unsupported media type")]
    UnsupportedMediaType,

    #[error("Username already taken")]
    UsernameTaken,
}
