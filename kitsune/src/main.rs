use clap::Parser;
use kitsune::consts::STARTUP_FIGLET;
use kitsune_config::Configuration;
use kitsune_core::consts::VERSION;
use kitsune_job_runner::JobDispatcherState;
use miette::{Context, IntoDiagnostic};
use std::{env, path::PathBuf};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Kitsune Social Media server
#[derive(Parser)]
#[command(about, author, version = VERSION)]
struct Args {
    /// Path to the configuration file
    #[clap(long, short)]
    config: PathBuf,
}

async fn boot() -> miette::Result<()> {
    println!("{STARTUP_FIGLET}");

    let args = Args::parse();
    let config = Configuration::load(args.config).await?;
    kitsune_observability::initialise(env!("CARGO_PKG_NAME"), &config)?;

    let conn = kitsune_db::connect(
        &config.database.url,
        config.database.max_connections as usize,
    )
    .await
    .wrap_err("Failed to connect to and migrate the database")?;

    let job_queue = kitsune_job_runner::prepare_job_queue(conn.clone(), &config.job_queue)
        .into_diagnostic()
        .wrap_err("Failed to connect to the Redis instance for the job scheduler")?;

    let state = kitsune::initialise_state(&config, conn, job_queue.clone()).await?;
    let dispatcher_state = JobDispatcherState::builder()
        .attachment_service(state.service.attachment.clone())
        .db_pool(state.db_pool.clone())
        .federation_filter(state.federation_filter.clone())
        .mail_sender(state.service.mailing.sender())
        .url_service(state.service.url.clone())
        .build();

    let shutdown_signal = kitsune::signal::shutdown();

    let server_fut = tokio::spawn(kitsune::http::run(
        state,
        config.server.clone(),
        shutdown_signal.clone(),
    ));
    let job_runner_fut = tokio::spawn(kitsune_job_runner::run_dispatcher(
        job_queue,
        dispatcher_state,
        config.job_queue.num_workers.get(),
    ));

    tokio::select! {
        res = server_fut => res.into_diagnostic()??,
        res = job_runner_fut => res.into_diagnostic()?,
    }

    Ok(())
}

fn main() -> miette::Result<()> {
    miette::set_panic_hook();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(4 * 1024 * 1024) // Set the stack size to 4MiB
        .build()
        .into_diagnostic()?;

    runtime.block_on(boot())
}
