use crate::entity::{accounts, accounts_followers};
use sea_orm::{Linked, RelationTrait};

/// Find accounts this account is following
pub struct Following;

impl Linked for Following {
    type FromEntity = accounts::Entity;
    type ToEntity = accounts::Entity;

    fn link(&self) -> Vec<sea_orm::LinkDef> {
        vec![
            accounts_followers::Relation::Accounts1.def().rev(),
            accounts_followers::Relation::Accounts2.def(),
        ]
    }
}
