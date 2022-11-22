use argon2::password_hash;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rsa::{pkcs1, pkcs8};
use sea_orm::TransactionError;
use thiserror::Error;
use tokio::sync::oneshot;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Broken database record encountered")]
    BrokenRecord,

    #[error(transparent)]
    Database(#[from] sea_orm::DbErr),

    #[error(transparent)]
    HttpSignature(#[from] phenomenon_http_signatures::Error),

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
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    #[error("User not found")]
    UserNotFound,
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
            err @ (Self::OAuthApplicationNotFound | Self::UserNotFound) => {
                (StatusCode::BAD_REQUEST, err.to_string()).into_response()
            }
            err @ Self::PasswordMismatch => {
                (StatusCode::UNAUTHORIZED, err.to_string()).into_response()
            }
            err => {
                error!(error = %err, "Error occurred in handler");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
