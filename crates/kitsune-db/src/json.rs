use diesel::{
    AsExpression, FromSqlRow,
    backend::Backend,
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, IsNull, ToSql},
    sql_types::Jsonb,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sonic_rs::writer::BufferedWriter;
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
    #[inline]
    fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let bytes = bytes.as_bytes();
        if bytes[0] != 1 {
            return Err(JsonError("Unsupported JSONB encoding version").into());
        }
        Ok(sonic_rs::from_slice(&bytes[1..])?)
    }
}

impl<T> ToSql<Jsonb, Pg> for Json<T>
where
    T: Debug + Serialize,
{
    #[inline]
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(&[1])?;
        sonic_rs::to_writer(BufferedWriter::new(out), self)
            .map(|()| IsNull::No)
            .map_err(Into::into)
    }
}
