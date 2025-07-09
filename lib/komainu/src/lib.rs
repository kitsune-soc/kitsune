#[macro_use]
extern crate tracing;

use self::{error::Error, flow::pkce, scope::Scope};

pub use self::primitive::{Authorization, Client, Request};

mod primitive;

pub mod code_grant;
pub mod error;
pub mod extract;
pub mod flow;
pub mod params;
pub mod scope;

pub struct AuthInstruction<'a, 'b> {
    pub client: &'b Client<'a>,
    pub pkce_payload: Option<&'b pkce::Payload<'a>>,
    pub scopes: &'b Scope,
}

pub trait ClientExtractor {
    fn extract(
        &self,
        client_id: &str,
        client_secret: Option<&str>,
    ) -> impl Future<Output = Result<Client<'_>, Error>> + Send;
}
