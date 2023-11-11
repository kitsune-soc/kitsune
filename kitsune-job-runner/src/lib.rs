#[macro_use]
extern crate tracing;

use athena::JobQueue;
use kitsune_config::job_queue::Configuration;
use kitsune_core::{
    activitypub::Deliverer,
    job::{JobRunnerContext, KitsuneContextRepo},
    state::State as CoreState,
};
use kitsune_db::PgPool;
use kitsune_retry_policies::{futures_backoff_policy, RetryPolicy};
use std::{ops::ControlFlow, sync::Arc, time::Duration};
use tokio::task::JoinSet;

const EXECUTION_TIMEOUT_DURATION: Duration = Duration::from_secs(30);

pub fn prepare_job_queue(
    db_pool: PgPool,
    config: &Configuration,
) -> Result<JobQueue<KitsuneContextRepo>, deadpool_redis::CreatePoolError> {
    let context_repo = KitsuneContextRepo::builder().db_pool(db_pool).build();
    let redis_pool = deadpool_redis::Config::from_url(config.redis_url.as_str())
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

    let queue = JobQueue::builder()
        .context_repository(context_repo)
        .queue_name("kitsune-jobs")
        .redis_pool(redis_pool)
        .build();

    Ok(queue)
}

#[instrument(skip(job_queue, state))]
pub async fn run_dispatcher(
    job_queue: JobQueue<KitsuneContextRepo>,
    state: CoreState,
    num_job_workers: usize,
) {
    let deliverer = Deliverer::builder()
        .federation_filter(state.service.federation_filter.clone())
        .build();
    let ctx = Arc::new(JobRunnerContext { deliverer, state });

    let mut job_joinset = JoinSet::new();
    loop {
        let mut backoff_policy = futures_backoff_policy();
        loop {
            let result = job_queue
                .spawn_jobs(
                    num_job_workers - job_joinset.len(),
                    Arc::clone(&ctx),
                    &mut job_joinset,
                )
                .await;

            if let ControlFlow::Continue(duration) = backoff_policy.should_retry(result) {
                tokio::time::sleep(duration).await;
            } else {
                break;
            }
        }

        let _ = tokio::time::timeout(EXECUTION_TIMEOUT_DURATION, async {
            while job_joinset.join_next().await.is_some() {}
        })
        .await;
    }
}
