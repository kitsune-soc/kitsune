use bitflags::bitflags;
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::ToSql,
    sql_types::BigInt,
};
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(AsExpression, Clone, Copy, Debug, Deserialize, Eq, FromSqlRow, Ord, PartialEq, PartialOrd, Serialize)]
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub struct NotificationPreference: i64 {
        const ON_FOLLOW = 1 << 0;
        const ON_FOLLOW_REQUEST = 1 << 1;
        const ON_REPOST = 1 << 2;
        const ON_POST_UPDATE = 1 << 3;
        const ON_FAVOURITE = 1 << 4;
        const ON_MENTION = 1 << 5;
    }
}

impl<Db> FromSql<BigInt, Db> for NotificationPreference
where
    i64: FromSql<BigInt, Db>,
    Db: Backend,
{
    fn from_sql(bytes: <Db as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let value = i64::from_sql(bytes)?;
        Ok(Self::from_bits_truncate(value))
    }
}

impl<Db> ToSql<BigInt, Db> for NotificationPreference
where
    i64: ToSql<BigInt, Db>,
    Db: Backend,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Db>,
    ) -> diesel::serialize::Result {
        ToSql::to_sql(self.0.as_ref(), out)
    }
}
