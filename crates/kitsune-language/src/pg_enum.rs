use crate::supported_languages;
use diesel::{QueryResult, QueryableByName, pg::Pg, row::NamedRow};
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::fmt::Write;

struct CountResult {
    count: i64,
}

impl QueryableByName<Pg> for CountResult {
    fn build<'a>(row: &impl NamedRow<'a, Pg>) -> diesel::deserialize::Result<Self> {
        Ok(Self {
            count: NamedRow::get(row, "count")?,
        })
    }
}

/// Generate a PostgreSQL enum definition of all supported ISO language codes
pub async fn generate_postgres_enum<C>(conn: &mut C, enum_name: &str) -> QueryResult<()>
where
    C: AsyncConnection<Backend = Pg>,
{
    let language_count: CountResult = diesel::sql_query(format!(
        "SELECT COUNT(1) AS count FROM UNNEST(ENUM_RANGE(NULL::{enum_name}));"
    ))
    .get_result(conn)
    .await?;

    // Good enough.
    #[allow(clippy::cast_possible_wrap)] // There are only ~200 languages
    if language_count.count == supported_languages().count() as i64 {
        return Ok(());
    }

    let queries = supported_languages().fold(String::new(), |mut out, lang| {
        write!(
            out,
            "ALTER TYPE {enum_name} ADD VALUE IF NOT EXISTS '{}';",
            lang.to_639_3()
        )
        .unwrap();

        out
    });

    conn.batch_execute(&queries).await?;

    Ok(())
}
