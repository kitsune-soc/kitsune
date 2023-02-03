use crate::entity::{accounts, favourites, posts};
use sea_orm::{Linked, RelationTrait};

/// Find the author of the favourited post
pub struct FavouritedPostAuthor;

impl Linked for FavouritedPostAuthor {
    type FromEntity = favourites::Entity;
    type ToEntity = accounts::Entity;

    fn link(&self) -> Vec<sea_orm::LinkDef> {
        vec![
            favourites::Relation::Posts.def(),
            posts::Relation::Accounts.def(),
        ]
    }
}
