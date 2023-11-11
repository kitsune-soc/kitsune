use super::oidc;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    #[serde(default)]
    pub clacks_overhead: Vec<SmolStr>,
    pub frontend_dir: SmolStr,
    pub max_upload_size: usize,
    pub media_proxy_enabled: bool,
    pub oidc: Option<oidc::Configuration>,
    pub port: u16,
    pub request_timeout_secs: u64,
}
