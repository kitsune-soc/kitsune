use crate::state::Zustand;
use kitsune_db::model::{account::Account, oauth2::access_token::AccessToken, post::Visibility};
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

impl AccessTokenTtl for AccessToken {
    fn ttl(&self) -> time::Duration {
        self.expired_at - OffsetDateTime::now_utc()
    }
}

pub trait BaseToCc {
    fn base_to_cc(&self, state: &Zustand, account: &Account) -> (Vec<String>, Vec<String>);
}

impl BaseToCc for Visibility {
    fn base_to_cc(&self, state: &Zustand, account: &Account) -> (Vec<String>, Vec<String>) {
        let followers_url = state.service.url.followers_url(account.id);

        match self {
            Visibility::Public => (vec![PUBLIC_IDENTIFIER.to_string()], vec![followers_url]),
            Visibility::Unlisted => (vec![], vec![PUBLIC_IDENTIFIER.to_string(), followers_url]),
            Visibility::FollowerOnly => (vec![followers_url], vec![]),
            Visibility::MentionOnly => (vec![], vec![]),
        }
    }
}

/// Wrapper around the [`tokio::try_join`] macro but it passes each future through
/// a small no-op function that gives the compiler some trait bound hints
#[macro_export]
macro_rules! try_join {
    ($($try_future:expr),+$(,)?) => {{
        /// Hack around the bogus "higher-ranked lifetime" errors
        ///
        /// Asserts `Send` bounds via its type signature and helps the compiler a little bit with it
        #[allow(clippy::inline_always)] // This is literally an empty function, only used for its type signature. 0 runtime implications.
        #[inline(always)]
        fn assert_future_send<O>(
            fut: impl ::core::future::Future<Output = O> + Send,
        ) -> impl ::core::future::Future<Output = O> + Send {
            fut
        }

        ::tokio::try_join!(
            $( assert_future_send($try_future) ),+
        )
    }};
}
