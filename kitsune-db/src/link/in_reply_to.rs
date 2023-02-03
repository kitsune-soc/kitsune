use crate::entity::posts;
use sea_orm::{Linked, RelationDef, RelationTrait};

/// Find the post this post is replying to
pub struct InReplyTo;

impl Linked for InReplyTo {
    type FromEntity = posts::Entity;
    type ToEntity = posts::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![posts::Relation::SelfRef.def()]
    }
}
