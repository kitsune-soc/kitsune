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
        error: Error,
    },
}

pub struct Impls<AI, CI, RI> {
    pub auth_issuer: AI,
    pub client_extractor: CI,
    pub refresh_issuer: RI,
}

#[instrument(skip_all)]
pub async fn dispatch<AI, CI, RI>(
    req: &crate::Request<'_>,
    impls: &Impls<AI, CI, RI>,
) -> Result<http::Response<Bytes>, Error>
where
    AI: self::authorization::Issuer,
    CI: crate::ClientExtractor,
    RI: self::refresh::Issuer,
{
    let grant_type = req.query.get("grant_type").map(Borrow::borrow);

    let token_response = match grant_type {
        Some("authorization_code") => {
            authorization::perform(req, &impls.client_extractor, &impls.auth_issuer).await?
        }
        Some("refresh_token") => {
            refresh::perform(req, &impls.client_extractor, &impls.refresh_issuer).await?
        }
        _ => TokenResponse::Error {
            error: Error::UnsupportedGrantType,
        },
    };

    let mut response = http::Response::builder();
    response = if matches!(token_response, TokenResponse::Success { .. }) {
        response.status(http::StatusCode::OK)
    } else {
        response.status(http::StatusCode::BAD_REQUEST)
    };

    let body = Bytes::from(sonic_rs::to_vec(&token_response).unwrap());
    Ok(response.body(body).unwrap())
}
