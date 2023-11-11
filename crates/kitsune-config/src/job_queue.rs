use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::num::NonZeroUsize;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub redis_url: SmolStr,
    pub num_workers: NonZeroUsize,
}
