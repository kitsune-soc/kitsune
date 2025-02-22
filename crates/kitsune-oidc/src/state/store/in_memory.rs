use super::Store;
use crate::state::LoginState;
use kitsune_error::{ErrorType, Result, kitsune_error};
use moka::sync::Cache;

#[derive(Clone)]
pub struct InMemory {
    inner: Cache<String, LoginState>,
}

impl InMemory {
    pub fn new(size: u64) -> Self {
        Self {
            inner: Cache::new(size),
        }
    }
}

impl Store for InMemory {
    async fn get_and_remove(&self, key: &str) -> Result<LoginState> {
        self.inner
            .remove(key)
            .ok_or_else(|| kitsune_error!(type = ErrorType::BadRequest, "missing login state"))
    }

    async fn set(&self, key: &str, value: LoginState) -> Result<()> {
        self.inner.insert(key.to_string(), value);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::InMemory;
    use crate::state::{LoginState, OAuth2LoginState, Store};
    use oauth2::PkceCodeVerifier;
    use openidconnect::Nonce;
    use speedy_uuid::Uuid;

    #[tokio::test]
    async fn basic_ops() {
        let val = LoginState {
            nonce: Nonce::new_random(),
            pkce_verifier: PkceCodeVerifier::new("test".into()),
            oauth2: OAuth2LoginState {
                application_id: Uuid::now_v7(),
                scope: "owo".into(),
                state: None,
            },
        };

        let in_memory = InMemory::new(10);
        in_memory.set("uwu", val.clone()).await.unwrap();
        let got_val = in_memory.get_and_remove("uwu").await.unwrap();
        assert_eq!(got_val, val);
    }

    #[tokio::test]
    async fn limits_size() {
        let val = LoginState {
            nonce: Nonce::new_random(),
            pkce_verifier: PkceCodeVerifier::new("test".into()),
            oauth2: OAuth2LoginState {
                application_id: Uuid::now_v7(),
                scope: "owo".into(),
                state: None,
            },
        };

        let in_memory = InMemory::new(2);
        in_memory.set("owo", val.clone()).await.unwrap();
        in_memory.set("uwu", val.clone()).await.unwrap();
        in_memory.set("ùwú", val.clone()).await.unwrap();

        in_memory.inner.run_pending_tasks();

        assert_eq!(in_memory.inner.entry_count(), 2);
    }
}
