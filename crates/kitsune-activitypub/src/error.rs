use diesel_async::pooled_connection::bb8;
use kitsune_core::error::BoxError;
use rsa::pkcs8::der;
use std::{
    convert::Infallible,
    fmt::{Debug, Display},
};
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Instance is blocked")]
    BlockedInstance,

    #[error(transparent)]
    Cache(#[from] kitsune_cache::Error),

    #[error(transparent)]
    DatabasePool(#[from] bb8::RunError),

    #[error(transparent)]
    Der(#[from] der::Error),

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error(transparent)]
    Embed(#[from] kitsune_embed::Error),

    #[error(transparent)]
    FederationFilter(#[from] kitsune_federation_filter::error::Error),

    #[error(transparent)]
    FetchAccount(BoxError),

    #[error(transparent)]
    FetchEmoji(BoxError),

    #[error(transparent)]
    FetchPost(BoxError),

    #[error(transparent)]
    Http(#[from] http::Error),

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
    Mrf(#[from] kitsune_wasm_mrf::Error),

    #[error("Not found")]
    NotFound,

    #[error(transparent)]
    Resolver(BoxError),

    #[error(transparent)]
    Search(#[from] kitsune_search::Error),

    #[error(transparent)]
    Service(#[from] kitsune_service::error::Error),

    #[error(transparent)]
    SimdJson(#[from] simd_json::Error),

    #[error("Unsupported media type")]
    UnsupportedMediaType,

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        match err {}
    }
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
