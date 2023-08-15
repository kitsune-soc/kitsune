#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use diesel::{pg::Pg, QueryResult};
use diesel_async::{AsyncConnection, RunQueryDsl};

mod regconfig;

pub use self::regconfig::generate_regconfig_function;
pub use whatlang::Lang;

/// Get the ISO code of the specified text
///
/// If the language couldn't get detected, default to english
#[must_use]
pub fn get_iso_code(text: &str) -> Lang {
    whatlang::detect_lang(text).unwrap_or(Lang::Eng)
}

/// Generate a PostgreSQL enum definition of all supported ISO language codes
pub async fn generate_postgres_enum<C>(conn: &mut C, enum_name: &str) -> QueryResult<()>
where
    C: AsyncConnection<Backend = Pg>,
{
    for lang in Lang::all() {
        diesel::sql_query(format!(
            "ALTER TYPE {enum_name} ADD VALUE IF NOT EXISTS '{}';",
            lang.code()
        ))
        .execute(conn)
        .await?;
    }

    Ok(())
}
