use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, IsNull, ToSql},
    sql_types::Jsonb,
    AsExpression, FromSqlRow,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Debug},
    io::Write,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct JsonError(&'static str);

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSONB error: {}", self.0)
    }
}

impl Error for JsonError {}

#[derive(AsExpression, Clone, Debug, Deserialize, FromSqlRow, Serialize)]
#[diesel(sql_type = Jsonb)]
#[serde(transparent)]
pub struct Json<T>(pub T);

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> FromSql<Jsonb, Pg> for Json<T>
where
    T: DeserializeOwned,
{
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let bytes = bytes.as_bytes();
        if bytes[0] != 1 {
            return Err(JsonError("Unsupported JSONB encoding version").into());
        }
        Ok(simd_json::from_reader(&bytes[1..])?)
    }
}

impl<T> ToSql<Jsonb, Pg> for Json<T>
where
    T: Debug + Serialize,
{
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(&[1])?;
        simd_json::to_writer(out, self)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}
