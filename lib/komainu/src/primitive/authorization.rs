use crate::{flow::pkce, primitive::Client, scope::Scope};
use std::borrow::Cow;

#[derive(Clone)]
pub struct Authorization<'a> {
    pub code: Cow<'a, str>,
    pub client: Client<'a>,
    pub pkce_payload: Option<pkce::Payload<'a>>,
    pub scopes: Scope,
    pub user_id: Cow<'a, str>,
}

impl Authorization<'_> {
    pub fn into_owned(self) -> Authorization<'static> {
        Authorization {
            code: self.code.into_owned().into(),
            client: self.client.into_owned(),
            pkce_payload: self.pkce_payload.map(pkce::Payload::into_owned),
            scopes: self.scopes,
            user_id: self.user_id.into_owned().into(),
        }
    }
}
