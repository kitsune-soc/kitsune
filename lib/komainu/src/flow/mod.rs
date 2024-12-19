use serde::Serialize;
use std::borrow::Cow;
use strum::Display;
use thiserror::Error;

pub mod authorization;
pub mod pkce;
pub mod refresh;

trait OptionExt<T> {
    fn or_invalid_request(self) -> Result<T, FlowError>;
}

impl<T> OptionExt<T> for Option<T> {
    #[inline]
    fn or_invalid_request(self) -> Result<T, FlowError> {
        self.ok_or(FlowError::InvalidRequest)
    }
}

#[derive(Debug, Display, Error, Serialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FlowError {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,
    #[serde(skip)]
    Other(#[from] crate::error::Error),
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TokenType {
    Bearer,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum TokenResponse<'a> {
    Success {
        access_token: Cow<'a, str>,
        token_type: TokenType,
        refresh_token: Cow<'a, str>,
        expires_in: u64,
    },
    Error {
        errorr: FlowError,
    },
}
