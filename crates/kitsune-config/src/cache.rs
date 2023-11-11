use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct RedisConfiguration {
    pub url: SmolStr,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum Configuration {
    Redis(RedisConfiguration),
    InMemory,
    None,
}
