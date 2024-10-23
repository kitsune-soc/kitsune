use crate::supported_languages;
use diesel::{deserialize, pg::Pg, row::NamedRow, sql_types, QueryableByName};
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::{collections::HashSet, fmt::Write};

#[derive(Debug)]
struct PgCatalogResult {
    cfgname: String,
}

impl QueryableByName<Pg> for PgCatalogResult {
    fn build<'a>(row: &impl NamedRow<'a, Pg>) -> deserialize::Result<Self> {
        Ok(Self {
            cfgname: NamedRow::get::<sql_types::Text, _>(row, "cfgname")?,
        })
    }
}

pub async fn generate_regconfig_function<C>(
    conn: &mut C,
    function_name: &str,
    enum_name: &str,
) -> diesel::QueryResult<()>
where
    C: AsyncConnection<Backend = Pg>,
{
    let pg_supported_languages: Vec<PgCatalogResult> =
        diesel::sql_query("SELECT cfgname FROM pg_catalog.pg_ts_config;")
            .get_results(conn)
            .await?;

    let pg_supported_languages: HashSet<String> = pg_supported_languages
        .into_iter()
        .map(|result| result.cfgname)
        .collect();

    let mut function = format!(
        r"
        CREATE OR REPLACE FUNCTION {function_name} ({enum_name})
            RETURNS regconfig
            AS $$
                SELECT CASE $1
        "
    );

    for lang in supported_languages() {
        let english_name = lang.to_name().to_lowercase();
        if !pg_supported_languages.contains(&english_name) {
            continue;
        }

        writeln!(
            function,
            "WHEN '{}' THEN '{english_name}'::regconfig",
            lang.to_639_3()
        )
        .unwrap();
    }

    writeln!(
        function,
        r"
                ELSE 'english'::regconfig
                END
            $$ LANGUAGE SQL IMMUTABLE;
        "
    )
    .unwrap();

    conn.batch_execute(&function).await?;

    Ok(())
}
