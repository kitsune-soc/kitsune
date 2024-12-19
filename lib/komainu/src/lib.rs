#[macro_use]
extern crate tracing;

use self::{error::Error, flow::pkce};
use std::{borrow::Cow, future::Future};
use subtle::ConstantTimeEq;

pub mod code_grant;
pub mod error;
pub mod extract;
pub mod flow;
pub mod params;
pub mod primitive;

pub struct Authorization<'a> {
    pub code: Cow<'a, str>,
    pub client: Client<'a>,
    pub pkce_payload: Option<pkce::Payload<'a>>,
    pub scopes: Cow<'a, [Cow<'a, str>]>,
}

pub struct AuthInstruction<'a, 'b> {
    pub client: &'b Client<'a>,
    pub pkce_payload: Option<&'b pkce::Payload<'a>>,
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
    ) -> impl Future<Output = Result<Client<'_>, Error>> + Send;
}
