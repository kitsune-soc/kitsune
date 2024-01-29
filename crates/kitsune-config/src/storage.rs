use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct FsConfiguration {
    pub upload_dir: SmolStr,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct S3Configuration {
    pub bucket_name: SmolStr,
    pub endpoint_url: SmolStr,
    pub region: SmolStr,
    pub force_path_style: bool,
    pub access_key: SmolStr,
    pub secret_access_key: SmolStr,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum Configuration {
    Fs(FsConfiguration),
    S3(S3Configuration),
}
