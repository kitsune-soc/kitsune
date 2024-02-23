use crate::{error::IsoCodeConversionError, schema::sql_types};
use diesel::{
    deserialize::FromSql,
    pg::Pg,
    serialize::{self, ToSql},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    ops::{Deref, DerefMut},
    str,
};

#[derive(AsExpression, Clone, Copy, Debug, Deserialize, Eq, FromSqlRow, PartialEq, Serialize)]
#[diesel(sql_type = sql_types::LanguageIsoCode)]
#[repr(transparent)]
#[serde(transparent)]
pub struct LanguageIsoCode(pub kitsune_language::Language);

impl Deref for LanguageIsoCode {
    type Target = kitsune_language::Language;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LanguageIsoCode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<kitsune_language::Language> for LanguageIsoCode {
    fn from(value: kitsune_language::Language) -> Self {
        Self(value)
    }
}

impl FromSql<sql_types::LanguageIsoCode, Pg> for LanguageIsoCode {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        #[allow(unsafe_code)]
        let code_txt = unsafe { str::from_utf8_unchecked(bytes.as_bytes()) };
        let lang = kitsune_language::Language::from_639_3(code_txt)
            .ok_or_else(|| IsoCodeConversionError(code_txt.to_string()))?;

        Ok(Self(lang))
    }
}

impl ToSql<sql_types::LanguageIsoCode, Pg> for LanguageIsoCode {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(self.0.to_639_3().as_bytes())?;

        Ok(serialize::IsNull::No)
    }
}
