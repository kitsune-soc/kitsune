use crate::http::extractor::Json;
use argon2::password_hash;
use axum::{
    extract::multipart::MultipartError,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use kitsune_service::error::{Error as ServiceError, PostError};
use std::str::ParseBoolError;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Blocking(#[from] kitsune_blocking::Error),

    #[error(transparent)]
    Cache(#[from] kitsune_cache::Error),

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
    OAuth2(#[from] OAuth2Error),

    #[cfg(feature = "oidc")]
    #[error(transparent)]
    Oidc(#[from] kitsune_oidc::Error),

    #[error(transparent)]
    ParseBool(#[from] ParseBoolError),

    #[error(transparent)]
    PasswordHash(#[from] password_hash::Error),

    #[error("Password mismatch")]
    PasswordMismatch,

    #[error(transparent)]
    Service(#[from] ServiceError),

    #[error(transparent)]
    SimdJson(#[from] simd_json::Error),

    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),

    #[error("Unconfirmed email address. Check your inbox!")]
    UnconfirmedEmailAddress,

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    #[error(transparent)]
    Uuid(#[from] speedy_uuid::Error),
}

#[derive(Debug, Error)]
pub enum OAuth2Error {
    #[error("Missing grant type")]
    MissingGrantType,

    #[error(transparent)]
    OxideAuth(#[from] oxide_auth::endpoint::OAuthError),

    #[error(transparent)]
    ParseScope(#[from] oxide_auth::primitives::scope::ParseScopeErr),

    #[error("Unknown grant type")]
    UnknownGrantType,

    #[error(transparent)]
    Web(#[from] oxide_auth_axum::WebError),
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
            Self::Service(ServiceError::Validate(report)) => {
                (StatusCode::BAD_REQUEST, Json(report)).into_response()
            }
            err @ Self::Service(ServiceError::NotFound) => {
                (StatusCode::NOT_FOUND, err.to_string()).into_response()
            }
            err @ Self::Service(ServiceError::Post(PostError::BadRequest)) => {
                (StatusCode::BAD_REQUEST, err.to_string()).into_response()
            }
            err @ Self::Service(ServiceError::Post(PostError::Unauthorised)) => {
                (StatusCode::UNAUTHORIZED, err.to_string()).into_response()
            }
            err @ Self::Service(_) => {
                (StatusCode::UNSUPPORTED_MEDIA_TYPE, err.to_string()).into_response()
            }
            error => {
                error!(?error, "Error occurred in handler");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
