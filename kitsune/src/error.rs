use argon2::password_hash;
use axum::response::{IntoResponse, Response};
use deadpool_redis::PoolError;
use http::StatusCode;
use redis::RedisError;
use rsa::{
    pkcs1,
    pkcs8::{self, der},
};
use sea_orm::TransactionError;
use thiserror::Error;
use tokio::sync::oneshot;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found")]
    NotFound,

    #[error("Unauthorised")]
    Unauthorised,

    #[error("Unsupported media type")]
    UnsupportedMediaType,
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
    Http(#[from] http::Error),

    #[error(transparent)]
    HttpClient(#[from] kitsune_http_client::Error),

    #[error(transparent)]
    HttpSignature(#[from] kitsune_http_signatures::Error),

    #[error(transparent)]
    KeyRejected(#[from] kitsune_http_signatures::ring::error::KeyRejected),

    #[error("Malformed ActivityPub object")]
    MalformedApObject,

    #[error("OAuth application not found")]
    OAuthApplicationNotFound,

    #[error(transparent)]
    Oneshot(#[from] oneshot::error::RecvError),

    #[error(transparent)]
    Rsa(#[from] rsa::errors::Error),

    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),

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
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    TonicStatus(#[from] tonic::Status),

    #[error(transparent)]
    TonicTransport(#[from] tonic::transport::Error),

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

impl From<TransactionError<Error>> for Error {
    fn from(err: TransactionError<Error>) -> Self {
        match err {
            TransactionError::Connection(db) => Self::Database(db),
            TransactionError::Transaction(err) => err,
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
            err @ Self::OAuthApplicationNotFound => {
                (StatusCode::BAD_REQUEST, err.to_string()).into_response()
            }
            err @ (Self::Api(ApiError::Unauthorised) | Self::PasswordMismatch) => {
                (StatusCode::UNAUTHORIZED, err.to_string()).into_response()
            }
            err => {
                error!(error = %err, "Error occurred in handler");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
