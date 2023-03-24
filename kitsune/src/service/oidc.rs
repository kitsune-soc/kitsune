use crate::{
    cache::ArcCache,
    error::{OidcError, Result},
};
use derive_builder::Builder;
use http::Request;
use hyper::Body;
use kitsune_http_client::{Client, Error};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient},
    AccessTokenHash, AuthorizationCode, CsrfToken, HttpRequest, HttpResponse, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[allow(clippy::missing_panics_doc)]
pub async fn async_client(req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut request = Request::builder().method(req.method).uri(req.url.as_str());
    *request.headers_mut().unwrap() = req.headers;
    let request = request.body(Body::from(req.body)).unwrap();
    let response = Client::default().execute(request).await?;

    Ok(HttpResponse {
        status_code: response.status(),
        headers: response.headers().clone(),
        body: response.bytes().await?.to_vec(),
    })
}

#[derive(Debug)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
}

#[derive(Deserialize, Serialize)]
pub struct VerificationData {
    nonce: Nonce,
    pkce_verifier: PkceCodeVerifier,
}

impl Clone for VerificationData {
    fn clone(&self) -> Self {
        Self {
            nonce: self.nonce.clone(),
            pkce_verifier: PkceCodeVerifier::new(self.pkce_verifier.secret().clone()),
        }
    }
}

#[derive(Builder, Clone)]
pub struct OidcService {
    client: CoreClient,
    login_state: ArcCache<String, VerificationData>,
}

impl OidcService {
    #[must_use]
    pub fn builder() -> OidcServiceBuilder {
        OidcServiceBuilder::default()
    }

    pub async fn authorisation_url(&self) -> Result<Url> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, csrf_token, nonce) = self
            .client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".into()))
            .add_scope(Scope::new("profile".into()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        let verification_data = VerificationData {
            nonce,
            pkce_verifier,
        };
        self.login_state
            .set(csrf_token.secret(), &verification_data)
            .await?;

        Ok(auth_url)
    }

    pub async fn get_user_info(
        &self,
        state: String,
        authorization_code: String,
    ) -> Result<UserInfo, OidcError> {
        let VerificationData {
            nonce,
            pkce_verifier,
        } = self
            .login_state
            .get(&state)
            .await?
            .ok_or(OidcError::UnknownCsrfToken)?;
        self.login_state.delete(&state).await?;

        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(authorization_code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_client)
            .await?;

        let id_token = token_response.id_token().ok_or(OidcError::MissingIdToken)?;
        let claims = id_token.claims(&self.client.id_token_verifier(), &nonce)?;

        if let Some(expected_hash) = claims.access_token_hash() {
            let actual_hash = AccessTokenHash::from_token(
                token_response.access_token(),
                &id_token.signing_alg()?,
            )?;

            if actual_hash != *expected_hash {
                return Err(OidcError::MismatchingHash);
            }
        }

        Ok(UserInfo {
            username: claims
                .preferred_username()
                .ok_or(OidcError::MissingUsername)?
                .to_string(),
            email: claims.email().ok_or(OidcError::MissingEmail)?.to_string(),
        })
    }
}
