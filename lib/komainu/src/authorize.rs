use crate::{
    error::{Error, Result},
    flow::{PkceMethod, PkcePayload},
    params::ParamStorage,
    Authorization, Client, ClientExtractor, OAuthError, OptionExt, PreAuthorization,
};
use std::{borrow::{Cow, Borrow}, collections::HashSet, future::Future, str::FromStr};

pub trait Issuer {
    type UserId;

    fn issue_code(
        &self,
        user_id: Self::UserId,
        pre_authorization: PreAuthorization<'_>,
    ) -> impl Future<Output = Result<String>> + Send;
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
    pub async fn extract<'a>(&'a self, req: &'a http::Request<()>) -> Result<Authorizer<'a, I>> {
        let query: ParamStorage<&str, &str> =
            serde_urlencoded::from_str(req.uri().query().or_missing_param()?)
                .map_err(Error::query)?;

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
            .map(|scope| scope.borrow())
            .collect::<HashSet<_>>();

        if !request_scopes.is_subset(&client_scopes) {
            debug!(?client_id, "scopes aren't a subset");
            return Err(Error::Unauthorized);
        }

        let pkce_payload = if let Some(challenge) = query.get("code_challenge") {
            let method = if let Some(method) = query.get("challenge_code_method") {
                PkceMethod::from_str(*method).map_err(Error::query)?
            } else {
                PkceMethod::default()
            };

            Some(PkcePayload { method, challenge: Cow::Borrowed(challenge) })
        } else {
            None
        };

        Ok(Authorizer {
            issuer: &self.issuer,
            client,
            pkce_payload,
            query,
            state,
        })
    }
}

pub struct Authorizer<'a, I> {
    issuer: &'a I,
    client: Client<'a>,
    pkce_payload: Option<PkcePayload<'a>>,
    query: ParamStorage<&'a str, &'a str>,
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
    pub fn query(&self) -> &ParamStorage<&'a str, &'a str> {
        &self.query
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
    #[instrument(skip_all)]
    pub async fn accept<'a>(self, user_id: I::UserId, scopes: &'a [&'a str]) -> http::Response<()> {
        let pre_authorization = PreAuthorization {
            client: &self.client,
            scopes,
            pkce_payload: self.pkce_payload.as_ref(),
        };

        let code = self
            .issuer
            .issue_code(user_id, pre_authorization)
            .await
            .unwrap();

        let mut url = url::Url::parse(&self.client.redirect_uri).unwrap();
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
        let mut url = url::Url::parse(&self.client.redirect_uri).unwrap();
        url.query_pairs_mut()
            .append_pair("error", OAuthError::AccessDenied.as_ref());

        Self::build_response(url)
    }
}
