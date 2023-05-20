use diesel::{
    sql_function,
    sql_types::{Nullable, Text},
};

sql_function! {
    /// Coalesce between two nullable text fields, where one of those should have a value
    #[sql_name = "COALESCE"]
    fn coalesce_nullable(x: Nullable<Text>, y: Nullable<Text>) -> Text;
}

sql_function! {
    /// Return the current date with the timezone
    fn now() -> Timestamptz;
}
