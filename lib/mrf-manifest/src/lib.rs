//!
//! Rust definition of the MRF manifest
//!
//! Includes some utility functions for parsing/encoding
//!

#![deny(missing_docs)]

use schemars::{schema::RootSchema, JsonSchema};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[cfg(feature = "parse")]
pub use self::decode::{decode, DecodeError, SectionRange};
#[cfg(feature = "encode")]
pub use self::encode::encode;
#[cfg(feature = "serialise")]
pub use self::serialise::serialise;

#[cfg(feature = "parse")]
mod decode;
#[cfg(feature = "encode")]
mod encode;
#[cfg(feature = "serialise")]
mod serialise;

/// Name of the section the manifest has to be encoded to
pub const SECTION_NAME: &str = "manifest-v0";

/// Wrapper around a hash set intended for use with the `activityTypes` field
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
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

/// Version of the API used
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum ApiVersion {
    /// Version 1
    V1,
}

/// Manifest of MRF modules
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", tag = "manifestVersion")]
#[non_exhaustive]
pub enum Manifest<'a> {
    /// Manifest v1
    #[serde(borrow)]
    V1(ManifestV1<'a>),
}

/// Manifest v1
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
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

    /// JSON schema of the configuration passed to the module
    ///
    /// This is optional but can be used for automatically generating a configuration UI
    pub config_schema: Option<RootSchema>,
}
