#[macro_use]
extern crate tracing;

use self::error::{BoxError, Result};
use futures_util::{Future, Stream};
use speedy_uuid::Uuid;

pub use self::{
    error::Error,
    queue::{JobDetails, JobQueue},
};

mod error;
mod macros;
mod queue;

type RedisPool = multiplex_pool::Pool<redis::aio::ConnectionManager>;

pub trait Runnable {
    /// User-defined context that is getting passed to the job when run
    ///
    /// This way you can reference services, configurations, etc.
    type Context: Send + Sync + 'static;

    type Error: Into<BoxError> + Send;

    /// Run the job
    fn run(&self, ctx: &Self::Context) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

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
    fn fetch_context<I>(
        &self,
        job_ids: I,
    ) -> impl Future<Output = Result<Self::Stream, Self::Error>> + Send
    where
        I: Iterator<Item = Uuid> + Send + 'static;

    /// Remove job context from the database
    fn remove_context(&self, job_id: Uuid) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Store job context into the database
    ///
    /// Make sure the job can be efficiently found via the job ID (such as using the job ID as the primary key of a database table)
    fn store_context(
        &self,
        job_id: Uuid,
        context: Self::JobContext,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
