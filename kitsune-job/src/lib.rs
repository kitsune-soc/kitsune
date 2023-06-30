#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::used_underscore_binding,
    forbidden_lint_groups
)]

#[macro_use]
extern crate tracing;

use self::error::Result;
use async_trait::async_trait;
use std::time::Duration;

pub use self::{
    error::Error,
    queue::{JobDetails, JobQueue},
};

mod error;
mod macros;
mod queue;

const MAX_RETRIES: u32 = 10;
const MIN_BACKOFF_DURATION: Duration = Duration::from_secs(5);

#[async_trait]
pub trait Runnable {
    async fn run(&self) -> Result<()>;
}
