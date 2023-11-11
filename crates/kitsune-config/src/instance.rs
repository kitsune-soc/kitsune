use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum FederationFilterConfiguration {
    Allow { domains: Vec<SmolStr> },
    Deny { domains: Vec<SmolStr> },
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub allow_non_ascii_usernames: bool,
    pub name: SmolStr,
    pub description: SmolStr,
    pub webfinger_domain: Option<SmolStr>,
    pub character_limit: usize,
    pub federation_filter: FederationFilterConfiguration,
    pub registrations_open: bool,
}
