use crate::{
    error::Result,
    state::{
        store::{InMemory as InMemoryStore, Redis as RedisStore},
        LoginState, OAuth2LoginState, Store,
    },
};
use kitsune_config::oidc::{Configuration, StoreConfiguration};
use multiplex_pool::RoundRobinStrategy;
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    AccessTokenHash, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, RedirectUrl, Scope, TokenResponse,
};
use speedy_uuid::Uuid;
use url::Url;

pub use self::error::Error;

mod error;
mod state;

pub mod http;

const LOGIN_STATE_STORE_SIZE: u64 = 100;

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

#[derive(Clone)]
pub struct OidcService {
    client: CoreClient,
    login_state_store: self::state::AnyStore,
}

impl OidcService {
    #[inline]
    pub async fn initialise(config: &Configuration, redirect_uri: String) -> Result<Self> {
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(config.server_url.to_string())?,
            self::http::async_client,
        )
        .await?;

        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(config.client_id.to_string()),
            Some(ClientSecret::new(config.client_secret.to_string())),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri)?);

        let login_state_store = match config.store {
            StoreConfiguration::InMemory => InMemoryStore::new(LOGIN_STATE_STORE_SIZE).into(),
            StoreConfiguration::Redis(ref redis_config) => {
                let client = redis::Client::open(redis_config.url.as_str())?;
                let pool = multiplex_pool::Pool::from_producer(
                    || client.get_connection_manager(),
                    10,
                    RoundRobinStrategy::default(),
                )
                .await?;

                RedisStore::new(pool).into()
            }
        };

        Ok(Self {
            client,
            login_state_store,
        })
    }

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
        self.login_state_store
            .set(csrf_token.secret(), verification_data)
            .await?;

        Ok(auth_url)
    }

    pub async fn get_user_info(
        &self,
        state: String,
        authorization_code: String,
    ) -> Result<UserInfo> {
        let LoginState {
            nonce,
            oauth2,
            pkce_verifier,
        } = self.login_state_store.get_and_remove(&state).await?;

        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(authorization_code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(self::http::async_client)
            .await?;

        let id_token = token_response.id_token().ok_or(Error::MissingIdToken)?;
        let claims = id_token.claims(&self.client.id_token_verifier(), &nonce)?;

        if let Some(expected_hash) = claims.access_token_hash() {
            let actual_hash = AccessTokenHash::from_token(
                token_response.access_token(),
                &id_token.signing_alg()?,
            )?;

            if actual_hash != *expected_hash {
                return Err(Error::MismatchingHash);
            }
        }

        Ok(UserInfo {
            subject: claims.subject().to_string(),
            username: claims
                .preferred_username()
                .ok_or(Error::MissingUsername)?
                .to_string(),
            email: claims.email().ok_or(Error::MissingEmail)?.to_string(),
            oauth2: OAuth2Info {
                application_id: oauth2.application_id,
                scope: oauth2.scope,
                state: oauth2.state,
            },
        })
    }
}
