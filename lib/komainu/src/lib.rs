#[macro_use]
extern crate tracing;

use self::{
    error::{BoxError, Error},
    flow::pkce,
    params::ParamStorage,
    scope::Scope,
};
use http_body_util::BodyExt;
use std::{borrow::Cow, fmt, future::Future};
use subtle::ConstantTimeEq;

pub mod code_grant;
pub mod error;
pub mod extract;
pub mod flow;
pub mod params;
pub mod scope;

pub struct Request<'a> {
    pub headers: http::HeaderMap,
    pub query: ParamStorage<Cow<'a, str>, Cow<'a, str>>,
    pub body: ParamStorage<Cow<'a, str>, Cow<'a, str>>,
}

impl Request<'_> {
    #[inline]
    #[instrument(skip_all)]
    pub async fn read_from<B>(req: http::Request<B>) -> Result<Self, Error>
    where
        B: http_body::Body,
        B::Error: Into<BoxError>,
    {
        let raw_query = req.uri().query().unwrap_or("");
        let query = serde_urlencoded::from_str(raw_query).map_err(Error::query)?;

        let (parts, body) = req.into_parts();
        let collected = body.collect().await.map_err(Error::body)?.to_bytes();
        let req = http::Request::from_parts(parts, collected.clone());

        let body = if collected.is_empty() {
            ParamStorage::default()
        } else {
            crate::extract::body(&req).map_err(Error::body)?
        };

        Ok(Self {
            headers: req.headers().clone(),
            query,
            body,
        })
    }
}

#[derive(Clone)]
pub struct Authorization<'a> {
    pub code: Cow<'a, str>,
    pub client: Client<'a>,
    pub pkce_payload: Option<pkce::Payload<'a>>,
    pub scopes: Scope,
    pub user_id: Cow<'a, str>,
}

impl Authorization<'_> {
    pub fn into_owned(self) -> Authorization<'static> {
        Authorization {
            code: self.code.into_owned().into(),
            client: self.client.into_owned(),
            pkce_payload: self.pkce_payload.map(pkce::Payload::into_owned),
            scopes: self.scopes,
            user_id: self.user_id.into_owned().into(),
        }
    }
}

pub struct AuthInstruction<'a, 'b> {
    pub client: &'b Client<'a>,
    pub pkce_payload: Option<&'b pkce::Payload<'a>>,
    pub scopes: &'b Scope,
}

#[derive(Clone)]
pub struct Client<'a> {
    pub client_id: Cow<'a, str>,
    pub client_secret: Cow<'a, str>,
    pub scopes: Scope,
    pub redirect_uri: Cow<'a, str>,
}

impl Client<'_> {
    #[must_use]
    pub fn into_owned(self) -> Client<'static> {
        Client {
            client_id: self.client_id.into_owned().into(),
            client_secret: self.client_secret.into_owned().into(),
            scopes: self.scopes,
            redirect_uri: self.redirect_uri.into_owned().into(),
        }
    }
}

impl fmt::Debug for Client<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .field("client_id", &self.client_id)
            .field("client_secret", &"[redacted]")
            .field("scopes", &self.scopes)
            .field("redirect_uri", &self.redirect_uri)
            .finish_non_exhaustive()
    }
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
