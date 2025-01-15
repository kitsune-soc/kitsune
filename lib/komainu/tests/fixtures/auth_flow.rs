use crate::fixtures::{AuthorizationStorage, TokenStorage};
use komainu::flow;

const EXPIRES_IN: u64 = 1337;

#[derive(Clone)]
pub struct Issuer {
    inner: AuthorizationStorage,
    tokens: TokenStorage,
}

impl Issuer {
    pub fn new(inner: AuthorizationStorage, tokens: TokenStorage) -> Self {
        Self { inner, tokens }
    }
}

impl flow::authorization::Issuer for Issuer {
    async fn load_authorization(
        &self,
        auth_code: &str,
    ) -> Result<Option<komainu::Authorization<'_>>, flow::Error> {
        let guard = self.inner.lock().unwrap();
        Ok(guard.get(auth_code).cloned())
    }

    async fn issue_token(
        &self,
        authorization: &komainu::Authorization<'_>,
    ) -> Result<flow::TokenResponse<'_>, flow::Error> {
        let value = self.tokens.generate(authorization.clone(), EXPIRES_IN);
        Ok(value.response.into())
    }
}
