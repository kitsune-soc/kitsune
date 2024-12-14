use thiserror::Error;
use serde::Serialize;
use strum::AsRefStr;

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

impl From<Error> for OAuthError {
    #[track_caller]
    fn from(value: Error) -> Self {
        debug!(error = ?value);

        match value {
            Error::Body(..) | Error::MissingParam | Error::Query(..) => Self::InvalidRequest,
            Error::Unauthorized => Self::AccessDenied,
        }
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

macro_rules! fallible {
    ($op:expr) => {{
        match { $op } {
            Ok(val) => val,
            Err(error) => {
                debug!(?error);
                $crate::error::yield_error!(error);
            }
        }
    }};
}

macro_rules! yield_error {
    (@ser $error:expr) => {{
        return ::http::Response::builder()
            .status(::http::StatusCode::BAD_REQUEST)
            .body(sonic_rs::to_vec(&$error).unwrap().into())
            .unwrap();
    }};
    ($error:expr) => {{
        $crate::error::yield_error!(@ser $crate::error::OAuthErrorResponse {
            error: $error.into(),
        });
    }};
}

pub(crate) use {fallible, yield_error};
