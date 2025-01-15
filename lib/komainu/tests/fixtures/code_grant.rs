use crate::fixtures::{client_extractor::ClientExtractor, generate_secret, AuthorizationStorage};
use komainu::{
    code_grant::{self, AuthorizerExtractor},
    flow::pkce,
};
use std::borrow::Cow;

#[derive(Clone)]
pub struct Issuer {
    inner: AuthorizationStorage,
}

impl code_grant::Issuer for Issuer {
    type UserId = String;

    async fn issue_code(
        &self,
        user_id: Self::UserId,
        pre_authorization: komainu::AuthInstruction<'_, '_>,
    ) -> Result<String, code_grant::GrantError> {
        let code = generate_secret();
        let authorization = komainu::Authorization {
            code: Cow::Owned(code.clone()),
            client: pre_authorization.client.clone().into_owned(),
            pkce_payload: pre_authorization
                .pkce_payload
                .cloned()
                .map(pkce::Payload::into_owned),
            scopes: pre_authorization.scopes.clone(),
            user_id: Cow::Owned(user_id),
        };

        let mut guard = self.inner.lock().unwrap();
        guard.insert(code.clone(), authorization);

        Ok(code)
    }
}

#[inline]
pub fn extractor(
    storage: AuthorizationStorage,
    client_extractor: ClientExtractor,
) -> AuthorizerExtractor<Issuer, ClientExtractor> {
    AuthorizerExtractor::new(Issuer { inner: storage }, client_extractor)
}
