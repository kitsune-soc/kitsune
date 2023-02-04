//!
//! Custom column definitions
//!
//! These definitions are required by `SeaORM` if you only want to select some of the columns of a larger table.
//!

use sea_orm::prelude::*;

/// Inbox URL query columns
///
/// This only selects the column called `inbox_url` from a table
#[derive(Copy, Clone, Debug, DeriveColumn, EnumIter)]
pub enum InboxUrlQuery {
    /// `inbox_url` column
    InboxUrl,
}

/// URL query columns
///
/// This only selects the column called `url` from a table
#[derive(Copy, Clone, Debug, DeriveColumn, EnumIter)]
pub enum UrlQuery {
    /// `url` column
    Url,
}
