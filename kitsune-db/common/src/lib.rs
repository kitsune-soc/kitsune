//!
//! Common database utility code shared between the `migration` and the main database crate
//!

#![forbid(rust_2018_idioms)]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions, forbidden_lint_groups)]

pub mod tsvector_column;
pub mod types;
