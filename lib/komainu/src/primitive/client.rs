use crate::scope::Scope;
use std::{borrow::Cow, fmt};
use subtle::ConstantTimeEq;

#[derive(Clone)]
pub struct Client<'a> {
    pub client_id: Cow<'a, str>,
    pub client_secret: Cow<'a, str>,
    pub scopes: Scope,
    pub redirect_uri: Cow<'a, str>,
}

impl Client<'_> {
    #[must_use]
    pub fn into_owned(self) -> Client<'static> {
        Client {
            client_id: self.client_id.into_owned().into(),
            client_secret: self.client_secret.into_owned().into(),
            scopes: self.scopes,
            redirect_uri: self.redirect_uri.into_owned().into(),
        }
    }
}

impl fmt::Debug for Client<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .field("client_id", &self.client_id)
            .field("client_secret", &"[redacted]")
            .field("scopes", &self.scopes)
            .field("redirect_uri", &self.redirect_uri)
            .finish_non_exhaustive()
    }
}

impl PartialEq for Client<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let client_id_l = self.client_id.as_bytes();
        let client_id_r = other.client_id.as_bytes();

        let client_secret_l = self.client_secret.as_bytes();
        let client_secret_r = other.client_secret.as_bytes();

        (client_id_l.ct_eq(client_id_r) & client_secret_l.ct_eq(client_secret_r)).into()
    }
}
