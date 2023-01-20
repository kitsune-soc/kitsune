#![forbid(rust_2018_idioms, unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    forbidden_lint_groups,
    clippy::cast_possible_truncation,
    clippy::module_name_repetitions
)]

#[macro_use]
extern crate tracing;

use kitsune_search::{config::Configuration, search::SearchIndex};
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Layer as _;
use std::env;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{filter::Targets, layer::SubscriberExt, Layer as _, Registry};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config: Configuration = envy::from_env().unwrap();

    let (prometheus_recorder, server_future) = PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], config.prometheus_port))
        .build()
        .unwrap();
    tokio::spawn(server_future);

    let env_filter = env::var("RUST_LOG").map_or_else(
        |_| Targets::default().with_default(LevelFilter::INFO),
        |targets| targets.parse().expect("Failed to parse RUST_LOG value"),
    );
    let subscriber = Registry::default()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter))
        .with(MetricsLayer::new());
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install tracing subscriber");

    let recorder = TracingContextLayer::all().layer(prometheus_recorder);
    metrics::set_boxed_recorder(Box::new(recorder)).expect("failed to install metrics recorder");

    info!(port = config.port, "Starting up Kitsune search");

    let index = SearchIndex::prepare(&config).unwrap();
    kitsune_search::grpc::start(config, index).await;
}
