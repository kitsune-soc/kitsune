use argon2::password_hash;
use axum::{
    extract::multipart::MultipartError,
    response::{IntoResponse, Response},
};
use deadpool_redis::PoolError;
use http::StatusCode;
use kitsune_messaging::BoxError;
use openidconnect::{
    core::CoreErrorResponseType, ClaimsVerificationError, RequestTokenError, SigningError,
    StandardErrorResponse,
};
use redis::RedisError;
use rsa::{
    pkcs1,
    pkcs8::{self, der, spki},
};
use sea_orm::TransactionError;
use std::{error::Error as StdError, str::ParseBoolError};
use thiserror::Error;
use tokio::sync::oneshot;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Bad request")]
    BadRequest,

    #[error("Email already taken")]
    EmailTaken,

    #[error("Internal server error")]
    InternalServerError,

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
pub enum CacheError {
    #[error(transparent)]
    Pool(#[from] PoolError),

    #[error(transparent)]
    Redis(#[from] RedisError),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
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
pub enum OidcError {
    #[error(transparent)]
    ClaimsVerification(#[from] ClaimsVerificationError),

    #[error(transparent)]
    LoginState(#[from] CacheError),

    #[error("Missing Email address")]
    MissingEmail,

    #[error("Mismatching hash")]
    MismatchingHash,

    #[error("Missing ID token")]
    MissingIdToken,

    #[error("Missing username")]
    MissingUsername,

    #[error(transparent)]
    RequestToken(
        #[from]
        RequestTokenError<
            kitsune_http_client::Error,
            StandardErrorResponse<CoreErrorResponseType>,
        >,
    ),

    #[error(transparent)]
    Signing(#[from] SigningError),

    #[error("Unknown CSRF token")]
    UnknownCsrfToken,
}

#[derive(Debug, Error)]
pub enum SearchError {
    #[error(transparent)]
    Database(#[from] sea_orm::DbErr),

    #[cfg(feature = "meilisearch")]
    #[error(transparent)]
    Meilisearch(#[from] meilisearch_sdk::errors::Error),

    #[error(transparent)]
    TonicStatus(#[from] tonic::Status),

    #[error(transparent)]
    TonicTransport(#[from] tonic::transport::Error),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Api(#[from] ApiError),

    #[error("Broken database record encountered")]
    BrokenRecord,

    #[error(transparent)]
    Cache(#[from] CacheError),

    #[error(transparent)]
    Database(#[from] sea_orm::DbErr),

    #[error(transparent)]
    Der(#[from] der::Error),

    #[error(transparent)]
    Event(BoxError),

    #[error(transparent)]
    FederationFilter(#[from] FederationFilterError),

    #[error(transparent)]
    Http(#[from] http::Error),

    #[error(transparent)]
    HttpClient(#[from] kitsune_http_client::Error),

    #[error(transparent)]
    HttpSignature(#[from] kitsune_http_signatures::Error),

    #[error(transparent)]
    KeyRejected(#[from] kitsune_http_signatures::ring::error::KeyRejected),

    #[error("Malformed ActivityPub object")]
    MalformedApObject,

    #[error(transparent)]
    Mime(#[from] mime::FromStrError),

    #[error(transparent)]
    Multipart(#[from] MultipartError),

    #[error("OAuth application not found")]
    OAuthApplicationNotFound,

    #[error(transparent)]
    ParseBool(#[from] ParseBoolError),

    #[error(transparent)]
    Oneshot(#[from] oneshot::error::RecvError),

    #[error(transparent)]
    Rsa(#[from] rsa::errors::Error),

    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),

    #[error(transparent)]
    Oidc(#[from] OidcError),

    #[error(transparent)]
    PasswordHash(#[from] password_hash::Error),

    #[error("Password mismatch")]
    PasswordMismatch,

    #[error(transparent)]
    Pkcs1(#[from] pkcs1::Error),

    #[error(transparent)]
    Pkcs8(#[from] pkcs8::Error),

    #[error(transparent)]
    PostProcessing(post_process::BoxError),

    #[error(transparent)]
    Search(#[from] SearchError),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Spki(#[from] spki::Error),

    #[error(transparent)]
    Storage(kitsune_storage::BoxError),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    #[error(transparent)]
    Uuid(#[from] uuid::Error),
}

impl From<Error> for Response {
    fn from(err: Error) -> Response {
        err.into_response()
    }
}

impl<E> From<TransactionError<E>> for Error
where
    E: StdError + Into<Error>,
{
    fn from(err: TransactionError<E>) -> Self {
        match err {
            TransactionError::Connection(db) => Self::Database(db),
            TransactionError::Transaction(err) => err.into(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::Database(sea_orm::DbErr::RecordNotFound(..)) => {
                StatusCode::NOT_FOUND.into_response()
            }
            err @ Self::Api(ApiError::NotFound) => {
                (StatusCode::NOT_FOUND, err.to_string()).into_response()
            }
            err @ (Self::Api(ApiError::BadRequest) | Self::OAuthApplicationNotFound) => {
                (StatusCode::BAD_REQUEST, err.to_string()).into_response()
            }
            err @ (Self::Api(ApiError::Unauthorised) | Self::PasswordMismatch) => {
                (StatusCode::UNAUTHORIZED, err.to_string()).into_response()
            }
            err @ Self::Api(ApiError::UnsupportedMediaType) => {
                (StatusCode::UNSUPPORTED_MEDIA_TYPE, err.to_string()).into_response()
            }
            err => {
                error!(error = ?err, "Error occurred in handler");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
