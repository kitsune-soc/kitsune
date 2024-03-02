use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct FsKvStorage {
    pub path: SmolStr,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum KvStorage {
    Fs(FsKvStorage),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub module_dir: SmolStr,
    pub module_config: HashMap<SmolStr, SmolStr>,
    pub storage: KvStorage,
}
