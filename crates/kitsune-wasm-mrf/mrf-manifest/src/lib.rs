use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[cfg(feature = "parse")]
pub use self::parse::ParseError;

#[cfg(feature = "parse")]
mod parse;
#[cfg(feature = "serialise")]
mod serialise;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct ActivitySet<'a>(#[serde(borrow)] pub ahash::HashSet<&'a str>);

impl ActivitySet<'_> {
    /// Does the set of requested activity types contain `*`?
    pub fn all_activities(&self) -> bool {
        self.0.contains("*")
    }
}

impl<'a> Deref for ActivitySet<'a> {
    type Target = ahash::HashSet<&'a str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ActivitySet<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> From<ActivitySet<'a>> for ahash::HashSet<&'a str> {
    fn from(value: ActivitySet<'a>) -> Self {
        value.0
    }
}

impl<'a> From<ahash::HashSet<&'a str>> for ActivitySet<'a> {
    fn from(value: ahash::HashSet<&'a str>) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum ApiVersion {
    V1,
}

/// Manifest of MRF modules
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", tag = "manifestVersion")]
#[non_exhaustive]
pub enum Manifest<'a> {
    #[serde(borrow)]
    V1(ManifestV1<'a>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ManifestV1<'a> {
    /// Version of the MRF API
    pub api_version: ApiVersion,

    /// Name of the MRF module
    pub name: &'a str,

    /// Version of the MRF module
    pub version: semver::Version,

    /// Activity types passed to the MRF module
    ///
    /// `*` matching all types
    pub activity_types: ActivitySet<'a>,
}
