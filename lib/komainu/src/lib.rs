#[macro_use]
extern crate tracing;

use self::flow::PkcePayload;
use std::{borrow::Cow, future::Future};
use subtle::ConstantTimeEq;

pub use self::error::{Error, Result};
pub use self::params::ParamStorage;

mod error;
mod extractor;
mod params;

pub mod authorize;
pub mod flow;

trait OptionExt<T> {
    fn or_missing_param(self) -> Result<T>;
    fn or_unauthorized(self) -> Result<T>;
}

impl<T> OptionExt<T> for Option<T> {
    #[inline]
    fn or_missing_param(self) -> Result<T> {
        self.ok_or(Error::MissingParam)
    }

    #[inline]
    fn or_unauthorized(self) -> Result<T> {
        self.ok_or(Error::Unauthorized)
    }
}

pub struct Authorization<'a> {
    pub code: Cow<'a, str>,
    pub client: Client<'a>,
    pub pkce_payload: Option<PkcePayload<'a>>,
    pub scopes: Cow<'a, [Cow<'a, str>]>,
}

pub struct PreAuthorization<'a, 'b> {
    pub client: &'b Client<'a>,
    pub pkce_payload: Option<&'b PkcePayload<'a>>,
    pub scopes: &'b [&'b str],
}

#[derive(Clone)]
pub struct Client<'a> {
    pub client_id: &'a str,
    pub client_secret: &'a str,
    pub scopes: Cow<'a, [Cow<'a, str>]>,
    pub redirect_uri: Cow<'a, str>,
}

impl PartialEq for Client<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let client_id_l = self.client_id.as_bytes();
        let client_id_r = other.client_id.as_bytes();

        let client_secret_l = self.client_secret.as_bytes();
        let client_secret_r = other.client_secret.as_bytes();

        (client_id_l.ct_eq(client_id_r) & client_secret_l.ct_eq(client_secret_r)).into()
    }
}

pub trait ClientExtractor {
    fn extract(
        &self,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> impl Future<Output = Result<Client<'_>>> + Send;
}
