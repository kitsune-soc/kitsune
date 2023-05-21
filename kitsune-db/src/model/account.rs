use crate::{error::EnumConversionError, impl_columns, schema::accounts};
use diesel::{
    backend::RawValue,
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::Integer,
    AsChangeset, AsExpression, FromSqlRow, Identifiable, Insertable, Queryable,
};
use kitsune_type::ap::actor::ActorType as ApActorType;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Deserialize, Identifiable, Serialize, Queryable)]
pub struct Account {
    pub id: Uuid,
    pub avatar_id: Option<Uuid>,
    pub header_id: Option<Uuid>,
    pub display_name: Option<String>,
    pub note: Option<String>,
    pub username: String,
    pub locked: bool,
    pub local: bool,
    pub domain: String,
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

impl_columns! {
    Account => (
        accounts::id,
        accounts::avatar_id,
        accounts::header_id,
        accounts::display_name,
        accounts::note,
        accounts::username,
        accounts::locked,
        accounts::local,
        accounts::domain,
        accounts::actor_type,
        accounts::url,
        accounts::featured_collection_url,
        accounts::followers_url,
        accounts::following_url,
        accounts::inbox_url,
        accounts::outbox_url,
        accounts::shared_inbox_url,
        accounts::public_key_id,
        accounts::public_key,
        accounts::created_at,
        accounts::updated_at,
    )
}

#[derive(AsChangeset)]
#[diesel(table_name = accounts)]
pub struct AccountConflictChangeset<'a> {
    pub display_name: Option<&'a str>,
    pub note: Option<&'a str>,
    pub locked: bool,
    pub public_key_id: &'a str,
    pub public_key: &'a str,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = accounts)]
pub struct UpdateAccount<'a> {
    pub display_name: Option<&'a str>,
    pub note: Option<&'a str>,
    pub avatar_id: Option<Uuid>,
    pub header_id: Option<Uuid>,
    pub locked: Option<bool>,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = accounts)]
pub struct UpdateAccountMedia {
    pub avatar_id: Option<Uuid>,
    pub header_id: Option<Uuid>,
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
    FromPrimitive,
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

impl FromSql<Integer, Pg> for ActorType
where
    i32: FromSql<Integer, Pg>,
{
    fn from_sql(bytes: RawValue<'_, Pg>) -> deserialize::Result<Self> {
        let value = i32::from_sql(bytes)?;
        Ok(Self::from_i32(value).ok_or(EnumConversionError(value))?)
    }
}

impl ToSql<Integer, Pg> for ActorType
where
    i32: ToSql<Integer, Pg>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <i32 as ToSql<Integer, _>>::to_sql(&(*self as i32), &mut out.reborrow())
    }
}
