use crate::http::extractor::Json;
use argon2::password_hash;
use axum::{
    extract::multipart::MultipartError,
    response::{IntoResponse, Response},
};
use color_eyre::eyre;
use diesel_async::pooled_connection::bb8;
use http::StatusCode;
use kitsune_core::error::HttpError;
use kitsune_service::error::{Error as ServiceError, PostError};
use std::{fmt::Debug, str::ParseBoolError};
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    CoreHttp(#[from] kitsune_core::error::HttpError),

    #[error(transparent)]
    Database(#[from] diesel::result::Error),

    #[error(transparent)]
    Fetcher(eyre::Report),

    #[error(transparent)]
    Messaging(kitsune_messaging::BoxError),

    #[error(transparent)]
    OAuth2(#[from] OAuth2Error),

    #[error(transparent)]
    ParseBool(#[from] ParseBoolError),

    #[error("Password mismatch")]
    PasswordMismatch,

    #[error(transparent)]
    Service(#[from] ServiceError),

    #[error("Unconfirmed email address. Check your inbox!")]
    UnconfirmedEmailAddress,

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
    bb8::RunError,
    blowocking::Error,
    der::Error,
    http::Error,
    kitsune_activitypub::error::Error,
    kitsune_cache::Error,
    kitsune_email::error::Error,
    kitsune_wasm_mrf::Error,
    MultipartError,
    password_hash::Error,
    simd_json::Error,
    speedy_uuid::Error,
    tokio::task::JoinError,
    url::ParseError,
}

#[cfg(feature = "mastodon-api")]
other_error!(kitsune_mastodon::error::Error);

#[cfg(feature = "oidc")]
other_error!(kitsune_oidc::Error);

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
            err @ Self::CoreHttp(HttpError::NotFound) => {
                (StatusCode::NOT_FOUND, err.to_string()).into_response()
            }
            err @ Self::Service(ServiceError::Post(PostError::BadRequest)) => {
                (StatusCode::BAD_REQUEST, err.to_string()).into_response()
            }
            err @ Self::Service(ServiceError::Post(PostError::Unauthorised)) => {
                (StatusCode::UNAUTHORIZED, err.to_string()).into_response()
            }
            error => {
                error!(?error, "Error occurred in handler");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
