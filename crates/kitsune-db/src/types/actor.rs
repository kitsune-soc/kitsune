use crate::error::EnumConversionError;
use diesel::{
    AsExpression, FromSqlRow,
    backend::Backend,
    deserialize::{self, FromSql},
    serialize::{self, Output, ToSql},
    sql_types::Integer,
};
use kitsune_derive::TwoWayFrom;
use kitsune_type::ap::actor::ActorType as ApActorType;
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
    TwoWayFrom,
)]
#[diesel(sql_type = diesel::sql_types::Integer)]
#[repr(i32)]
#[two_way_from(ApActorType)]
/// ActivityPub actor types
pub enum AccountType {
    /// Actor representing a group
    Group = 0,

    /// Actor representing an individual
    Person = 1,

    /// Actor representing a service (bot account)
    Service = 2,
}

impl AccountType {
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

impl<Db> FromSql<Integer, Db> for AccountType
where
    i32: FromSql<Integer, Db>,
    Db: Backend,
{
    fn from_sql(bytes: <Db as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let value = i32::from_sql(bytes)?;
        Ok(Self::from_i32(value).ok_or(EnumConversionError(value))?)
    }
}

impl<Db> ToSql<Integer, Db> for AccountType
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
