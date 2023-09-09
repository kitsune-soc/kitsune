use crate::http::extractor::Json;
use axum::{
    extract::multipart::MultipartError,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use kitsune_core::error::{ApiError, Error as CoreError};
use std::str::ParseBoolError;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Broken database record encountered")]
    BrokenRecord,

    #[error(transparent)]
    Core(#[from] CoreError),

    #[error(transparent)]
    Database(#[from] diesel::result::Error),

    #[error(transparent)]
    DatabasePool(#[from] diesel_async::pooled_connection::deadpool::PoolError),

    #[error(transparent)]
    Der(#[from] der::Error),

    #[error(transparent)]
    Http(#[from] http::Error),

    #[error(transparent)]
    Multipart(#[from] MultipartError),

    #[error(transparent)]
    ParseBool(#[from] ParseBoolError),

    #[error(transparent)]
    SimdJson(#[from] simd_json::Error),

    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),

    #[error("Unconfirmed email address. Check your inbox!")]
    UnconfirmedEmailAddress,

    #[error(transparent)]
    Uuid(#[from] speedy_uuid::Error),
}

impl From<ApiError> for Error {
    fn from(value: ApiError) -> Self {
        Self::Core(value.into())
    }
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

impl From<Error> for Response {
    fn from(err: Error) -> Response {
        err.into_response()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::Database(diesel::result::Error::NotFound) => {
                StatusCode::NOT_FOUND.into_response()
            }
            Self::Core(CoreError::Validate(report)) => {
                (StatusCode::BAD_REQUEST, Json(report)).into_response()
            }
            err @ Self::Core(CoreError::Api(ApiError::NotFound)) => {
                (StatusCode::NOT_FOUND, err.to_string()).into_response()
            }
            err @ Self::Core(CoreError::Api(ApiError::BadRequest)) => {
                (StatusCode::BAD_REQUEST, err.to_string()).into_response()
            }
            err @ Self::Core(CoreError::Api(ApiError::Unauthorised)) => {
                (StatusCode::UNAUTHORIZED, err.to_string()).into_response()
            }
            err @ Self::Core(CoreError::Api(ApiError::UnsupportedMediaType)) => {
                (StatusCode::UNSUPPORTED_MEDIA_TYPE, err.to_string()).into_response()
            }
            error => {
                error!(?error, "Error occurred in handler");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
