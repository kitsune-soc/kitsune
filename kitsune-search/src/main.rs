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

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config: Configuration = envy::from_env().unwrap();
    info!(port = config.port, "Starting up Kitsune search");

    let index = SearchIndex::prepare(&config).unwrap();
    kitsune_search::grpc::start(config, index).await;
}
