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

pub trait AuthIssuer {
    type UserId;

    fn issue_code(
        &self,
        user_id: Self::UserId,
        client_id: &str,
        scopes: &[&str],
    ) -> impl Future<Output = Result<String>> + Send;
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

pub struct AuthorizerExtractor<AI, CE> {
    // pls do not use ai for this, even if the type alias implies it.
    // kthx bestie. bussi aufs bauchi.
    auth_issuer: AI,
    client_extractor: CE,
}

impl<AI, CE> AuthorizerExtractor<AI, CE>
where
    CE: ClientExtractor,
{
    pub fn new(auth_issuer: AI, client_extractor: CE) -> Self {
        Self {
            auth_issuer,
            client_extractor,
        }
    }

    pub async fn extract<'a>(&'a self, req: &'a http::Request<()>) -> Result<Authorizer<'a, AI>> {
        let query: ParamStorage<&str, &str> =
            serde_urlencoded::from_str(req.uri().query().or_missing_param()?)
                .map_err(Error::query)?;

        // TODO: Load client and verify the parameters (client ID, client secret, redirect URI, scopes, etc.) check out
        // Error out if that's not the case
        //
        // Check the grant_type, let the client access it _somehow_
        //
        // Give the user some kind of "state" parameter, preferably typed, so they can store the authenticated user, and their
        // consent answer.

        let client_id = query.get("client_id").or_missing_param()?;
        let response_type = query.get("response_type").or_missing_param()?;
        if *response_type != "code" {
            debug!(?client_id, "response_type not set to \"code\"");
            return Err(Error::Unauthorized);
        }

        let scope = query.get("scope").or_missing_param()?;
        let redirect_uri = query.get("redirect_uri").or_missing_param()?;
        let state = query.get("state").map(|state| &**state);

        let client = self.client_extractor.extract(client_id, None).await?;

        if client.redirect_uri != *redirect_uri {
            debug!(?client_id, "redirect uri doesn't match");
            return Err(Error::Unauthorized);
        }

        let request_scopes = scope.split_whitespace().collect::<HashSet<_>>();
        let client_scopes = client
            .scopes
            .iter()
            .map(|scope| &**scope)
            .collect::<HashSet<_>>();

        if !request_scopes.is_subset(&client_scopes) {
            debug!(?client_id, "scopes aren't a subset");
            return Err(Error::Unauthorized);
        }

        Ok(Authorizer {
            auth_issuer: &self.auth_issuer,
            client,
            query,
            state,
        })
    }
}

pub struct Authorizer<'a, AI> {
    auth_issuer: &'a AI,
    client: Client<'a>,
    query: ParamStorage<&'a str, &'a str>,
    state: Option<&'a str>,
}

impl<'a, AI> Authorizer<'a, AI>
where
    AI: AuthIssuer,
{
    pub fn client(&self) -> &Client<'a> {
        &self.client
    }

    pub fn query(&self) -> &ParamStorage<&'a str, &'a str> {
        &self.query
    }

    pub async fn accept(self, user_id: AI::UserId, scopes: &[&str]) -> http::Response<()> {
        // TODO: Call an issuer to issue an access token for a particular user
        // Construct the callback url
        // Construct a redirect HTTP response UwU

        let code = self
            .auth_issuer
            .issue_code(user_id, self.client.client_id, scopes)
            .await
            .unwrap();

        todo!();
    }

    pub async fn deny(self) -> http::Response<()> {
        todo!();
    }
}
