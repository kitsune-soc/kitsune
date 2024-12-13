#[macro_use]
extern crate tracing;

use bytes::Bytes;
use headers::HeaderMapExt;
use std::{borrow::Cow, future::Future};
use strum::AsRefStr;

pub use self::error::{Error, Result};
pub use self::params::ParamStorage;

mod error;
mod params;

pub mod authorize;
pub mod flow;

trait OptionExt<T> {
    fn or_missing_param(self) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    #[inline]
    fn or_missing_param(self) -> Result<T> {
        self.ok_or(Error::MissingParam)
    }
}

pub struct Client<'a> {
    pub client_id: &'a str,
    pub client_secret: &'a str,
    pub scopes: Cow<'a, [Cow<'a, str>]>,
    pub redirect_uri: Cow<'a, str>,
}

pub trait ClientExtractor {
    fn extract(
        &self,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> impl Future<Output = Result<Client<'_>>> + Send;
}

#[derive(AsRefStr)]
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

#[inline]
fn deserialize_body<'a, T: serde::Deserialize<'a>>(req: &'a http::Request<Bytes>) -> Result<T> {
    // Not part of the RFC, but a bunch of implementations allow this.
    // And because they allow this, clients make use of this.
    //
    // Done to increase compatibility.
    let content_type = req.headers().typed_get::<headers::ContentType>();
    if content_type == Some(headers::ContentType::json()) {
        sonic_rs::from_slice(req.body()).map_err(Error::body)
    } else {
        serde_urlencoded::from_bytes(req.body()).map_err(Error::body)
    }
}

#[inline]
fn get_from_either<'a>(
    key: &str,
    left: &'a ParamStorage<&str, &str>,
    right: &'a ParamStorage<&str, &str>,
) -> Option<&'a str> {
    left.get(key).or_else(|| right.get(key)).map(|item| &**item)
}
