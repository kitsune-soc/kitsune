use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, ops::Deref};

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum CowStr<'a> {
    Borrowed(&'a str),
    Owned(CompactString),
}

impl CowStr<'_> {
    #[inline]
    #[must_use]
    pub fn into_owned(self) -> CowStr<'static> {
        match self {
            Self::Borrowed(str) => CowStr::Owned(CompactString::from(str)),
            Self::Owned(str) => CowStr::Owned(str),
        }
    }
}

impl Borrow<str> for CowStr<'_> {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

impl Deref for CowStr<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(str) => str,
            Self::Owned(str) => str,
        }
    }
}

impl<'a> From<&'a str> for CowStr<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self::Borrowed(value)
    }
}

impl From<CompactString> for CowStr<'static> {
    #[inline]
    fn from(value: CompactString) -> Self {
        Self::Owned(value)
    }
}
