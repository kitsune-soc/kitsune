use crate::schema::sql_types::LanguageIsoCode;
use diesel::{
    define_sql_function,
    sql_types::{Nullable, Text},
};
use diesel_full_text_search::RegConfig;

define_sql_function! {
    /// Coalesce between two nullable text fields, where one of those should have a value
    #[sql_name = "COALESCE"]
    fn coalesce_nullable(x: Nullable<Text>, y: Nullable<Text>) -> Text;
}

define_sql_function! {
    /// Get the optimal regconfig for the ISO code with the current database configuration
    fn iso_code_to_language(iso_code: LanguageIsoCode) -> RegConfig;
}

define_sql_function! {
    /// Return the current date with the timezone
    fn now() -> Timestamptz;
}
