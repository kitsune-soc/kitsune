#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc, forbidden_lint_groups)]

pub mod cache;
pub mod captcha;
pub mod database;
pub mod email;
pub mod embed;
pub mod instance;
pub mod job_queue;
pub mod messaging;
pub mod oidc;
pub mod open_telemetry;
pub mod search;
pub mod server;
pub mod storage;
pub mod url;

use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub cache: cache::Configuration,
    pub captcha: Option<captcha::Configuration>,
    pub database: database::Configuration,
    pub email: Option<email::Configuration>,
    pub embed: Option<embed::Configuration>,
    pub instance: instance::Configuration,
    pub job_queue: job_queue::Configuration,
    pub messaging: messaging::Configuration,
    pub opentelemetry: Option<open_telemetry::Configuration>,
    pub server: server::Configuration,
    pub search: search::Configuration,
    pub storage: storage::Configuration,
    pub url: url::Configuration,
}

impl Configuration {
    pub async fn load<P>(path: P) -> eyre::Result<Self>
    where
        P: AsRef<Path>,
    {
        let content = fs::read_to_string(path).await?;
        toml::from_str(&content).map_err(eyre::Report::from)
    }
}
