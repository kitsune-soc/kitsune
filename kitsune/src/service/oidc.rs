use crate::error::{OidcError, Result};
use http::Request;
use hyper::Body;
use kitsune_cache::{ArcCache, CacheBackend};
use kitsune_http_client::{Client, Error};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient},
    AccessTokenHash, AuthorizationCode, CsrfToken, HttpRequest, HttpResponse, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use url::Url;
use uuid::Uuid;

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
pub struct OAuth2Info {
    pub application_id: Uuid,
    pub scope: String,
    pub state: Option<String>,
}

#[derive(Debug)]
pub struct UserInfo {
    pub subject: String,
    pub username: String,
    pub email: String,
    pub oauth2: OAuth2Info,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct OAuth2LoginState {
    application_id: Uuid,
    scope: String,
    state: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct LoginState {
    nonce: Nonce,
    pkce_verifier: PkceCodeVerifier,
    oauth2: OAuth2LoginState,
}

impl Clone for LoginState {
    fn clone(&self) -> Self {
        Self {
            nonce: self.nonce.clone(),
            pkce_verifier: PkceCodeVerifier::new(self.pkce_verifier.secret().clone()),
            oauth2: self.oauth2.clone(),
        }
    }
}

#[derive(Clone, TypedBuilder)]
pub struct OidcService {
    client: CoreClient,
    login_state: ArcCache<String, LoginState>,
}

impl OidcService {
    pub async fn authorisation_url(
        &self,
        oauth2_application_id: Uuid,
        oauth2_scope: String,
        oauth2_state: Option<String>,
    ) -> Result<Url> {
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

        let verification_data = LoginState {
            nonce,
            pkce_verifier,
            oauth2: OAuth2LoginState {
                application_id: oauth2_application_id,
                scope: oauth2_scope,
                state: oauth2_state,
            },
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
        let LoginState {
            nonce,
            oauth2,
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
            subject: claims.subject().to_string(),
            username: claims
                .preferred_username()
                .ok_or(OidcError::MissingUsername)?
                .to_string(),
            email: claims.email().ok_or(OidcError::MissingEmail)?.to_string(),
            oauth2: OAuth2Info {
                application_id: oauth2.application_id,
                scope: oauth2.scope,
                state: oauth2.state,
            },
        })
    }
}
