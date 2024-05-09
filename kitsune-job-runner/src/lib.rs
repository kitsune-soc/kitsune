#[macro_use]
extern crate tracing;

use athena::{Coerce, JobQueue, RedisJobQueue, TaskTracker};
use just_retry::RetryExt;
use kitsune_config::job_queue::Configuration;
use kitsune_db::PgPool;
use kitsune_email::{
    lettre::{AsyncSmtpTransport, Tokio1Executor},
    MailSender, MailingService,
};
use kitsune_federation::{
    activitypub::PrepareDeliverer as PrepareActivityPubDeliverer, PrepareDeliverer,
};
use kitsune_federation_filter::FederationFilter;
use kitsune_jobs::{JobRunnerContext, KitsuneContextRepo, Service};
use kitsune_service::attachment::AttachmentService;
use kitsune_url::UrlService;
use kitsune_wasm_mrf::MrfService;
use multiplex_pool::RoundRobinStrategy;
use redis::RedisResult;
use std::time::Duration;
use triomphe::Arc;
use typed_builder::TypedBuilder;

const EXECUTION_TIMEOUT_DURATION: Duration = Duration::from_secs(30);

#[derive(TypedBuilder)]
pub struct JobDispatcherState {
    attachment_service: AttachmentService,
    db_pool: PgPool,
    federation_filter: FederationFilter,
    mail_sender: Option<MailSender<AsyncSmtpTransport<Tokio1Executor>>>,
    mrf_service: MrfService,
    url_service: UrlService,
}

pub async fn prepare_job_queue(
    db_pool: PgPool,
    config: &Configuration,
) -> RedisResult<Arc<dyn JobQueue<ContextRepository = KitsuneContextRepo>>> {
    let context_repo = KitsuneContextRepo::builder().db_pool(db_pool).build();

    let client = redis::Client::open(config.redis_url.as_str())?;
    let redis_pool = multiplex_pool::Pool::from_producer(
        || client.get_connection_manager(),
        10,
        RoundRobinStrategy::default(),
    )
    .await?;

    let queue = RedisJobQueue::builder()
        .context_repository(context_repo)
        .queue_name("kitsune-jobs")
        .redis_pool(redis_pool)
        .build();

    Ok(Arc::new(queue).coerce())
}

#[instrument(skip(job_queue, state))]
pub async fn run_dispatcher(
    job_queue: Arc<dyn JobQueue<ContextRepository = KitsuneContextRepo>>,
    state: JobDispatcherState,
    num_job_workers: usize,
) {
    let prepare_activitypub_deliverer = PrepareActivityPubDeliverer::builder()
        .attachment_service(state.attachment_service)
        .db_pool(state.db_pool.clone())
        .federation_filter(state.federation_filter)
        .mrf_service(state.mrf_service)
        .url_service(state.url_service.clone())
        .build();
    let prepare_deliverer = PrepareDeliverer::builder()
        .activitypub(prepare_activitypub_deliverer)
        .build();

    let mailing_service = MailingService::builder()
        .db_pool(state.db_pool.clone())
        .sender(state.mail_sender)
        .url_service(state.url_service)
        .build();

    let ctx = Arc::new(JobRunnerContext {
        db_pool: state.db_pool,
        deliverer: Box::new(kitsune_federation::prepare_deliverer(prepare_deliverer)),
        service: Service {
            mailing: mailing_service,
        },
    });

    let job_tracker = TaskTracker::new();
    job_tracker.close();

    loop {
        let _ = (|| {
            let job_queue = Arc::clone(&job_queue);
            let ctx = Arc::clone(&ctx);
            let job_tracker = job_tracker.clone();

            async move {
                athena::spawn_jobs(
                    &job_queue,
                    num_job_workers - job_tracker.len(),
                    Arc::clone(&ctx),
                    &job_tracker,
                )
                .await
            }
        })
        .retry(just_retry::backoff_policy())
        .await;

        let _ = tokio::time::timeout(EXECUTION_TIMEOUT_DURATION, job_tracker.wait()).await;
    }
}
