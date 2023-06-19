use crate::state::Zustand;
use base64_simd::AsOut;
use hex_simd::AsciiCase;
use iso8601_timestamp::Timestamp;
use kitsune_db::model::{account::Account, oauth2::access_token::AccessToken, post::Visibility};
use kitsune_type::ap::PUBLIC_IDENTIFIER;
use uuid::Uuid;

const TOKEN_LENGTH: usize = 32;

#[inline]
#[must_use]
pub fn generate_secret() -> String {
    let token_data: [u8; TOKEN_LENGTH] = rand::random();
    let mut buf = [0_u8; TOKEN_LENGTH * 2];
    (*hex_simd::encode_as_str(&token_data, buf.as_mut_slice().as_out(), AsciiCase::Lower))
        .to_string()
}

#[inline]
#[must_use]
pub fn timestamp_to_uuid(timestamp: Timestamp) -> Uuid {
    let seconds = timestamp
        .duration_since(Timestamp::UNIX_EPOCH)
        .whole_seconds();
    let uuid_timestamp =
        uuid::Timestamp::from_unix(uuid::NoContext, seconds as u64, timestamp.nanosecond());

    Uuid::new_v7(uuid_timestamp)
}

pub trait AccessTokenTtl {
    fn ttl(&self) -> time::Duration;
}

impl AccessTokenTtl for AccessToken {
    #[inline]
    fn ttl(&self) -> time::Duration {
        *self.expires_at - *Timestamp::now_utc()
    }
}

pub trait BaseToCc {
    fn base_to_cc(&self, state: &Zustand, account: &Account) -> (Vec<String>, Vec<String>);
}

impl BaseToCc for Visibility {
    #[inline]
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
        /// Hack around the [bogus "higher-ranked lifetime" errors](https://github.com/rust-lang/rust/issues/102211)
        ///
        /// Asserts `Send` bounds via its type signature and helps the compiler a little bit with proving the bound
        #[inline(always)]
        fn assert_send<O>(
            fut: impl ::core::future::Future<Output = O> + Send,
        ) -> impl ::core::future::Future<Output = O> + Send {
            fut
        }

        ::tokio::try_join!(
            $( assert_send($try_future) ),+
        )
    }};
}
