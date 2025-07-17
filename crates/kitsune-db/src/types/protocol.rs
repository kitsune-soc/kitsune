use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::ToSql,
    sql_types::Integer,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::ptr;

use crate::error::EnumConversionError;

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
#[repr(i32)]
pub enum Protocol {
    Activitypub = 0,
    Atproto = 1,
}

impl<Db> FromSql<Integer, Db> for Protocol
where
    i32: FromSql<Integer, Db>,
    Db: Backend,
{
    fn from_sql(bytes: <Db as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let value = i32::from_sql(bytes)?;
        Self::from_i32(value).ok_or_else(|| EnumConversionError(value).into())
    }
}

impl<Db> ToSql<Integer, Db> for Protocol
where
    i32: ToSql<Integer, Db>,
    Db: Backend,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Db>,
    ) -> diesel::serialize::Result {
        // SAFETY: We have a `#[repr(i32)]` over the enum, so the representations are really the same
        #[allow(unsafe_code)]
        ToSql::to_sql(unsafe { &*ptr::from_ref(self).cast::<i32>() }, out)
    }
}
