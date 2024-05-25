use super::Store;
use crate::state::LoginState;
use fred::{clients::RedisPool, interfaces::KeysInterface};
use kitsune_error::Result;

const REDIS_PREFIX: &str = "OIDC-LOGIN-STATE";

#[derive(Clone)]
pub struct Redis {
    pool: RedisPool,
}

impl Redis {
    pub fn new(pool: RedisPool) -> Self {
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
        self.pool
            .set(Self::format_key(key), raw_value, None, None, false)
            .await?;

        Ok(())
    }
}
