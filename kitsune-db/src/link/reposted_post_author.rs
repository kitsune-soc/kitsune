use crate::entity::{accounts, posts, reposts};
use sea_orm::{Linked, RelationTrait};

/// Find the author of the reposted post
pub struct RepostedPostAuthor;

impl Linked for RepostedPostAuthor {
    type FromEntity = reposts::Entity;
    type ToEntity = accounts::Entity;

    fn link(&self) -> Vec<sea_orm::LinkDef> {
        vec![
            reposts::Relation::Posts.def(),
            posts::Relation::Accounts.def(),
        ]
    }
}
