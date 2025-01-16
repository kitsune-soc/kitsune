use self::client_extractor::ClientExtractor;
use komainu::{
    code_grant::AuthorizerExtractor,
    flow::{SuccessTokenResponse, TokenType},
};
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub mod auth_flow;
pub mod client_extractor;
pub mod code_grant;
pub mod refresh_flow;

#[allow(clippy::unreadable_literal)]
pub const RNG_SEED: u64 = 0xBADD1E;

#[inline]
pub fn generate_secret() -> String {
    (0..16).map(|_| fastrand::lowercase()).collect()
}

#[derive(Clone, Default)]
pub struct AuthorizationStorage {
    inner: Arc<Mutex<HashMap<String, komainu::Authorization<'static>>>>,
}

impl AuthorizationStorage {
    pub fn insert(&self, auth: komainu::Authorization<'_>) {
        let auth = auth.into_owned();
        let mut guard = self.inner.lock().unwrap();
        guard.insert(auth.code.clone().into_owned(), auth);
    }

    pub fn get(&self, code: &str) -> Option<komainu::Authorization<'static>> {
        let guard = self.inner.lock().unwrap();
        guard.get(code).cloned()
    }
}

#[derive(Clone)]
pub struct TokenValue {
    pub authorization: komainu::Authorization<'static>,
    pub response: SuccessTokenResponse<'static>,
}

#[derive(Clone, Default)]
pub struct TokenStorage {
    inner: Arc<Mutex<HashMap<String, TokenValue>>>,
}

impl TokenStorage {
    pub fn generate(
        &self,
        authorization: komainu::Authorization<'_>,
        expires_in: u64,
    ) -> TokenValue {
        let token = generate_secret();
        let refresh_token = generate_secret();

        let response = SuccessTokenResponse {
            access_token: Cow::Owned(token.clone()),
            refresh_token: Cow::Owned(refresh_token),
            token_type: TokenType::Bearer,
            expires_in,
        };
        let value = TokenValue {
            authorization: authorization.into_owned(),
            response: response.clone(),
        };

        let mut guard = self.inner.lock().unwrap();
        guard.insert(token, value.clone());

        value
    }

    pub fn insert(&self, value: TokenValue) {
        let mut guard = self.inner.lock().unwrap();
        guard.insert(value.response.access_token.clone().into_owned(), value);
    }

    pub fn get(&self, token: &str) -> Option<TokenValue> {
        let guard = self.inner.lock().unwrap();
        guard.get(token).cloned()
    }

    pub fn remove(&self, token: &str) {
        let mut guard = self.inner.lock().unwrap();
        guard.remove(token);
    }

    pub fn find_by_refresh(&self, refresh_token: &str) -> Option<TokenValue> {
        let guard = self.inner.lock().unwrap();
        let mut value = guard
            .values()
            .filter(|value| value.response.refresh_token == refresh_token)
            .cloned();

        value.next()
    }
}

#[derive(Clone)]
pub struct Fixture {
    pub auth_flow: self::auth_flow::Issuer,
    pub auth_storage: AuthorizationStorage,
    pub client_extractor: ClientExtractor,
    pub code_grant: Arc<AuthorizerExtractor<self::code_grant::Issuer, ClientExtractor>>,
    pub refresh_flow: self::refresh_flow::Issuer,
    pub token_storage: TokenStorage,
}

impl Fixture {
    pub fn generate() -> Self {
        let auth_storage = AuthorizationStorage::default();
        let token_storage = TokenStorage::default();

        let client_extractor = ClientExtractor::default();
        let code_grant =
            self::code_grant::extractor(auth_storage.clone(), client_extractor.clone());

        Self {
            auth_flow: self::auth_flow::Issuer::new(auth_storage.clone(), token_storage.clone()),
            auth_storage,
            client_extractor,
            code_grant: Arc::new(code_grant),
            refresh_flow: self::refresh_flow::Issuer::new(token_storage.clone()),
            token_storage,
        }
    }
}
