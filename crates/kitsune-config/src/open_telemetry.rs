use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Clone, Debug, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Transport {
    Grpc,
    Http,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub metrics_transport: Transport,
    pub metrics_endpoint: SmolStr,
    pub tracing_transport: Transport,
    pub tracing_endpoint: SmolStr,
}
