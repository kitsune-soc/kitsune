use athena::{JobContextRepository, JobDetails, JobQueue, RedisJobQueue, Runnable};
use fred::{
    clients::Pool as RedisPool, interfaces::ClientLike, types::config::Config as RedisConfig,
};
use futures_util::{
    stream::{self, BoxStream},
    StreamExt,
};
use iso8601_timestamp::Timestamp;
use speedy_uuid::Uuid;
use std::{
    io,
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};
use tokio_util::task::TaskTracker;
use triomphe::Arc;

#[derive(Clone)]
struct JobCtx;

impl Runnable for JobCtx {
    type Context = ();
    type Error = io::Error;

    async fn run(&self, _ctx: &Self::Context) -> Result<(), Self::Error> {
        static COUNTER: AtomicU32 = AtomicU32::new(1);
        println!("ran job {}!", COUNTER.fetch_add(1, Ordering::AcqRel));
        Ok(())
    }
}

struct ContextRepo;

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

    let config = RedisConfig::from_url("redis://localhost").unwrap();
    let pool = RedisPool::new(config, None, None, None, 5).unwrap();
    pool.init().await.unwrap();

    let queue = RedisJobQueue::builder()
        .conn_pool(pool)
        .context_repository(ContextRepo)
        .queue_name("test_queue")
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
                    .run_at(Timestamp::now_utc() + Duration::from_secs(1))
                    .build(),
            )
            .await
            .unwrap();
    }

    let jobs = TaskTracker::new();
    jobs.close();

    loop {
        if tokio::time::timeout(
            Duration::from_secs(5),
            athena::spawn_jobs(&queue, 20, Arc::new(()), &jobs),
        )
        .await
        .is_err()
        {
            return;
        }

        jobs.wait().await;
        println!("spawned");
    }
}
