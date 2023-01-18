#![doc = include_str!("../README.md")]
#![forbid(missing_docs, rust_2018_idioms, unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    forbidden_lint_groups,
    clippy::cast_possible_truncation,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions
)]

#[macro_use]
extern crate tracing;

pub mod config;
pub mod grpc;
pub mod search;
pub mod util;
