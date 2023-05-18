//!
//! Common database utility code shared between the `migration` and the main database crate
//!

#![forbid(rust_2018_idioms)]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, forbidden_lint_groups)]

mod generated_column;

pub mod tsvector_column;
pub mod types;

pub use crate::generated_column::StoredGeneratedColumn;

use sea_orm::sea_query::{Expr, FunctionCall, PgFunc, SimpleExpr};

/// Create a function call to the `to_tsvector` function using the `('[language]', [content])` syntax
pub fn to_tsvector<L, V>(lang: L, val: V) -> FunctionCall
where
    L: Into<SimpleExpr>,
    V: Into<SimpleExpr>,
{
    PgFunc::to_tsvector(Expr::asterisk(), None).args([lang.into(), val.into()])
}
