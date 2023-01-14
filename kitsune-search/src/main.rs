#![forbid(rust_2018_idioms, unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(forbidden_lint_groups, clippy::module_name_repetitions)]

#[macro_use]
extern crate tracing;

use self::config::Configuration;
use std::future;

mod config;
mod grpc;
mod search;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config: Configuration = envy::from_env().unwrap();
    info!(port = config.port, "Starting up Kitsune search");

    let index = self::search::prepare_index(&config).unwrap();
    tokio::spawn(self::grpc::start(config, index));

    future::pending::<()>().await;
}
