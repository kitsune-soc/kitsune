use chrono::{DateTime, FixedOffset, Utc};
use kitsune_db::entity::oauth2_access_tokens;

#[must_use]
pub fn generate_secret() -> String {
    let token_data: [u8; 32] = rand::random();
    hex::encode(token_data)
}

pub trait AccessTokenTtl {
    fn ttl(&self) -> chrono::Duration;
}

impl AccessTokenTtl for oauth2_access_tokens::Model {
    fn ttl(&self) -> chrono::Duration {
        self.expired_at - DateTime::<FixedOffset>::from(Utc::now())
    }
}
