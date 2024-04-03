use diesel_async::pooled_connection::bb8;
use std::{error::Error as StdError, fmt::Debug};
use thiserror::Error;

pub type BoxError = Box<dyn StdError + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum AttachmentError {
    #[error(transparent)]
    ImageProcessingError(#[from] img_parts::Error),

    #[error("Not found")]
    NotFound,

    #[error(transparent)]
    StreamError(#[from] BoxError),
}

#[derive(Debug, Error)]
pub enum PostError {
    #[error("Bad request")]
    BadRequest,

    #[error("Unauthorised")]
    Unauthorised,
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error("Invalid captcha")]
    InvalidCaptcha,

    #[error("Registrations closed")]
    RegistrationsClosed,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Event(kitsune_messaging::BoxError),

    #[error(transparent)]
    Fetcher(eyre::Report),

    #[error(transparent)]
    Post(#[from] PostError),

    #[error(transparent)]
    PostProcessing(post_process::BoxError),

    #[error(transparent)]
    Resolver(eyre::Report),

    #[error(transparent)]
    Storage(kitsune_storage::BoxError),

    #[error(transparent)]
    Validate(#[from] garde::Report),

    #[error(transparent)]
    Other(eyre::Report),
}

macro_rules! other_error {
    ($($err_ty:path),+$(,)?) => {
        $(
            impl From<$err_ty> for Error {
                fn from(err: $err_ty) -> Self {
                    Self::Other(err.into())
                }
            }
        )+
    };
}

other_error! {
    athena::Error,
    AttachmentError,
    bb8::RunError,
    blowocking::Error,
    diesel::result::Error,
    http::Error,
    http::header::ToStrError,
    http::uri::InvalidUri,
    kitsune_cache::Error,
    kitsune_captcha::Error,
    kitsune_email::error::Error,
    kitsune_embed::Error,
    kitsune_http_client::Error,
    kitsune_search::Error,
    mime::FromStrError,
    password_hash::Error,
    pkcs8::Error,
    pkcs8::der::Error,
    pkcs8::spki::Error,
    rsa::Error,
    simd_json::Error,
    url::ParseError,
    UserError,
}
