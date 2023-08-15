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

// TODO: Get rid of this when (if?) `whatlang` adds serde support`
//
// See: <https://github.com/greyblake/whatlang-rs/issues/134>
mod serde_impl {
    use core::fmt;
    use serde::{Deserializer, Serializer};

    struct LangVisitor;

    impl<'vi> serde::de::Visitor<'vi> for LangVisitor {
        type Value = kitsune_lang_id::Lang;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "an ISO language code")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<kitsune_lang_id::Lang, E> {
            value
                .parse()
                .map_err(|err| E::custom(format_args!("ISO code parsing failed: {err}")))
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<kitsune_lang_id::Lang, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(LangVisitor)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)] // We can't control this. It's serde specific
    pub fn serialize<S>(code: &kitsune_lang_id::Lang, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(code.code())
    }
}

#[derive(AsExpression, Clone, Copy, Debug, Deserialize, Eq, FromSqlRow, PartialEq, Serialize)]
#[diesel(sql_type = sql_types::Languageisocode)]
#[repr(transparent)]
#[serde(transparent)]
pub struct LanguageIsoCode(#[serde(with = "serde_impl")] pub kitsune_lang_id::Lang);

impl Deref for LanguageIsoCode {
    type Target = kitsune_lang_id::Lang;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LanguageIsoCode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<kitsune_lang_id::Lang> for LanguageIsoCode {
    fn from(value: kitsune_lang_id::Lang) -> Self {
        Self(value)
    }
}

impl FromSql<sql_types::Languageisocode, Pg> for LanguageIsoCode {
    fn from_sql(
        bytes: <Pg as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let code_txt = unsafe { str::from_utf8_unchecked(bytes.as_bytes()) };
        let lang = kitsune_lang_id::Lang::from_code(code_txt)
            .ok_or_else(|| IsoCodeConversionError(code_txt.to_string()))?;

        Ok(Self(lang))
    }
}

impl ToSql<sql_types::Languageisocode, Pg> for LanguageIsoCode {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        out.write_all(self.0.code().as_bytes())?;

        Ok(serialize::IsNull::No)
    }
}
