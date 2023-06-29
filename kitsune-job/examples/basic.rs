use deadpool_redis::{Config, Runtime};
use kitsune_job::{JobDetails, JobQueue};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cfg = Config::from_url("redis://localhost");
    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();

    let queue = JobQueue::builder()
        .queue_name("test_queue")
        .redis_pool(pool)
        .build();

    for _ in 0..100 {
        queue.enqueue(JobDetails::builder().build()).await.unwrap();
    }

    queue.fetch_jobs(20).await.unwrap();
}