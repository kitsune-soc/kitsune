use bytes::Bytes;
use serde::Serialize;
use std::borrow::{Borrow, Cow};
use strum::Display;
use thiserror::Error;

pub mod authorization;
pub mod pkce;
pub mod refresh;

trait OptionExt<T> {
    fn or_invalid_request(self) -> Result<T, Error>;
}

impl<T> OptionExt<T> for Option<T> {
    #[inline]
    fn or_invalid_request(self) -> Result<T, Error> {
        self.ok_or(Error::InvalidRequest)
    }
}

#[derive(Debug, Display, Error, Serialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Error {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,
    #[serde(skip)]
    Other(#[from] crate::error::Error),
}

#[derive(Clone, Serialize)]
#[non_exhaustive]
pub enum TokenType {
    Bearer,
}

#[derive(Clone, Serialize)]
pub struct SuccessTokenResponse<'a> {
    pub access_token: Cow<'a, str>,
    pub token_type: TokenType,
    pub refresh_token: Cow<'a, str>,
    pub expires_in: u64,
}

#[derive(Serialize)]
pub struct ErrorTokenResponse {
    pub error: Error,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum TokenResponse<'a> {
    Success(SuccessTokenResponse<'a>),
    Error(ErrorTokenResponse),
}

impl<'a> TokenResponse<'a> {
    #[must_use]
    pub fn unwrap_success(self) -> SuccessTokenResponse<'a> {
        match self {
            Self::Success(resp) => resp,
            Self::Error(..) => panic!("expected success, got error"),
        }
    }

    #[must_use]
    pub fn unwrap_error(self) -> ErrorTokenResponse {
        match self {
            Self::Success(..) => panic!("expected error, got success"),
            Self::Error(resp) => resp,
        }
    }
}

impl<'a> From<SuccessTokenResponse<'a>> for TokenResponse<'a> {
    #[inline]
    fn from(value: SuccessTokenResponse<'a>) -> Self {
        Self::Success(value)
    }
}

impl From<ErrorTokenResponse> for TokenResponse<'_> {
    #[inline]
    fn from(value: ErrorTokenResponse) -> Self {
        Self::Error(value)
    }
}

pub struct Impls<AI, CI, RI> {
    pub auth_issuer: AI,
    pub client_extractor: CI,
    pub refresh_issuer: RI,
}

#[cfg_attr(not(coverage), instrument(skip_all))]
pub async fn dispatch<AI, CI, RI>(
    req: &crate::Request<'_>,
    impls: &Impls<AI, CI, RI>,
) -> Result<http::Response<Bytes>, Error>
where
    AI: self::authorization::Issuer,
    CI: crate::ClientExtractor,
    RI: self::refresh::Issuer,
{
    let grant_type = req.body.get("grant_type").map(Borrow::borrow);

    let token_response = match grant_type {
        Some("authorization_code") => {
            authorization::perform(req, &impls.client_extractor, &impls.auth_issuer).await?
        }
        Some("refresh_token") => {
            refresh::perform(req, &impls.client_extractor, &impls.refresh_issuer).await?
        }
        _ => ErrorTokenResponse {
            error: Error::UnsupportedGrantType,
        }
        .into(),
    };

    let mut response = http::Response::builder();
    response = if matches!(token_response, TokenResponse::Success(..)) {
        response.status(http::StatusCode::OK)
    } else {
        response.status(http::StatusCode::BAD_REQUEST)
    };

    let body = Bytes::from(sonic_rs::to_vec(&token_response).unwrap());
    Ok(response.body(body).unwrap())
}
