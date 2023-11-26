#[macro_use]
extern crate tracing;

use athena::JobQueue;
use kitsune_config::job_queue::Configuration;
use kitsune_db::PgPool;
use kitsune_federation::{
    activitypub::PrepareDeliverer as PrepareActivityPubDeliverer, PrepareDeliverer,
};
use kitsune_federation_filter::FederationFilter;
use kitsune_jobs::{JobRunnerContext, KitsuneContextRepo};
use kitsune_retry_policies::{futures_backoff_policy, RetryPolicy};
use kitsune_service::{attachment::AttachmentService, url::UrlService};
use std::{ops::ControlFlow, sync::Arc, time::Duration};
use tokio::task::JoinSet;
use typed_builder::TypedBuilder;

const EXECUTION_TIMEOUT_DURATION: Duration = Duration::from_secs(30);

#[derive(TypedBuilder)]
pub struct JobDispatcherState {
    attachment_service: AttachmentService,
    db_pool: PgPool,
    federation_filter: FederationFilter,
    url_service: UrlService,
}

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
    state: JobDispatcherState,
    num_job_workers: usize,
) {
    let prepare_activitypub_deliverer = PrepareActivityPubDeliverer::builder()
        .attachment_service(state.attachment_service)
        .db_pool(state.db_pool.clone())
        .federation_filter(state.federation_filter)
        .url_service(state.url_service)
        .build();
    let prepare_deliverer = PrepareDeliverer::builder()
        .activitypub(prepare_activitypub_deliverer)
        .build();

    let ctx = Arc::new(JobRunnerContext {
        db_pool: state.db_pool,
        deliverer: Box::new(kitsune_federation::prepare_deliverer(prepare_deliverer)),
    });

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
