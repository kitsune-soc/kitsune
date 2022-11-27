#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::doc_markdown,
    clippy::module_name_repetitions,
    forbidden_lint_groups
)]

use self::{config::Configuration, fetcher::Fetcher, state::State, webfinger::Webfinger};
use std::future;

#[macro_use]
extern crate tracing;

mod blocking;
mod config;
mod consts;
mod db;
mod deliverer;
mod error;
mod fetcher;
mod http;
mod job;
mod mapping;
mod state;
mod util;
mod webfinger;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config: Configuration = envy::from_env().expect("Failed to parse configuration");
    let conn = self::db::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let state = State {
        config: config.clone(),
        db_conn: conn.clone(),
        fetcher: Fetcher::new(conn),
        webfinger: Webfinger::new(),
    };

    tokio::spawn(self::http::run(state.clone(), config.port));

    for _ in 0..config.job_workers.get() {
        tokio::spawn(self::job::run(state.clone()));
    }

    future::pending::<()>().await;
}
