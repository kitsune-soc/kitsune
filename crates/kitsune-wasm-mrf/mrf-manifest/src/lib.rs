use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum ApiVersion {
    V1,
}

/// Manifest of MRF modules
#[derive(Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", tag = "manifestVersion")]
#[non_exhaustive]
pub enum Manifest<'a> {
    #[serde(borrow)]
    V1(ManifestV1<'a>),
}

#[derive(Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestV1<'a> {
    /// Version of the MRF API
    pub api_version: ApiVersion,

    /// Name of the MRF module
    #[serde(borrow)]
    pub name: Cow<'a, str>,

    /// Version of the MRF module
    #[serde(borrow)]
    pub version: Cow<'a, str>,

    /// Activity types passed to the MRF module
    ///
    /// `*` matching all types
    #[serde(borrow)]
    pub activity_types: Cow<'a, [Cow<'a, str>]>,
}
