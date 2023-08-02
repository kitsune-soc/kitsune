#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]

use color_eyre::{config::HookBuilder, Help};
use eyre::Context;
use kitsune::{config::Configuration, consts::VERSION, http, job};
use std::{
    borrow::Cow,
    env, future,
    panic::{self, PanicInfo},
    process,
};
use tracing::level_filters::LevelFilter;
use tracing_error::ErrorLayer;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, Layer, Registry};
use url::Url;

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

const STARTUP_FIGLET: &str = r#"
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃                                                           ┃
┃  ██╗  ██╗██╗████████╗███████╗██╗   ██╗███╗   ██╗███████╗  ┃
┃  ██║ ██╔╝██║╚══██╔══╝██╔════╝██║   ██║████╗  ██║██╔════╝  ┃
┃  █████╔╝ ██║   ██║   ███████╗██║   ██║██╔██╗ ██║█████╗    ┃
┃  ██╔═██╗ ██║   ██║   ╚════██║██║   ██║██║╚██╗██║██╔══╝    ┃
┃  ██║  ██╗██║   ██║   ███████║╚██████╔╝██║ ╚████║███████╗  ┃
┃  ╚═╝  ╚═╝╚═╝   ╚═╝   ╚══════╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝  ┃
┃                                                           ┃
┃            ActivityPub-federated microblogging            ┃
┃                                                           ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
"#;

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

#[cfg(feature = "metrics")]
fn initialise_metrics<S>(config: &Configuration) -> impl Layer<S>
where
    S: for<'a> tracing_subscriber::registry::LookupSpan<'a> + tracing::Subscriber,
{
    use axum_prometheus::{AXUM_HTTP_REQUESTS_DURATION_SECONDS, SECONDS_DURATION_BUCKETS};
    use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
    use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
    use metrics_util::layers::Layer as _;

    let (prometheus_recorder, server_future) = PrometheusBuilder::new()
        // Some defaults that would have been set by the `axum-prometheus` crate
        .set_buckets_for_metric(
            Matcher::Full(AXUM_HTTP_REQUESTS_DURATION_SECONDS.to_string()),
            SECONDS_DURATION_BUCKETS,
        )
        .unwrap()
        .with_http_listener(([0, 0, 0, 0], config.server.prometheus_port))
        .build()
        .unwrap();
    tokio::spawn(server_future);

    let recorder = TracingContextLayer::all().layer(prometheus_recorder);
    metrics::set_boxed_recorder(Box::new(recorder)).unwrap();

    MetricsLayer::new()
}

fn initialise_logging(_config: &Configuration) -> eyre::Result<()> {
    let env_filter = env::var("RUST_LOG")
        .map_err(eyre::Report::from)
        .and_then(|targets| targets.parse().context("Failed to parse RUST_LOG value"))
        .unwrap_or_else(|_| Targets::default().with_default(LevelFilter::INFO));

    let subscriber = Registry::default()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter))
        .with(ErrorLayer::default());

    #[cfg(feature = "metrics")]
    #[allow(clippy::used_underscore_binding)]
    let subscriber = subscriber.with(initialise_metrics(_config));

    tracing::subscriber::set_global_default(subscriber)
        .context("Couldn't install the global tracing subscriber")?;

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

#[tokio::main]
async fn main() -> eyre::Result<()> {
    install_handlers()?;

    println!("{STARTUP_FIGLET}");

    let args: Vec<String> = env::args().take(2).collect();
    if args.len() == 1 {
        eprintln!("Usage: {} <Path to configuration file>", args[0]);
        process::exit(1);
    }

    let config = Configuration::load(&args[1]).await?;
    initialise_logging(&config)?;

    let conn = kitsune_db::connect(
        &config.database.url,
        config.database.max_connections as usize,
    )
    .await
    .context("Failed to connect to and migrate the database")
    .with_suggestion(|| postgres_url_diagnostics(&config.database.url))?;

    let job_queue = kitsune::prepare_job_queue(conn.clone(), &config.job_queue)
        .context("Failed to connect to the Redis instance for the job scheduler")?;
    let state = kitsune::initialise_state(&config, conn, job_queue.clone()).await?;

    tokio::spawn(self::http::run(state.clone(), config.server.clone()));
    tokio::spawn(self::job::run_dispatcher(
        job_queue,
        state.clone(),
        config.job_queue.num_workers.get(),
    ));

    future::pending::<()>().await;

    Ok(())
}
