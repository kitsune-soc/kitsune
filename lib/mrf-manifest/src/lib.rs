//!
//! Rust definition of the MRF manifest
//!
//! Includes some utility functions for parsing/encoding
//!

#![deny(missing_docs)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::BTreeSet,
    ops::{Deref, DerefMut},
};

#[cfg(feature = "decode")]
pub use self::decode::{decode, DecodeError, SectionRange};
#[cfg(feature = "encode")]
pub use self::encode::encode;
#[cfg(feature = "serialise")]
pub use self::serialise::serialise;

#[cfg(feature = "decode")]
mod decode;
#[cfg(feature = "encode")]
mod encode;
#[cfg(feature = "serialise")]
mod serialise;

/// Name of the section the manifest has to be encoded to
pub const SECTION_NAME: &str = "manifest-v0";

#[inline]
fn cow_to_static<T>(cow: Cow<'_, T>) -> Cow<'static, T>
where
    T: ToOwned + ?Sized,
{
    Cow::Owned(cow.into_owned())
}

/// Wrapper around a hash set intended for use with the `activityTypes` field
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ActivitySet<'a>(#[serde(borrow)] pub BTreeSet<Cow<'a, str>>);

impl ActivitySet<'_> {
    /// Does the set of requested activity types contain `*`?
    #[must_use]
    pub fn all_activities(&self) -> bool {
        self.0.contains("*")
    }

    /// Turn a borrowed version of `ActivitySet` into a version with a `'static` lifetime
    ///
    /// This might allocate a bunch.
    pub fn to_owned(&self) -> ActivitySet<'static> {
        self.0
            .iter()
            .cloned()
            .map(cow_to_static)
            .collect::<BTreeSet<Cow<'static, str>>>()
            .into()
    }
}

impl<'a> Deref for ActivitySet<'a> {
    type Target = BTreeSet<Cow<'a, str>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ActivitySet<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> From<ActivitySet<'a>> for BTreeSet<Cow<'a, str>> {
    fn from(value: ActivitySet<'a>) -> Self {
        value.0
    }
}

impl<'a> From<BTreeSet<Cow<'a, str>>> for ActivitySet<'a> {
    fn from(value: BTreeSet<Cow<'a, str>>) -> Self {
        Self(value)
    }
}

/// Version of the API used
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum ApiVersion {
    /// Version 1
    V1,
}

/// Manifest of MRF modules
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "manifestVersion")]
#[non_exhaustive]
pub enum Manifest<'a> {
    /// Manifest v1
    #[serde(borrow)]
    V1(ManifestV1<'a>),
}

impl Manifest<'_> {
    /// Turn a borrowed version of `Manifest` into a version with a `'static` lifetime
    ///
    /// This might allocate a bunch.
    #[must_use]
    pub fn to_owned(&self) -> Manifest<'static> {
        match self {
            Self::V1(v1) => Manifest::V1(v1.to_owned()),
        }
    }
}

/// Manifest v1
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestV1<'a> {
    /// Version of the MRF API
    pub api_version: ApiVersion,

    /// Name of the MRF module
    pub name: Cow<'a, str>,

    /// Version of the MRF module
    pub version: semver::Version,

    /// Activity types passed to the MRF module
    ///
    /// `*` matching all types
    #[serde(borrow)]
    pub activity_types: ActivitySet<'a>,

    /// JSON schema of the configuration passed to the module
    ///
    /// This is optional but can be used for automatically generating a configuration UI
    pub config_schema: Option<schemars::Schema>,
}

impl ManifestV1<'_> {
    /// Turn a borrowed version of `ManifestV1` into a version with a `'static` lifetime
    ///
    /// This might allocate a bunch.
    #[must_use]
    pub fn to_owned(&self) -> ManifestV1<'static> {
        ManifestV1 {
            api_version: self.api_version,
            name: cow_to_static(self.name.clone()),
            activity_types: self.activity_types.to_owned(),
            version: self.version.clone(),
            config_schema: self.config_schema.clone(),
        }
    }
}
