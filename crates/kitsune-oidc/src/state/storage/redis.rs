use redis::AsyncCommands;

use super::Storage;
use crate::{error::Result, state::LoginState};

const REDIS_NAMESPACE: &str = "OIDC-LOGIN-STATE";

pub struct Redis {
    pool: deadpool_redis::Pool,
}

impl Storage for Redis {
    async fn get_and_remove(&self, key: &str) -> Result<LoginState> {
        let conn = self.pool.get().await?;
        let raw_data: String = conn.get_del(key).await?
    }

    async fn set(&self, key: &str, value: LoginState) -> Result<()> {
        todo!()
    }
}
