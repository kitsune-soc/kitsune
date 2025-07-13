use crate::error::EnumConversionError;
use diesel::{
    AsExpression, FromSqlRow,
    backend::Backend,
    deserialize::{self, FromSql},
    query_builder::BindCollector,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::SmallInt,
};
use kitsune_type::mastodon::notification::NotificationType as MastodonNotification;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};

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
        Ok(Self::from_i16(value).ok_or(EnumConversionError(value.into()))?)
    }
}

impl<Db> ToSql<SmallInt, Db> for NotificationType
where
    i16: for<'a> Into<<Db::BindCollector<'a> as BindCollector<'a, Db>>::Buffer>
        + ToSql<SmallInt, Db>,
    Db: Backend,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Db>) -> serialize::Result {
        out.set_value(*self as i16);
        Ok(IsNull::No)
    }
}
