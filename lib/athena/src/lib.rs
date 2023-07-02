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

use self::error::{BoxError, Result};
use async_trait::async_trait;
use futures_util::Stream;
use speedy_uuid::Uuid;

pub use self::{
    error::Error,
    queue::{JobDetails, JobQueue},
};

mod error;
mod macros;
mod queue;

#[async_trait]
pub trait Runnable: Clone {
    /// User-defined context that is getting passed to the job when run
    ///
    /// This way you can reference services, configurations, etc.
    type Context: Send + Sync + 'static;

    type Error: Into<BoxError> + Send;

    /// Run the job
    async fn run(&self, ctx: &Self::Context) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait JobContextRepository {
    /// Some job context
    ///
    /// To support multiple job types per repository, consider using the enum dispatch technique
    type JobContext: Runnable + Send + Sync + 'static;
    type Error: Into<BoxError>;
    type Stream: Stream<Item = Result<(Uuid, Self::JobContext), Self::Error>>;

    /// Batch fetch job contexts
    ///
    /// The stream has to return `([Job ID], [Job context])`, this gives you the advantage that the order isn't enforced.
    /// You can return them as you find them
    async fn fetch_context<I>(&self, job_ids: I) -> Result<Self::Stream, Self::Error>
    where
        I: Iterator<Item = Uuid> + Send + 'static;

    /// Remove job context from the database
    async fn remove_context(&self, job_id: Uuid) -> Result<(), Self::Error>;

    /// Store job context into the database
    ///
    /// Make sure the job can be efficiently found via the job ID (such as using the job ID as the primary key of a database table)
    async fn store_context(
        &self,
        job_id: Uuid,
        context: Self::JobContext,
    ) -> Result<(), Self::Error>;
}
