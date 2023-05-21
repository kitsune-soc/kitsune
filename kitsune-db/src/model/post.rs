use super::account::Account;
use crate::{error::EnumConversionError, impl_columns, schema::posts};
use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::Integer,
    AsExpression, Associations, FromSqlRow, Identifiable, Insertable, Queryable,
};
use kitsune_type::{
    ap::{helper::CcTo, Privacy},
    mastodon::status::Visibility as MastodonVisibility,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
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

impl_columns! {
    Post => (
        posts::id,
        posts::account_id,
        posts::in_reply_to_id,
        posts::reposted_post_id,
        posts::is_sensitive,
        posts::subject,
        posts::content,
        posts::visibility,
        posts::is_local,
        posts::url,
        posts::created_at,
        posts::updated_at,
    )
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
    FromPrimitive,
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

impl FromSql<Integer, Pg> for Visibility
where
    i32: FromSql<Integer, Pg>,
{
    fn from_sql(bytes: RawValue<'_, Pg>) -> deserialize::Result<Self> {
        let value = i32::from_sql(bytes)?;
        Ok(Self::from_i32(value).ok_or(EnumConversionError(value))?)
    }
}

impl ToSql<Integer, Pg> for Visibility
where
    i32: ToSql<Integer, Pg>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <i32 as ToSql<Integer, _>>::to_sql(&(*self as i32), &mut out.reborrow())
    }
}
