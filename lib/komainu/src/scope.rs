use compact_str::CompactString;
use indexmap::{set, IndexSet};
use serde::Deserialize;
use std::{
    convert::Infallible,
    fmt::{self, Display},
    str::FromStr,
};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct Scope {
    inner: IndexSet<CompactString>,
}

impl FromStr for Scope {
    type Err = Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.split_whitespace().collect())
    }
}

impl Scope {
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

impl Display for Scope {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in itertools::intersperse(self.iter(), " ") {
            f.write_str(item)?;
        }

        Ok(())
    }
}

impl<Item> FromIterator<Item> for Scope
where
    Item: Into<CompactString>,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = Item>>(iter: T) -> Self {
        iter.into_iter().fold(Scope::new(), |mut acc, item| {
            acc.insert(item.into());
            acc
        })
    }
}

impl IntoIterator for Scope {
    type Item = CompactString;
    type IntoIter = set::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
