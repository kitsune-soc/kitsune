use super::user::User;
use crate::{error::EnumConversionError, schema::users_roles};
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::Integer,
    AsExpression, Associations, FromSqlRow, Identifiable, Insertable, Queryable,
};
use iso8601_timestamp::Timestamp;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

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
#[serde(rename_all = "camelCase")]
#[diesel(sql_type = diesel::sql_types::Integer)]
/// Role of a local user on this server
pub enum Role {
    /// Administrator
    ///
    /// This user is an administrator on this instance and has elevated access
    Administrator = 0,
}

impl FromSql<Integer, Pg> for Role
where
    i32: FromSql<Integer, Pg>,
{
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let value = i32::from_sql(bytes)?;
        Ok(Self::from_i32(value).ok_or(EnumConversionError(value))?)
    }
}

impl ToSql<Integer, Pg> for Role
where
    i32: ToSql<Integer, Pg>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <i32 as ToSql<Integer, _>>::to_sql(&(*self as i32), &mut out.reborrow())
    }
}

#[derive(Associations, Clone, Identifiable, Queryable)]
#[diesel(belongs_to(User), table_name = users_roles)]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: Role,
    pub created_at: Timestamp,
}

#[derive(Clone, Deserialize, Insertable, Serialize)]
#[diesel(table_name = users_roles)]
pub struct NewUserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: Role,
}
