use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct MeilisearchConfiguration {
    pub instance_url: SmolStr,
    pub api_key: SmolStr,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum Configuration {
    Meilisearch(MeilisearchConfiguration),
    Sql,
    None,
}
