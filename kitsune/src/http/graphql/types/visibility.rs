use async_graphql::Enum;
use kitsune_db::model::post::Visibility as DbVisibility;

#[derive(Clone, Copy, Debug, Enum, PartialEq, Eq, PartialOrd, Ord)]
pub enum Visibility {
    Public,
    Unlisted,
    FollowerOnly,
    MentionOnly,
}

impl From<Visibility> for DbVisibility {
    fn from(value: Visibility) -> Self {
        match value {
            Visibility::Public => DbVisibility::Public,
            Visibility::Unlisted => DbVisibility::Unlisted,
            Visibility::FollowerOnly => DbVisibility::FollowerOnly,
            Visibility::MentionOnly => DbVisibility::MentionOnly,
        }
    }
}

impl From<DbVisibility> for Visibility {
    fn from(value: DbVisibility) -> Self {
        match value {
            DbVisibility::Public => Visibility::Public,
            DbVisibility::Unlisted => Visibility::Unlisted,
            DbVisibility::FollowerOnly => Visibility::FollowerOnly,
            DbVisibility::MentionOnly => Visibility::MentionOnly,
        }
    }
}
