pub mod cache;
pub mod captcha;
pub mod database;
pub mod email;
pub mod embed;
pub mod instance;
pub mod job_queue;
pub mod language_detection;
pub mod messaging;
pub mod oidc;
pub mod open_telemetry;
pub mod search;
pub mod server;
pub mod storage;
pub mod url;

use miette::{Context, IntoDiagnostic};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub cache: cache::Configuration,
    pub captcha: Option<captcha::Configuration>,
    pub database: database::Configuration,
    pub email: Option<email::Configuration>,
    pub embed: Option<embed::Configuration>,
    pub instance: instance::Configuration,
    pub job_queue: job_queue::Configuration,
    pub language_detection: language_detection::Configuration,
    pub messaging: messaging::Configuration,
    pub opentelemetry: Option<open_telemetry::Configuration>,
    pub server: server::Configuration,
    pub search: search::Configuration,
    pub storage: storage::Configuration,
    pub url: url::Configuration,
}

impl Configuration {
    pub async fn load<P>(path: P) -> miette::Result<Self>
    where
        P: AsRef<Path>,
    {
        let content = fs::read_to_string(path)
            .await
            .into_diagnostic()
            .wrap_err("Couldn't read configuration file")?;

        toml::from_str(&content)
            .into_diagnostic()
            .wrap_err("Failed to parse configuration file")
    }
}
