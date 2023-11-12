use crate::state::State;
use iso8601_timestamp::Timestamp;
use kitsune_db::model::{account::Account, oauth2::access_token::AccessToken, post::Visibility};
use kitsune_type::ap::PUBLIC_IDENTIFIER;
use pulldown_cmark::{html, Options, Parser};

#[inline]
#[must_use]
pub fn process_markdown(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::all());
    let mut buf = String::new();
    html::push_html(&mut buf, parser);
    buf
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
    fn base_to_cc(&self, state: &State, account: &Account) -> (Vec<String>, Vec<String>);
}

impl BaseToCc for Visibility {
    #[inline]
    fn base_to_cc(&self, state: &State, account: &Account) -> (Vec<String>, Vec<String>) {
        let followers_url = state.service.url.followers_url(account.id);

        match self {
            Visibility::Public => (vec![PUBLIC_IDENTIFIER.to_string()], vec![followers_url]),
            Visibility::Unlisted => (vec![], vec![PUBLIC_IDENTIFIER.to_string(), followers_url]),
            Visibility::FollowerOnly => (vec![followers_url], vec![]),
            Visibility::MentionOnly => (vec![], vec![]),
        }
    }
}
