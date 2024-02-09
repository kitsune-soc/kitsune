use super::Store;
use crate::{error::Result, state::LoginState};
use redis::{aio::ConnectionManager, AsyncCommands};

const REDIS_PREFIX: &str = "OIDC-LOGIN-STATE";

#[derive(Clone)]
pub struct Redis {
    pool: multiplex_pool::Pool<ConnectionManager>,
}

impl Redis {
    pub fn new(pool: multiplex_pool::Pool<ConnectionManager>) -> Self {
        Self { pool }
    }

    #[inline]
    fn format_key(key: &str) -> String {
        format!("{REDIS_PREFIX}:{key}")
    }
}

impl Store for Redis {
    async fn get_and_remove(&self, key: &str) -> Result<LoginState> {
        let mut conn = self.pool.get();
        let raw_value: String = conn.get_del(Self::format_key(key)).await?;

        let mut raw_value = raw_value.into_bytes();
        Ok(simd_json::from_slice(&mut raw_value)?)
    }

    async fn set(&self, key: &str, value: LoginState) -> Result<()> {
        let raw_value = simd_json::to_string(&value)?;
        let mut conn = self.pool.get();
        conn.set(Self::format_key(key), raw_value).await?;

        Ok(())
    }
}
