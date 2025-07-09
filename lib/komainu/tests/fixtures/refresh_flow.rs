use crate::fixtures::{TokenStorage, generate_secret};
use komainu::flow::refresh;
use std::borrow::Cow;

#[derive(Clone)]
pub struct Issuer {
    inner: TokenStorage,
}

impl Issuer {
    pub fn new(inner: TokenStorage) -> Self {
        Self { inner }
    }
}

impl refresh::Issuer for Issuer {
    async fn issue_token(
        &self,
        client: &komainu::Client<'_>,
        refresh_token: &str,
    ) -> Result<komainu::flow::TokenResponse<'_>, komainu::flow::Error> {
        let mut value = self.inner.find_by_refresh(refresh_token).unwrap();

        assert_eq!(*client, value.authorization.client);

        self.inner.remove(&value.response.access_token);
        value.response.access_token = Cow::Owned(generate_secret());
        self.inner.insert(value.clone());

        Ok(value.response.into())
    }
}
