#[macro_use]
extern crate tracing;

use self::error::{BoxError, Result};
use async_trait::async_trait;
use futures_util::{Future, Stream};
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;
use std::{
    any::{Any, TypeId},
    sync::Arc,
};
use typed_builder::TypedBuilder;

pub use self::error::Error;
pub use tokio_util::task::TaskTracker;

pub use self::common::spawn_jobs;
#[cfg(feature = "redis")]
pub use self::redis::JobQueue as RedisJobQueue;

mod common;
mod error;
mod macros;
#[cfg(feature = "redis")]
mod redis;

pub mod consts;

#[derive(TypedBuilder)]
#[non_exhaustive]
pub struct JobDetails<C> {
    #[builder(setter(into))]
    pub context: C,
    #[builder(default)]
    pub fail_count: u32,
    #[builder(default = Uuid::now_v7(), setter(into))]
    pub job_id: Uuid,
    #[builder(default, setter(into))]
    pub run_at: Option<Timestamp>,
}

#[typetag::serde]
pub trait Keepable: Any + Send + Sync + 'static {}

// Hack around <https://github.com/rust-lang/rust/issues/65991> because it's not stable yet.
// So I had to implement trait downcasting myself.
//
// TODO: Remove this once <https://github.com/rust-lang/rust/issues/65991> is stabilized.
#[inline]
fn downcast_to<T>(obj: &dyn Keepable) -> Option<&T>
where
    T: 'static,
{
    if obj.type_id() == TypeId::of::<T>() {
        #[allow(unsafe_code)]
        // SAFETY: the `TypeId` equality check ensures this type cast is correct
        Some(unsafe { &*(obj as *const dyn Keepable).cast::<T>() })
    } else {
        None
    }
}

#[typetag::serde]
impl Keepable for String {}

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct KeeperOfTheSecrets {
    inner: Option<Box<dyn Keepable>>,
}

impl KeeperOfTheSecrets {
    #[inline]
    #[must_use]
    pub fn empty() -> Self {
        Self { inner: None }
    }

    #[inline]
    pub fn new<T>(inner: T) -> Self
    where
        T: Keepable,
    {
        Self {
            inner: Some(Box::new(inner)),
        }
    }

    #[inline]
    #[must_use]
    pub fn get<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        self.inner
            .as_ref()
            .and_then(|item| downcast_to(item.as_ref()))
    }
}

pub enum Outcome {
    Success,
    Fail { fail_count: u32 },
}

pub struct JobResult<'a> {
    pub outcome: Outcome,
    pub job_id: Uuid,
    pub ctx: &'a KeeperOfTheSecrets,
}

#[derive(Deserialize, Serialize)]
pub struct JobData {
    pub job_id: Uuid,
    pub fail_count: u32,
    pub ctx: KeeperOfTheSecrets,
}

#[async_trait]
pub trait JobQueue: Send + Sync + 'static {
    type ContextRepository: JobContextRepository + 'static;

    fn context_repository(&self) -> &Self::ContextRepository;

    async fn enqueue(
        &self,
        job_details: JobDetails<<Self::ContextRepository as JobContextRepository>::JobContext>,
    ) -> Result<()>;

    async fn fetch_job_data(&self, max_jobs: usize) -> Result<Vec<JobData>>;

    async fn reclaim_job(&self, job_data: &JobData) -> Result<()>;

    async fn complete_job(&self, state: &JobResult<'_>) -> Result<()>;
}

#[async_trait]
impl<CR> JobQueue for Arc<dyn JobQueue<ContextRepository = CR> + '_>
where
    CR: JobContextRepository + 'static,
{
    type ContextRepository = CR;

    fn context_repository(&self) -> &Self::ContextRepository {
        (**self).context_repository()
    }

    async fn enqueue(
        &self,
        job_details: JobDetails<<Self::ContextRepository as JobContextRepository>::JobContext>,
    ) -> Result<()> {
        (**self).enqueue(job_details).await
    }

    async fn fetch_job_data(&self, max_jobs: usize) -> Result<Vec<JobData>> {
        (**self).fetch_job_data(max_jobs).await
    }

    async fn reclaim_job(&self, job_data: &JobData) -> Result<()> {
        (**self).reclaim_job(job_data).await
    }

    async fn complete_job(&self, state: &JobResult<'_>) -> Result<()> {
        (**self).complete_job(state).await
    }
}

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
    type Stream: Stream<Item = Result<(Uuid, Self::JobContext), Self::Error>> + Send;

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
