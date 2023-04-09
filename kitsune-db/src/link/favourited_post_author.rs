use crate::entity::{accounts, posts, posts_favourites};
use sea_orm::{Linked, RelationTrait};

/// Find the author of the favourited post
pub struct FavouritedPostAuthor;

impl Linked for FavouritedPostAuthor {
    type FromEntity = posts_favourites::Entity;
    type ToEntity = accounts::Entity;

    fn link(&self) -> Vec<sea_orm::LinkDef> {
        vec![
            posts_favourites::Relation::Posts.def(),
            posts::Relation::Accounts.def(),
        ]
    }
}
