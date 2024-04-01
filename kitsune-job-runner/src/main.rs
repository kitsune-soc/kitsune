use clap::Parser;
use color_eyre::eyre;
use kitsune_config::Configuration;
use kitsune_core::consts::VERSION;
use kitsune_federation_filter::FederationFilter;
use kitsune_job_runner::JobDispatcherState;
use kitsune_service::{attachment::AttachmentService, prepare};
use kitsune_url::UrlService;
use kitsune_wasm_mrf::MrfService;
use std::path::PathBuf;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Dedicated Kitsune job runner
#[derive(Parser)]
#[command(about, author, version = VERSION)]
struct Args {
    /// Path to the configuration
    #[arg(long, short)]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let config = Configuration::load(args.config).await?;

    kitsune_observability::initialise(env!("CARGO_PKG_NAME"), &config)?;

    let db_pool = kitsune_db::connect(&config.database).await?;
    let job_queue =
        kitsune_job_runner::prepare_job_queue(db_pool.clone(), &config.job_queue).await?;

    let mrf_service = MrfService::from_config(&config.mrf).await?;
    let url_service = UrlService::builder()
        .domain(config.url.domain)
        .scheme(config.url.scheme)
        .webfinger_domain(config.instance.webfinger_domain)
        .build();
    let attachment_service = AttachmentService::builder()
        .db_pool(db_pool.clone())
        .media_proxy_enabled(config.server.media_proxy_enabled)
        .storage_backend(kitsune_service::prepare::storage(&config.storage)?)
        .url_service(url_service.clone())
        .build();
    let federation_filter = FederationFilter::new(&config.instance.federation_filter)?;

    let state = JobDispatcherState::builder()
        .attachment_service(attachment_service)
        .db_pool(db_pool)
        .federation_filter(federation_filter)
        .mail_sender(
            config
                .email
                .as_ref()
                .map(prepare::mail_sender)
                .transpose()?,
        )
        .mrf_service(mrf_service)
        .url_service(url_service)
        .build();

    kitsune_job_runner::run_dispatcher(job_queue, state, config.job_queue.num_workers.into()).await;

    Ok(())
}
