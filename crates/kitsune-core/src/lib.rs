#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    forbidden_lint_groups
)]

#[macro_use]
extern crate tracing;

pub mod activitypub;
pub mod blocking;
pub mod config;
pub mod consts;
pub mod error;
pub mod event;
pub mod job;
pub mod mapping;
pub mod resolve;
pub mod sanitize;
pub mod service;
pub mod state;
pub mod util;
pub mod webfinger;
