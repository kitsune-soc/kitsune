#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]

use kitsune::{config::Configuration, http, job};
use std::{env, future, process};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, Layer, Registry};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

fn initialise_logging(_config: &Configuration) {
    let env_filter = env::var("RUST_LOG").map_or_else(
        |_| Targets::default().with_default(LevelFilter::INFO),
        |targets| targets.parse().expect("Failed to parse RUST_LOG value"),
    );
    let subscriber =
        Registry::default().with(tracing_subscriber::fmt::layer().with_filter(env_filter));

    #[cfg(feature = "metrics")]
    #[allow(clippy::used_underscore_binding)]
    let subscriber = subscriber.with(initialise_metrics(_config));

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[tokio::main]
async fn main() {
    println!("{STARTUP_FIGLET}");

    let args: Vec<String> = env::args().take(2).collect();
    if args.len() == 1 {
        eprintln!("Usage: {} <Path to configuration file>", args[0]);
        process::exit(1);
    }

    let config = match Configuration::load(&args[1]) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };
    initialise_logging(&config);

    let conn = kitsune_db::connect(
        &config.database.url,
        config.database.max_connections as usize,
    )
    .await
    .expect("Failed to connect to database");
    let state = kitsune::initialise_state(&config, conn).await;

    tokio::spawn(self::http::run(state.clone(), config.server.clone()));
    tokio::spawn(self::job::run_dispatcher(
        state.clone(),
        config.server.job_workers,
    ));

    future::pending::<()>().await;
}
