use crate::error::EnumConversionError;
use diesel::{
    AsExpression, FromSqlRow,
    backend::Backend,
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::Integer,
};
use kitsune_type::ap::actor::ActorType as ApActorType;
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
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
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
