use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub url: SmolStr,
    pub max_connections: u32,
    #[serde(default)]
    pub use_tls: bool,
}
