use kitsune_db::{
    custom::Visibility,
    entity::{accounts, oauth2_access_tokens},
};
use kitsune_type::ap::PUBLIC_IDENTIFIER;
use time::OffsetDateTime;

#[must_use]
pub fn generate_secret() -> String {
    let token_data: [u8; 32] = rand::random();
    hex::encode(token_data)
}

pub trait AccessTokenTtl {
    fn ttl(&self) -> time::Duration;
}

impl AccessTokenTtl for oauth2_access_tokens::Model {
    fn ttl(&self) -> time::Duration {
        self.expired_at - OffsetDateTime::now_utc()
    }
}

pub trait BaseToCc {
    fn base_to_cc(&self, account: &accounts::Model) -> (Vec<String>, Vec<String>);
}

impl BaseToCc for Visibility {
    fn base_to_cc(&self, account: &accounts::Model) -> (Vec<String>, Vec<String>) {
        match self {
            Visibility::Public => (
                vec![PUBLIC_IDENTIFIER.to_string()],
                vec![account.followers_url.clone()],
            ),
            Visibility::Unlisted => (
                vec![],
                vec![PUBLIC_IDENTIFIER.to_string(), account.followers_url.clone()],
            ),
            Visibility::FollowerOnly => (vec![account.followers_url.clone()], vec![]),
            Visibility::MentionOnly => (vec![], vec![]),
        }
    }
}
