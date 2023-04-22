use crate::entity::accounts;
use kitsune_type::{
    ap::{helper::CcTo, Privacy},
    mastodon::status::Visibility as MastodonVisibility,
};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    EnumIter,
    DeriveActiveEnum,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
#[serde(rename_all = "camelCase")]
/// Post visiblity
pub enum Visibility {
    /// Post is public and can be seen and interacted with by anyone
    ///
    /// This is the default
    #[default]
    Public = 0,
    /// The post will not appear on the local and federated timelines but still can be seen and interacted with by anyone
    Unlisted = 1,
    /// The post is only visible and can only be interacted with by the followers of that person
    FollowerOnly = 2,
    /// The post is de-facto private and can only be seen and interacted with by the people explicitly mentioned in the post
    MentionOnly = 3,
}

impl Visibility {
    /// Determine the visibility for some ActivityPub object
    ///
    /// Returns none in case the account is local
    pub fn from_activitypub<O>(owner: &accounts::Model, obj: &O) -> Option<Self>
    where
        O: CcTo + Privacy,
    {
        if owner.local {
            return None;
        }

        let visibility = if obj.is_public() {
            Self::Public
        } else if obj.is_unlisted() {
            Self::Unlisted
        } else if obj
            .to()
            .iter()
            .any(|item| owner.followers_url.as_ref() == Some(item))
        {
            Self::FollowerOnly
        } else {
            Self::MentionOnly
        };
        Some(visibility)
    }
}

impl From<MastodonVisibility> for Visibility {
    fn from(value: MastodonVisibility) -> Self {
        match value {
            MastodonVisibility::Public => Self::Public,
            MastodonVisibility::Unlisted => Self::Unlisted,
            MastodonVisibility::Private => Self::FollowerOnly,
            MastodonVisibility::Direct => Self::MentionOnly,
        }
    }
}

impl From<Visibility> for MastodonVisibility {
    fn from(value: Visibility) -> Self {
        match value {
            Visibility::Public => Self::Public,
            Visibility::Unlisted => Self::Unlisted,
            Visibility::FollowerOnly => Self::Private,
            Visibility::MentionOnly => Self::Direct,
        }
    }
}
