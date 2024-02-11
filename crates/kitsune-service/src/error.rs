use diesel_async::pooled_connection::bb8;
use kitsune_http_signatures::ring;
use std::{
    error::Error as StdError,
    fmt::{Debug, Display},
};
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
    Attachment(#[from] AttachmentError),

    #[error(transparent)]
    Blocking(#[from] blowocking::Error),

    #[error(transparent)]
    Cache(#[from] kitsune_cache::Error),

    #[error(transparent)]
    Captcha(#[from] kitsune_captcha::Error),

    #[error(transparent)]
    DatabasePool(#[from] bb8::RunError),

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
    Fetcher(BoxError),

    #[error(transparent)]
    Http(#[from] http::Error),

    #[error(transparent)]
    HttpClient(#[from] kitsune_http_client::Error),

    #[error(transparent)]
    HttpHeaderToStr(#[from] http::header::ToStrError),

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
    Post(#[from] PostError),

    #[error(transparent)]
    PostProcessing(post_process::BoxError),

    #[error(transparent)]
    Resolver(BoxError),

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
    UriInvalid(#[from] http::uri::InvalidUri),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    #[error(transparent)]
    User(#[from] UserError),

    #[error(transparent)]
    Validate(#[from] garde::Report),
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
