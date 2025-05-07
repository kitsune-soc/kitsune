use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::num::NonZero;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct DatabaseConfiguration {
    pub num_workers: NonZero<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct RedisConfiguration {
    pub redis_url: SmolStr,
    pub num_workers: NonZero<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum Configuration {
    Database(DatabaseConfiguration),
    Redis(RedisConfiguration),
}

impl Configuration {
    #[must_use]
    pub fn num_workers(&self) -> NonZero<usize> {
        match self {
            Self::Database(config) => config.num_workers,
            Self::Redis(config) => config.num_workers,
        }
    }
}
