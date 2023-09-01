use crate::{error::EnumConversionError, schema::notifications};
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::SmallInt,
    AsExpression, FromSqlRow, Identifiable, Insertable, Queryable, Selectable,
};
use iso8601_timestamp::Timestamp;
use kitsune_type::mastodon::notification::NotificationType as MastodonNotification;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

#[derive(Identifiable, Insertable, Selectable, Queryable)]
#[diesel(table_name = notifications)]
pub struct Notification {
    pub id: Uuid,
    pub receiving_account_id: Uuid,
    pub triggering_account_id: Option<Uuid>,
    pub post_id: Option<Uuid>,
    pub notification_type: NotificationType,
    pub created_at: Timestamp,
}

#[derive(TypedBuilder)]
#[builder(build_method(vis="", name=__build))]
pub struct NewNotification {
    #[builder(default = Uuid::now_v7())]
    pub id: Uuid,
    pub receiving_account_id: Uuid,
    #[builder(default = Timestamp::now_utc())]
    pub created_at: Timestamp,
}

#[derive(TypedBuilder, Clone, Copy)]
struct NewNotificationExtraFields {
    pub triggering_account_id: Option<Uuid>,
    pub post_id: Option<Uuid>,
    pub notification_type: NotificationType,
}

#[allow(non_camel_case_types)]
impl<__id: ::typed_builder::Optional<Uuid>, __created_at: ::typed_builder::Optional<Timestamp>>
    NewNotificationBuilder<(__id, (Uuid,), __created_at)>
{
    pub fn follow(self, triggering_account_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: None,
            notification_type: NotificationType::Follow,
        })
    }

    pub fn follow_request(self, triggering_account_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: None,
            notification_type: NotificationType::FollowRequest,
        })
    }

    pub fn favourite(self, triggering_account_id: Uuid, post_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: Some(post_id),
            notification_type: NotificationType::Favourite,
        })
    }

    pub fn mention(self, triggering_account_id: Uuid, post_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: Some(post_id),
            notification_type: NotificationType::Mention,
        })
    }

    pub fn post(self, triggering_account_id: Uuid, post_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: Some(post_id),
            notification_type: NotificationType::Post,
        })
    }

    pub fn repost(self, triggering_account_id: Uuid, post_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: Some(post_id),
            notification_type: NotificationType::Repost,
        })
    }

    pub fn post_update(self, triggering_account_id: Uuid, post_id: Uuid) -> Notification {
        self.build(&NewNotificationExtraFields {
            triggering_account_id: Some(triggering_account_id),
            post_id: Some(post_id),
            notification_type: NotificationType::PostUpdate,
        })
    }

    fn build(self, extra_fields: &NewNotificationExtraFields) -> Notification {
        let built = self.__build();
        Notification {
            id: built.id,
            receiving_account_id: built.receiving_account_id,
            triggering_account_id: extra_fields.triggering_account_id,
            post_id: extra_fields.post_id,
            notification_type: extra_fields.notification_type,
            created_at: built.created_at,
        }
    }
}

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

impl FromSql<SmallInt, Pg> for NotificationType
where
    i16: FromSql<SmallInt, Pg>,
{
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let value = i16::from_sql(bytes)?;
        Ok(Self::from_i16(value).ok_or(EnumConversionError(value.into()))?)
    }
}

impl ToSql<SmallInt, Pg> for NotificationType
where
    i16: ToSql<SmallInt, Pg>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <i16 as ToSql<SmallInt, _>>::to_sql(&(*self as i16), &mut out.reborrow())
    }
}
