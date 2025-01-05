use crate::{
    error::Error, flow::pkce, params::ParamStorage, scope::Scope, AuthInstruction, Client,
    ClientExtractor,
};
use std::{borrow::Cow, future::Future, ops::Deref, str::FromStr};
use strum::{AsRefStr, Display};
use thiserror::Error;

trait OptionExt<T> {
    fn or_invalid_request(self) -> Result<T, GrantError>;
}

impl<T> OptionExt<T> for Option<T> {
    #[inline]
    fn or_invalid_request(self) -> Result<T, GrantError> {
        self.ok_or(GrantError::InvalidRequest)
    }
}

#[derive(AsRefStr, Debug, Display, Error)]
#[strum(serialize_all = "snake_case")]
pub enum GrantError {
    InvalidRequest,
    UnauthorizedClient,
    AccessDenied,
    UnsupportedResponseType,
    InvalidScope,
    ServerError,
    TemporarilyUnavailable,
    Other(#[from] Error),
}

pub trait Issuer {
    type UserId;

    fn issue_code(
        &self,
        user_id: Self::UserId,
        pre_authorization: AuthInstruction<'_, '_>,
    ) -> impl Future<Output = Result<String, GrantError>> + Send;
}

pub struct AuthorizerExtractor<I, CE> {
    issuer: I,
    client_extractor: CE,
}

impl<I, CE> AuthorizerExtractor<I, CE>
where
    CE: ClientExtractor,
{
    pub fn new(issuer: I, client_extractor: CE) -> Self {
        Self {
            issuer,
            client_extractor,
        }
    }

    #[instrument(skip_all)]
    pub async fn extract_raw<'a>(
        &'a self,
        req: &'a crate::Request<'a>,
    ) -> Result<Authorizer<'a, I>, GrantError> {
        let client_id = req.query.get("client_id").or_invalid_request()?;
        let response_type = req.query.get("response_type").or_invalid_request()?;
        let scope = req.query.get("scope").map_or("", Deref::deref);
        let state = req.query.get("state").map(|state| &**state);

        let client = self.client_extractor.extract(client_id, None).await?;
        if let Some(redirect_uri) = req.query.get("redirect_uri") {
            if client.redirect_uri != *redirect_uri {
                debug!(?client_id, "redirect uri doesn't match");
                return Err(GrantError::AccessDenied);
            }
        }

        if *response_type != "code" {
            debug!(?client_id, "response_type not set to \"code\"");
            return Err(GrantError::AccessDenied);
        }

        let request_scopes = scope.parse().unwrap();

        // Check whether the client can actually perform the grant
        if !client.scopes.can_perform(&request_scopes) {
            debug!(?client_id, "client can't issue the requested scopes");
            return Err(GrantError::AccessDenied);
        }

        let pkce_payload = if let Some(challenge) = req.query.get("code_challenge") {
            let method = if let Some(method) = req.query.get("challenge_code_method") {
                pkce::Method::from_str(method).map_err(Error::query)?
            } else {
                pkce::Method::default()
            };

            Some(pkce::Payload {
                method,
                challenge: Cow::Borrowed(challenge),
            })
        } else {
            None
        };

        Ok(Authorizer {
            issuer: &self.issuer,
            client,
            pkce_payload,
            scope: request_scopes,
            query: &req.query,
            state,
        })
    }
}

macro_rules! return_err {
    ($result:expr) => {{
        match { $result } {
            Ok(val) => val,
            Err(err) => return err,
        }
    }};
}

pub struct Authorizer<'a, I> {
    issuer: &'a I,
    client: Client<'a>,
    pkce_payload: Option<pkce::Payload<'a>>,
    scope: Scope,
    query: &'a ParamStorage<Cow<'a, str>, Cow<'a, str>>,
    state: Option<&'a str>,
}

impl<'a, I> Authorizer<'a, I>
where
    I: Issuer,
{
    #[must_use]
    pub fn client(&self) -> &Client<'a> {
        &self.client
    }

    #[must_use]
    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    #[must_use]
    pub fn query(&self) -> &ParamStorage<Cow<'a, str>, Cow<'a, str>> {
        self.query
    }

    #[inline]
    fn build_response<U>(url: U) -> http::Response<()>
    where
        U: AsRef<str>,
    {
        http::Response::builder()
            .header(http::header::LOCATION, url.as_ref())
            .status(http::StatusCode::FOUND)
            .body(())
            .unwrap()
    }

    #[inline]
    fn redirect_uri(&self) -> Result<url::Url, http::Response<()>> {
        url::Url::parse(&self.client.redirect_uri).map_err(|error| {
            error!(?error, redirect_uri = ?self.client.redirect_uri, "invalid redirect uri");

            http::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(())
                .unwrap()
        })
    }

    #[inline]
    fn build_error_response(&self, error: &GrantError) -> http::Response<()> {
        let mut uri = return_err!(self.redirect_uri());
        uri.query_pairs_mut().append_pair("error", error.as_ref());
        Self::build_response(uri)
    }

    #[inline]
    #[instrument(skip_all)]
    pub async fn accept(self, user_id: I::UserId, scopes: &Scope) -> http::Response<()> {
        let pre_authorization = AuthInstruction {
            client: &self.client,
            scopes,
            pkce_payload: self.pkce_payload.as_ref(),
        };

        let code = match self.issuer.issue_code(user_id, pre_authorization).await {
            Ok(code) => code,
            Err(error) => {
                debug!(?error, "failed to issue code");
                return self.build_error_response(&GrantError::TemporarilyUnavailable);
            }
        };

        let mut url = return_err!(self.redirect_uri());
        url.query_pairs_mut().append_pair("code", &code);

        if let Some(state) = self.state {
            url.query_pairs_mut().append_pair("state", state);
        }

        Self::build_response(url)
    }

    #[inline]
    #[must_use]
    #[instrument(skip_all)]
    pub fn deny(self) -> http::Response<()> {
        self.build_error_response(&GrantError::AccessDenied)
    }
}
