use serde::Serialize;
use strum::AsRefStr;
use thiserror::Error;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Malformed body")]
    Body(#[source] BoxError),

    #[error("Missing parameter")]
    MissingParam,

    #[error("Malformed query")]
    Query(#[source] BoxError),

    #[error("Request is unauthorized")]
    Unauthorized,
}

impl Error {
    #[track_caller]
    pub(crate) fn body(err: impl Into<BoxError>) -> Self {
        Self::Body(err.into())
    }

    #[track_caller]
    pub(crate) fn query(err: impl Into<BoxError>) -> Self {
        Self::Query(err.into())
    }
}

#[derive(AsRefStr, Serialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum OAuthError {
    InvalidRequest,
    UnauthorizedClient,
    AccessDenied,
    UnsupportedResponseType,
    InvalidScope,
    ServerError,
    TemporarilyUnavailable,
}

#[derive(Serialize)]
pub struct OAuthErrorResponse {
    pub error: OAuthError,
}
