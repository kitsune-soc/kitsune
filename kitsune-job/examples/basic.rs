use async_trait::async_trait;
use deadpool_redis::{Config, Runtime};
use futures_util::{
    stream::{self, BoxStream},
    StreamExt,
};
use iso8601_timestamp::{Duration, Timestamp};
use kitsune_job::{JobContextRepository, JobDetails, JobQueue, Runnable};
use kitsune_uuid::Uuid;
use std::{io, sync::Arc};

#[derive(Clone)]
struct JobCtx;

#[async_trait]
impl Runnable for JobCtx {
    type Context = ();
    type Error = io::Error;

    async fn run(&self, _ctx: &Self::Context) -> Result<(), Self::Error> {
        println!("ran job!");
        Ok(())
    }
}

struct ContextRepo;

#[async_trait]
impl JobContextRepository for ContextRepo {
    type JobContext = JobCtx;
    type Error = io::Error;
    type Stream = BoxStream<'static, Result<(Uuid, Self::JobContext), Self::Error>>;

    async fn fetch_context<I>(&self, job_ids: I) -> Result<Self::Stream, Self::Error>
    where
        I: Iterator<Item = Uuid> + Send + 'static,
    {
        let stream = stream::iter(job_ids).map(|id| Ok((id, JobCtx)));
        Ok(stream.boxed())
    }

    async fn remove_context(&self, _job_id: Uuid) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn store_context(
        &self,
        _job_id: Uuid,
        _context: Self::JobContext,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    let cfg = Config::from_url("redis://localhost");
    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();

    let queue = JobQueue::builder()
        .context_repository(ContextRepo)
        .queue_name("test_queue")
        .redis_pool(pool)
        .build();

    for _ in 0..100 {
        queue
            .enqueue(JobDetails::builder().context(JobCtx).build())
            .await
            .unwrap();
    }

    for _ in 0..100 {
        queue
            .enqueue(
                JobDetails::builder()
                    .context(JobCtx)
                    .run_at(Timestamp::now_utc() + Duration::SECOND)
                    .build(),
            )
            .await
            .unwrap();
    }

    let mut jobs = queue.spawn_jobs(20, Arc::new(())).await.unwrap();
    while jobs.join_next().await.is_some() {}
}
