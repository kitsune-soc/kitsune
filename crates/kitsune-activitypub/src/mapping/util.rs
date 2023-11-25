use super::State;
use kitsune_db::model::{account::Account, post::Visibility};
use kitsune_type::ap::PUBLIC_IDENTIFIER;

pub trait BaseToCc {
    fn base_to_cc(&self, state: State<'_>, account: &Account) -> (Vec<String>, Vec<String>);
}

impl BaseToCc for Visibility {
    #[inline]
    fn base_to_cc(&self, state: State<'_>, account: &Account) -> (Vec<String>, Vec<String>) {
        let followers_url = state.service.url.followers_url(account.id);

        match self {
            Visibility::Public => (vec![PUBLIC_IDENTIFIER.to_string()], vec![followers_url]),
            Visibility::Unlisted => (vec![], vec![PUBLIC_IDENTIFIER.to_string(), followers_url]),
            Visibility::FollowerOnly => (vec![followers_url], vec![]),
            Visibility::MentionOnly => (vec![], vec![]),
        }
    }
}
