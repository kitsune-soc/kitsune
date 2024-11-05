use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::{collections::HashMap, num::NonZero};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AllocationStrategy {
    OnDemand,
    #[default]
    Pooling,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArtifactCache {
    pub path: SmolStr,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct FsKvStorage {
    pub path: SmolStr,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct RedisKvStorage {
    pub url: SmolStr,
    pub pool_size: NonZero<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum KvStorage {
    Fs(FsKvStorage),
    Redis(RedisKvStorage),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    #[serde(default)]
    pub allocation_strategy: AllocationStrategy,
    pub artifact_cache: Option<ArtifactCache>,
    pub module_dir: SmolStr,
    pub module_config: HashMap<SmolStr, SmolStr>,
    pub storage: KvStorage,
}
