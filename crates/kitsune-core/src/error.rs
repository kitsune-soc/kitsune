use std::error::Error as ErrorTrait;

use kitsune_http_signatures::ring;
use thiserror::Error;

pub type BoxError = Box<dyn ErrorTrait + Send + Sync>;
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

#[derive(Debug, Error)]
pub enum FederationFilterError {
    #[error(transparent)]
    Glob(#[from] globset::Error),

    #[error("Host missing from URL")]
    HostMissing,

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}

#[derive(Debug, Error)]
pub enum UploadError {
    #[error(transparent)]
    ImageProcessingError(#[from] img_parts::Error),

    #[error(transparent)]
    StreamError(#[from] BoxError),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Api(#[from] ApiError),

    #[error(transparent)]
    Blocking(#[from] kitsune_blocking::Error),

    #[error(transparent)]
    Cache(#[from] kitsune_cache::Error),

    #[error(transparent)]
    Captcha(#[from] kitsune_captcha::Error),

    #[error(transparent)]
    DatabasePool(#[from] diesel_async::pooled_connection::deadpool::PoolError),

    #[error(transparent)]
    Der(#[from] pkcs8::der::Error),

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error(transparent)]
    Email(#[from] kitsune_email::error::Error),

    #[error(transparent)]
    Embed(#[from] kitsune_embed::Error),

    #[error(transparent)]
    Event(kitsune_messaging::BoxError),

    #[error(transparent)]
    FederationFilter(#[from] FederationFilterError),

    #[error(transparent)]
    Http(#[from] http::Error),

    #[error(transparent)]
    HttpClient(#[from] kitsune_http_client::Error),

    #[error(transparent)]
    JobQueue(#[from] athena::Error),

    #[error(transparent)]
    KeyRejected(#[from] ring::error::KeyRejected),

    #[error(transparent)]
    Mime(#[from] mime::FromStrError),

    #[error(transparent)]
    PasswordHash(#[from] password_hash::Error),

    #[error(transparent)]
    Pkcs8(#[from] pkcs8::Error),

    #[error(transparent)]
    PostProcessing(post_process::BoxError),

    #[error(transparent)]
    Rsa(#[from] rsa::Error),

    #[error(transparent)]
    Search(#[from] kitsune_search::Error),

    #[error(transparent)]
    SimdJson(#[from] simd_json::Error),

    #[error(transparent)]
    Spki(#[from] pkcs8::spki::Error),

    #[error(transparent)]
    Storage(kitsune_storage::BoxError),

    #[error(transparent)]
    Upload(#[from] UploadError),

    #[error(transparent)]
    UriInvalid(#[from] http::uri::InvalidUri),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    #[error(transparent)]
    Validate(#[from] garde::Report),
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
