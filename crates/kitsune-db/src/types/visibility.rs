use crate::{error::EnumConversionError, model::AccountsActivitypub};
use diesel::{
    AsExpression, FromSqlRow,
    backend::Backend,
    deserialize::{self, FromSql},
    serialize::{self, Output, ToSql},
    sql_types::Integer,
};
use kitsune_type::{
    ap::{Privacy, helper::CcTo},
    mastodon::status::Visibility as MastodonVisibility,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::ptr;

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
#[diesel(sql_type = diesel::sql_types::Integer)]
#[repr(i32)]
#[serde(rename_all = "camelCase")]
/// Post visibility
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
    pub fn from_activitypub<O>(owner: &AccountsActivitypub, obj: &O) -> Option<Self>
    where
        O: CcTo + Privacy,
    {
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

impl<Db> FromSql<Integer, Db> for Visibility
where
    i32: FromSql<Integer, Db>,
    Db: Backend,
{
    fn from_sql(bytes: <Db as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let value = i32::from_sql(bytes)?;
        Ok(Self::from_i32(value).ok_or(EnumConversionError(value))?)
    }
}

impl<Db> ToSql<Integer, Db> for Visibility
where
    i32: ToSql<Integer, Db>,
    Db: Backend,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Db>) -> serialize::Result {
        // SAFETY: We have a `#[repr(i32)]` over the enum, so the representations are really the same
        #[allow(unsafe_code)]
        ToSql::to_sql(unsafe { &*ptr::from_ref(self).cast::<i32>() }, out)
    }
}
