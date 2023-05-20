use diesel::{
    sql_function,
    sql_types::{Nullable, Text},
};
use diesel_full_text_search::TsQuery;

sql_function! {
    /// Coalesce between two nullable text fields, where one of those should have a value
    #[sql_name = "COALESCE"]
    fn coalesce_nullable(x: Nullable<Text>, y: Nullable<Text>) -> Text;
}

sql_function! {
    /// Return the current date with the timezone
    fn now() -> Timestamptz;
}

sql_function! {
    /// The `websearch_to_tsquery` in its simplest form with only one input parameter
    fn websearch_to_tsquery(query: Text) -> TsQuery;
}
