use crate::entity::{accounts, posts, posts_mentions};
use sea_orm::{Linked, RelationTrait};

/// Find posts mentioned by an account
pub struct MentionedAccounts;

impl Linked for MentionedAccounts {
    type FromEntity = posts::Entity;
    type ToEntity = accounts::Entity;

    fn link(&self) -> Vec<sea_orm::LinkDef> {
        vec![
            posts_mentions::Relation::Posts.def().rev(),
            posts_mentions::Relation::Accounts.def(),
        ]
    }
}
