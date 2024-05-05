use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::{collections::HashMap, num::NonZeroUsize};

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
    pub pool_size: NonZeroUsize,
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
    pub artifact_cache: Option<ArtifactCache>,
    pub module_dir: SmolStr,
    pub module_config: HashMap<SmolStr, SmolStr>,
    pub storage: KvStorage,
}
