use chrono::{DateTime, FixedOffset, Utc};
use kitsune_db::{
    custom::Visibility,
    entity::{accounts, oauth2_access_tokens},
};
use kitsune_type::ap::PUBLIC_IDENTIFIER;

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

pub trait BaseToCc {
    fn base_to_cc(&self, account: &accounts::Model) -> (Vec<String>, Vec<String>);
}

impl BaseToCc for Visibility {
    fn base_to_cc(&self, account: &accounts::Model) -> (Vec<String>, Vec<String>) {
        match self {
            Visibility::Public => (
                vec![PUBLIC_IDENTIFIER.to_string(), account.followers_url.clone()],
                vec![],
            ),
            Visibility::Unlisted => (
                vec![account.followers_url.clone()],
                vec![PUBLIC_IDENTIFIER.to_string()],
            ),
            Visibility::FollowerOnly => (vec![account.followers_url.clone()], vec![]),
            Visibility::MentionOnly => (vec![], vec![]),
        }
    }
}
