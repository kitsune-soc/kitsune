use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub from_address: SmolStr,
    pub host: SmolStr,
    pub username: SmolStr,
    pub password: SmolStr,
    pub starttls: bool,
}
