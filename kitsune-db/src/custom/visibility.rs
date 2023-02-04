use crate::entity::accounts;
use kitsune_type::ap::{helper::CcTo, Privacy};
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
    pub fn from_activitypub<O>(owner: &accounts::Model, obj: &O) -> Self
    where
        O: CcTo + Privacy,
    {
        if obj.is_public() {
            Self::Public
        } else if obj.is_unlisted() {
            Self::Unlisted
        } else if obj.to().contains(&owner.followers_url) {
            Self::FollowerOnly
        } else {
            Self::MentionOnly
        }
    }

    /// Convert the visibility into its JSON representation
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn json_repr(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
