use super::account::Account;
use crate::schema::posts;
use diesel::{AsExpression, Associations, FromSqlRow, Identifiable, Insertable, Queryable};
use kitsune_type::{
    ap::{helper::CcTo, Privacy},
    mastodon::status::Visibility as MastodonVisibility,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Associations, Clone, Deserialize, Identifiable, Queryable, Serialize)]
#[diesel(belongs_to(Account), table_name = posts)]
pub struct Post {
    pub id: Uuid,
    pub account_id: Uuid,
    pub in_reply_to_id: Option<Uuid>,
    pub reposted_post_id: Option<Uuid>,
    pub is_sensitive: bool,
    pub subject: Option<String>,
    pub content: String,
    pub visibility: Visibility,
    pub is_local: bool,
    pub url: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost<'a> {
    pub id: Uuid,
    pub account_id: Uuid,
    pub in_reply_to_id: Option<Uuid>,
    pub reposted_post_id: Option<Uuid>,
    pub is_sensitive: bool,
    pub subject: Option<&'a str>,
    pub content: &'a str,
    pub visibility: Visibility,
    pub is_local: bool,
    pub url: &'a str,
    pub created_at: Option<OffsetDateTime>,
}

#[derive(
    AsExpression,
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    FromSqlRow,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[diesel(sql_type = diesel::sql_types::Integer)]
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
    pub fn from_activitypub<O>(owner: &Account, obj: &O) -> Option<Self>
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
