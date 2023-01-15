#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    forbidden_lint_groups
)]

#[macro_use]
extern crate tracing;

pub mod activitypub;
pub mod blocking;
pub mod cache;
pub mod config;
pub mod consts;
pub mod db;
pub mod error;
pub mod http;
pub mod job;
pub mod mapping;
pub mod resolve;
pub mod sanitize;
pub mod search;
pub mod state;
pub mod util;
pub mod webfinger;
