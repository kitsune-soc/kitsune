#[macro_use]
extern crate tracing;

use clap::Parser;
use kitsune::consts::STARTUP_FIGLET;
use kitsune_config::Configuration;
use kitsune_core::consts::VERSION;
use kitsune_job_runner::JobDispatcherState;
use miette::{Context, IntoDiagnostic, MietteDiagnostic};
use std::{env, future, path::PathBuf};
use url::Url;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn postgres_url_diagnostics(db_url: &str) -> String {
    let url = match Url::parse(db_url) {
        Ok(url) => url,
        Err(err) => {
            return format!(
                "Failed to parse the connection string as a URL. Check the syntax!: {err}"
            );
        }
    };

    let message = if url.scheme().starts_with("postgres") && url.has_host() {
        "Your connection string has the correct syntax. Is the host up?"
    } else if url.scheme() == "sqlite" {
        "SQLite is no longer supported as of v0.0.1-pre.1, please use PostgreSQL (our only supported DBMS)\n(This is a temporary diagnostic message and will probably be removed in the future)"
    } else {
        "Your connection string doesn't seem to be valid. Please check it again!"
    };

    message.into()
}

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
    .wrap_err("Failed to connect to and migrate the database")
    .map_err(|err| {
        MietteDiagnostic::new(err.to_string())
            .with_help(postgres_url_diagnostics(&config.database.url))
    })?;

    let job_queue = kitsune_job_runner::prepare_job_queue(conn.clone(), &config.job_queue)
        .into_diagnostic()
        .wrap_err("Failed to connect to the Redis instance for the job scheduler")?;

    let state = kitsune::initialise_state(&config, conn, job_queue.clone()).await?;

    tokio::spawn({
        let server_fut = kitsune::http::run(state.clone(), config.server.clone());

        async move {
            if let Err(error) = server_fut.await {
                error!(?error, "failed to run http server");
            }
        }
    });

    let dispatcher_state = JobDispatcherState::builder()
        .attachment_service(state.service.attachment.clone())
        .db_pool(state.db_pool.clone())
        .federation_filter(state.federation_filter.clone())
        .mail_sender(state.service.mailing.sender())
        .url_service(state.service.url.clone())
        .build();

    tokio::spawn(kitsune_job_runner::run_dispatcher(
        job_queue,
        dispatcher_state,
        config.job_queue.num_workers.get(),
    ));

    future::pending::<()>().await;

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
