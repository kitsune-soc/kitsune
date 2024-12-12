#[macro_use]
extern crate tracing;

use bytes::Bytes;
use std::collections::HashSet;
use std::{borrow::Cow, future::Future};
use strum::AsRefStr;

pub use self::error::{Error, Result};
pub use self::params::ParamStorage;

mod error;
mod params;

pub mod authorize;

trait OptionExt<T> {
    fn or_missing_param(self) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    #[inline]
    fn or_missing_param(self) -> Result<T> {
        self.ok_or(Error::MissingParam)
    }
}

// TODO: Refactor into `AuthorizerExtractor` and `Authorizer`
//
// `AuthorizerExtractor` contains the `ClientExtractor`, so we can load client info.
// `Authorizer` is the handle passed to the consumer to accept or deny the request.
// Unlike `oxide-auth`, we won't force the user to implement a trait here, the flow better integrates with a simple function.
//
// Because we use native async traits where needed, we can't box the traits (not that we want to), so at least the compiler can inline stuff well

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
fn get_from_either<'a>(
    key: &str,
    left: &'a ParamStorage<&str, &str>,
    right: &'a ParamStorage<&str, &str>,
) -> Option<&'a str> {
    left.get(key).or_else(|| right.get(key)).map(|item| &**item)
}
