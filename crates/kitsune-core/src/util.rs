use crate::state::State;
use iso8601_timestamp::Timestamp;
use kitsune_db::model::{account::Account, oauth2::access_token::AccessToken, post::Visibility};
use kitsune_type::ap::PUBLIC_IDENTIFIER;

pub trait AccessTokenTtl {
    fn ttl(&self) -> time::Duration;
}

impl AccessTokenTtl for AccessToken {
    #[inline]
    fn ttl(&self) -> time::Duration {
        *self.expires_at - *Timestamp::now_utc()
    }
}
