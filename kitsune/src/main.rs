#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]

#[macro_use]
extern crate tracing;

use clap::Parser;
use color_eyre::{config::HookBuilder, Help};
use eyre::Context;
use kitsune::consts::STARTUP_FIGLET;
use kitsune_config::Configuration;
use kitsune_core::consts::VERSION;
use std::{
    borrow::Cow,
    env, future,
    panic::{self, PanicInfo},
    path::PathBuf,
};
use url::Url;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn install_handlers() -> eyre::Result<()> {
    let (eyre_panic_hook, eyre_hook) = HookBuilder::new().into_hooks();
    let metadata = human_panic::Metadata {
        version: Cow::Borrowed(VERSION),
        ..human_panic::metadata!()
    };

    let eyre_panic_hook = move |panic_info: &PanicInfo<'_>| {
        eprintln!("{}", eyre_panic_hook.panic_report(panic_info));
    };
    let human_panic_hook = move |panic_info: &PanicInfo<'_>| {
        let path = human_panic::handle_dump(&metadata, panic_info);
        human_panic::print_msg(path, &metadata).ok();
    };

    eyre_hook.install()?;
    panic::set_hook(Box::new(move |panic_info| {
        let hook: &(dyn Fn(&PanicInfo<'_>) + Send + Sync) =
            if cfg!(debug_assertions) || env::var("RUST_BACKTRACE").is_ok() {
                &eyre_panic_hook
            } else {
                &human_panic_hook
            };

        hook(panic_info);
    }));

    Ok(())
}

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

async fn boot() -> eyre::Result<()> {
    println!("{STARTUP_FIGLET}");

    let args = Args::parse();
    let config = Configuration::load(args.config).await?;
    kitsune_observability::initialise(env!("CARGO_PKG_NAME"), &config)?;

    let conn = kitsune_db::connect(
        &config.database.url,
        config.database.max_connections as usize,
    )
    .await
    .context("Failed to connect to and migrate the database")
    .with_suggestion(|| postgres_url_diagnostics(&config.database.url))?;

    let job_queue = kitsune_job_runner::prepare_job_queue(conn.clone(), &config.job_queue)
        .context("Failed to connect to the Redis instance for the job scheduler")?;
    let state = kitsune::initialise_state(&config, conn, job_queue.clone()).await?;

    tokio::spawn({
        let server_fut = kitsune::http::run(state.clone(), config.server.clone());

        async move {
            if let Err(error) = server_fut.await {
                error!(?error, "failed to run http server");
            }
        }
    });
    tokio::spawn(kitsune_job_runner::run_dispatcher(
        job_queue,
        state.core.clone(),
        config.job_queue.num_workers.get(),
    ));

    future::pending::<()>().await;

    Ok(())
}

fn main() -> eyre::Result<()> {
    install_handlers()?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(4 * 1024 * 1024) // Set the stack size to 4MiB
        .build()?;

    runtime.block_on(boot())
}
