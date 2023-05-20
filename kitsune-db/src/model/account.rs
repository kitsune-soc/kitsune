use crate::schema::accounts;
use diesel::{AsExpression, FromSqlRow, Identifiable, Insertable, Queryable};
use kitsune_type::ap::actor::ActorType as ApActorType;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Deserialize, Identifiable, Serialize, Queryable)]
pub struct Account {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub note: Option<String>,
    pub username: String,
    pub locked: bool,
    pub local: bool,
    pub doamin: String,
    pub actor_type: ActorType,
    pub url: Option<String>,
    pub featured_collection_url: Option<String>,
    pub followers_url: Option<String>,
    pub following_url: Option<String>,
    pub inbox_url: Option<String>,
    pub outbox_url: Option<String>,
    pub shared_inbox_url: Option<String>,
    pub public_key_id: String,
    pub public_key: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = accounts)]
pub struct NewAccount<'a> {
    pub id: Uuid,
    pub display_name: Option<&'a str>,
    pub note: Option<&'a str>,
    pub username: &'a str,
    pub locked: bool,
    pub local: bool,
    pub domain: &'a str,
    pub actor_type: ActorType,
    pub url: Option<&'a str>,
    pub featured_collection_url: Option<&'a str>,
    pub followers_url: Option<&'a str>,
    pub following_url: Option<&'a str>,
    pub inbox_url: Option<&'a str>,
    pub outbox_url: Option<&'a str>,
    pub shared_inbox_url: Option<&'a str>,
    pub public_key_id: &'a str,
    pub public_key: &'a str,
    pub created_at: Option<OffsetDateTime>,
}

#[derive(
    AsExpression,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    FromSqlRow,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[diesel(sql_type = diesel::sql_types::Integer)]
/// ActivityPub actor types
pub enum ActorType {
    /// Actor representing a group
    Group = 0,

    /// Actor representing an individual
    Person = 1,

    /// Actor representing a service (bot account)
    Service = 2,
}

impl ActorType {
    /// Return whether this actor type represents a bot account
    #[must_use]
    pub fn is_bot(&self) -> bool {
        ApActorType::from(*self).is_bot()
    }

    /// Return whether this actor type represents a group
    #[must_use]
    pub fn is_group(&self) -> bool {
        ApActorType::from(*self).is_group()
    }
}

impl From<ApActorType> for ActorType {
    fn from(value: ApActorType) -> Self {
        match value {
            ApActorType::Group => Self::Group,
            ApActorType::Person => Self::Person,
            ApActorType::Service => Self::Service,
        }
    }
}

impl From<ActorType> for ApActorType {
    fn from(value: ActorType) -> Self {
        match value {
            ActorType::Group => Self::Group,
            ActorType::Person => Self::Person,
            ActorType::Service => Self::Service,
        }
    }
}
