use super::Store;
use crate::state::LoginState;
use fred::{clients::Pool, interfaces::KeysInterface};
use kitsune_error::Result;

const REDIS_PREFIX: &str = "OIDC-LOGIN-STATE";

#[derive(Clone)]
pub struct Redis {
    pool: Pool,
}

impl Redis {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    #[inline]
    fn format_key(key: &str) -> String {
        format!("{REDIS_PREFIX}:{key}")
    }
}

impl Store for Redis {
    async fn get_and_remove(&self, key: &str) -> Result<LoginState> {
        let raw_value: String = self.pool.getdel(Self::format_key(key)).await?;
        Ok(sonic_rs::from_slice(raw_value.as_bytes())?)
    }

    async fn set(&self, key: &str, value: LoginState) -> Result<()> {
        let raw_value = sonic_rs::to_string(&value)?;
        let () = self
            .pool
            .set(Self::format_key(key), raw_value, None, None, false)
            .await?;

        Ok(())
    }
}
