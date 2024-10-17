use openidconnect::{Nonce, PkceCodeVerifier};
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

pub use self::store::{AnyStore, Store};

pub mod store;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OAuth2LoginState {
    pub application_id: Uuid,
    pub scope: String,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct LoginState {
    pub nonce: Nonce,
    pub pkce_verifier: PkceCodeVerifier,
    pub oauth2: OAuth2LoginState,
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
