use compact_str::CompactString;
use serde::Deserialize;
use std::{
    collections::{hash_set, HashSet},
    convert::Infallible,
    str::FromStr,
};

#[derive(Default, Deserialize)]
#[serde(transparent)]
pub struct Scopes {
    inner: HashSet<CompactString>,
}

impl FromStr for Scopes {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.split_whitespace().collect())
    }
}

impl Scopes {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn insert<Item>(&mut self, item: Item)
    where
        Item: Into<CompactString>,
    {
        self.inner.insert(item.into());
    }

    /// Determine whether `self` can be accessed by `resource`
    ///
    /// This implies that `resource` is equal to or a superset of `self`
    #[inline]
    #[must_use]
    pub fn can_be_accessed_by(&self, resource: &Self) -> bool {
        resource.inner.is_superset(&self.inner)
    }

    /// Determine whether `self` is allowed to perform an action
    /// for which you at least need `resource` scope
    #[inline]
    #[must_use]
    pub fn can_perform(&self, resource: &Self) -> bool {
        self.inner.is_superset(&resource.inner)
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.inner.iter().map(CompactString::as_str)
    }
}

impl<Item> FromIterator<Item> for Scopes
where
    Item: Into<CompactString>,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = Item>>(iter: T) -> Self {
        let mut collection = Self::new();
        for item in iter {
            collection.insert(item.into());
        }
        collection
    }
}

impl IntoIterator for Scopes {
    type Item = CompactString;
    type IntoIter = hash_set::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
