#[macro_use]
extern crate tracing;

use athena::{Coerce, JobContextRepository, JobQueue, RedisJobQueue, TaskTracker};
use color_eyre::eyre;
use fred::{
    clients::Pool as RedisPool, interfaces::ClientLike, types::config::Config as RedisConfig,
};
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
) -> eyre::Result<Arc<dyn JobQueue<ContextRepository = KitsuneContextRepo>>> {
    let context_repo = KitsuneContextRepo::builder().db_pool(db_pool).build();

    let config = RedisConfig::from_url(config.redis_url.as_str())?;
    // TODO: Make pool size configurable
    let redis_pool = RedisPool::new(config, None, None, None, 10)?;
    redis_pool.init().await?;

    let queue = RedisJobQueue::builder()
        .conn_pool(redis_pool)
        .context_repository(context_repo)
        .queue_name("kitsune-jobs")
        .build();

    Ok(Arc::new(queue).coerce())
}

#[instrument(skip(http_client, job_queue, state))]
pub async fn run_dispatcher(
    http_client: kitsune_http_client::Client,
    job_queue: Arc<dyn JobQueue<ContextRepository = KitsuneContextRepo> + '_>,
    state: JobDispatcherState,
    num_job_workers: usize,
) {
    let prepare_activitypub_deliverer = PrepareActivityPubDeliverer::builder()
        .attachment_service(state.attachment_service)
        .db_pool(state.db_pool.clone())
        .federation_filter(state.federation_filter)
        .http_client(http_client)
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

    // dunno why the compiler needs this? this is legit a regression.
    //
    // stable -> nightly. but i really cant be bothered anymore to report stuff.
    // i dont like reporting stuff to the rust issue tracker. interactions are always annoying.
    // if i need to write this, then i will. as long as i dont have to deal with the rust teams.
    //
    // because i know, i JUST KNOW, that there will be some response like "oh this is part of a fix. write around it. we dont care."
    // and then i have to write this anyway and wasted my time attempting to report something.
    //
    // happened before. will happen again.
    // i dont have the time nor energy to deal with it. so instead i rant in my source code and then go watch a movie.
    #[allow(clippy::items_after_statements)]
    #[inline]
    fn assert_trait_bounds<CR>(
        item: &(impl JobQueue<ContextRepository = CR> + Clone),
    ) -> &(impl JobQueue<ContextRepository = CR> + Clone)
    where
        CR: JobContextRepository,
    {
        item
    }

    loop {
        let _ = (|| {
            athena::spawn_jobs(
                assert_trait_bounds(&job_queue),
                num_job_workers - job_tracker.len(),
                Arc::clone(&ctx),
                &job_tracker,
            )
        })
        .retry(just_retry::backoff_policy())
        .await;

        let _ = tokio::time::timeout(EXECUTION_TIMEOUT_DURATION, job_tracker.wait()).await;
    }
}
