use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct RedisStoreConfiguration {
    pub url: SmolStr,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum StoreConfiguration {
    InMemory,
    Redis(RedisStoreConfiguration),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub store: StoreConfiguration,
    pub server_url: SmolStr,
    pub client_id: SmolStr,
    pub client_secret: SmolStr,
}
