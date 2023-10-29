use super::Store;
use crate::{error::Result, state::LoginState};
use async_trait::async_trait;
use redis::AsyncCommands;

const REDIS_PREFIX: &str = "OIDC-LOGIN-STATE";

#[derive(Clone)]
pub struct Redis {
    pool: deadpool_redis::Pool,
}

impl Redis {
    pub fn new(pool: deadpool_redis::Pool) -> Self {
        Self { pool }
    }

    #[inline]
    fn format_key(key: &str) -> String {
        format!("{REDIS_PREFIX}:{key}")
    }
}

#[async_trait]
impl Store for Redis {
    async fn get_and_remove(&self, key: &str) -> Result<LoginState> {
        let mut conn = self.pool.get().await?;
        let raw_value: String = conn.get_del(Self::format_key(key)).await?;

        let mut raw_value = raw_value.into_bytes();
        Ok(simd_json::from_slice(&mut raw_value)?)
    }

    async fn set(&self, key: &str, value: LoginState) -> Result<()> {
        let raw_value = simd_json::to_string(&value)?;
        let mut conn = self.pool.get().await?;
        conn.set(Self::format_key(key), raw_value).await?;

        Ok(())
    }
}
