use diesel::{deserialize, pg::Pg, row::NamedRow, QueryableByName};
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::collections::HashSet;
use std::fmt::Write;
use whatlang::Lang;

#[derive(Debug)]
struct PgCatalogResult {
    cfgname: String,
}

impl QueryableByName<Pg> for PgCatalogResult {
    fn build<'a>(row: &impl NamedRow<'a, Pg>) -> deserialize::Result<Self> {
        Ok(Self {
            cfgname: NamedRow::get(row, "cfgname")?,
        })
    }
}

pub async fn generate_regconfig_function<C>(
    conn: &mut C,
    function_name: &str,
) -> diesel::QueryResult<()>
where
    C: AsyncConnection<Backend = Pg>,
{
    let supported_languages: Vec<PgCatalogResult> =
        diesel::sql_query("SELECT cfgname FROM pg_catalog.pg_ts_config;")
            .get_results(conn)
            .await?;

    let supported_languages: HashSet<String> = supported_languages
        .into_iter()
        .map(|result| result.cfgname)
        .collect();

    let mut function = format!(
        r#"
        CREATE OR REPLACE FUNCTION {function_name} (TEXT)
            RETURNS regconfig
            AS $$
                SELECT CASE $1
        "#
    );

    for lang in Lang::all() {
        let english_name = lang.eng_name().to_lowercase();
        if !supported_languages.contains(&english_name) {
            continue;
        }

        let _ = writeln!(
            &mut function,
            "WHEN '{}' THEN '{english_name}'::regconfig",
            lang.code()
        );
    }

    let _ = writeln!(&mut function, "ELSE 'english'::regconfig");
    let _ = writeln!(&mut function, "END");
    let _ = writeln!(&mut function, "$$ LANGUAGE SQL IMMUTABLE;");

    diesel::sql_query(function).execute(conn).await?;

    Ok(())
}
