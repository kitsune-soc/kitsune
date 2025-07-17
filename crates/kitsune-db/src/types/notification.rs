use crate::error::EnumConversionError;
use diesel::{
    AsExpression, FromSqlRow,
    backend::Backend,
    deserialize::{self, FromSql},
    serialize::{self, Output, ToSql},
    sql_types::SmallInt,
};
use kitsune_type::mastodon::notification::NotificationType as MastodonNotification;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::ptr;

#[derive(
    AsExpression,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    FromPrimitive,
    FromSqlRow,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[diesel(sql_type = diesel::sql_types::SmallInt)]
#[repr(i16)]
/// ActivityPub actor types
pub enum NotificationType {
    /// User mentioned
    Mention = 0,

    /// New status posted from a followed account
    Post = 1,

    /// User reposted a status
    Repost = 2,

    /// New follow
    Follow = 3,

    /// New follow request
    FollowRequest = 4,

    /// User favourited a status
    Favourite = 5,

    /// Reposted status has been updated
    PostUpdate = 6,
}

impl From<MastodonNotification> for NotificationType {
    fn from(value: MastodonNotification) -> Self {
        match value {
            MastodonNotification::Mention => Self::Mention,
            MastodonNotification::Status => Self::Post,
            MastodonNotification::Reblog => Self::Repost,
            MastodonNotification::Follow => Self::Follow,
            MastodonNotification::FollowRequest => Self::FollowRequest,
            MastodonNotification::Favourite => Self::Favourite,
            MastodonNotification::Update => Self::PostUpdate,
        }
    }
}

impl From<NotificationType> for MastodonNotification {
    fn from(value: NotificationType) -> Self {
        match value {
            NotificationType::Mention => MastodonNotification::Mention,
            NotificationType::Post => MastodonNotification::Status,
            NotificationType::Repost => MastodonNotification::Reblog,
            NotificationType::Follow => MastodonNotification::Follow,
            NotificationType::FollowRequest => MastodonNotification::FollowRequest,
            NotificationType::Favourite => MastodonNotification::Favourite,
            NotificationType::PostUpdate => MastodonNotification::Update,
        }
    }
}

impl<Db> FromSql<SmallInt, Db> for NotificationType
where
    i16: FromSql<SmallInt, Db>,
    Db: Backend,
{
    fn from_sql(bytes: <Db as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        Ok(Self::from_i16(value).ok_or(EnumConversionError(value))?)
    }
}

impl<Db> ToSql<SmallInt, Db> for NotificationType
where
    i16: ToSql<SmallInt, Db>,
    Db: Backend,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Db>) -> serialize::Result {
        // SAFETY: We have a `#[repr(i16)]` over the enum, so the representations are really the same
        #[allow(unsafe_code)]
        ToSql::to_sql(unsafe { &*ptr::from_ref(self).cast::<i16>() }, out)
    }
}
