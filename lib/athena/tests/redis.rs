#![cfg(feature = "redis")]

use athena::{JobContextRepository, JobDetails, JobQueue, RedisJobQueue, Runnable};
use futures_util::{
    stream::{self, BoxStream},
    StreamExt,
};
use kitsune_test::redis_test;
use speedy_uuid::Uuid;
use std::{
    convert::Infallible,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio_util::task::TaskTracker;
use triomphe::Arc;

static DID_RUN: AtomicBool = AtomicBool::new(false);

#[derive(Clone)]
struct JobCtx;

impl Runnable for JobCtx {
    type Context = ();
    type Error = Infallible;

    async fn run(&self, _ctx: &Self::Context) -> Result<(), Self::Error> {
        DID_RUN.store(true, Ordering::Release);
        Ok(())
    }
}

struct ContextRepo;

impl JobContextRepository for ContextRepo {
    type JobContext = JobCtx;
    type Error = Infallible;
    type Stream = BoxStream<'static, Result<(Uuid, Self::JobContext), Self::Error>>;

    async fn fetch_context<I>(&self, job_ids: I) -> Result<Self::Stream, Self::Error>
    where
        I: Iterator<Item = Uuid> + Send,
    {
        let vec: Vec<_> = job_ids.collect();
        let stream = stream::iter(vec).map(|id| Ok((id, JobCtx)));
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

#[tokio::test]
async fn basic_schedule() {
    redis_test(|pool| async move {
        let queue = RedisJobQueue::builder()
            .conn_pool(pool)
            .context_repository(ContextRepo)
            .queue_name("test_queue")
            .build();

        queue
            .enqueue(JobDetails::builder().context(JobCtx).build())
            .await
            .unwrap();

        let jobs = TaskTracker::new();
        jobs.close();

        athena::spawn_jobs(&queue, 1, Arc::new(()), &jobs)
            .await
            .unwrap();

        jobs.wait().await;

        assert!(DID_RUN.load(Ordering::Acquire));
    })
    .await;
}
