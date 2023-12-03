use clap::Parser;
use color_eyre::eyre;
use kitsune_config::Configuration;
use kitsune_core::consts::VERSION;
use kitsune_federation_filter::FederationFilter;
use kitsune_job_runner::JobDispatcherState;
use kitsune_service::attachment::AttachmentService;
use kitsune_url::UrlService;
use std::path::PathBuf;
use tokio::fs;

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
    let raw_config = fs::read_to_string(args.config).await?;
    let config: Configuration = toml::from_str(&raw_config)?;

    kitsune_observability::initialise(env!("CARGO_PKG_NAME"), &config)?;

    let db_pool = kitsune_db::connect(
        &config.database.url,
        config.database.max_connections as usize,
    )
    .await?;
    let job_queue = kitsune_job_runner::prepare_job_queue(db_pool.clone(), &config.job_queue)?;

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
        .url_service(url_service)
        .build();

    kitsune_job_runner::run_dispatcher(job_queue, state, config.job_queue.num_workers.into()).await;

    Ok(())
}
