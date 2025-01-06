use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, ops::Deref};

const _: () = {
    assert!(std::mem::size_of::<CowStr<'static>>() == std::mem::size_of::<String>());
};

#[derive(Clone, Debug, Deserialize, Serialize, Hash)]
#[serde(untagged)]
pub enum CowStr<'a> {
    Borrowed(&'a str),
    Owned(CompactString),
}

impl<'a> CowStr<'a> {
    #[inline]
    #[must_use]
    pub fn borrowed(str: &'a str) -> Self {
        Self::Borrowed(str)
    }

    #[inline]
    #[must_use]
    pub fn owned(str: impl Into<CompactString>) -> Self {
        Self::Owned(str.into())
    }

    #[inline]
    #[must_use]
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(..))
    }

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

impl PartialEq for CowStr<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let lhs: &str = self.borrow();
        let rhs: &str = other.borrow();

        lhs.eq(rhs)
    }
}

impl Eq for CowStr<'_> {}

impl PartialOrd for CowStr<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CowStr<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let lhs: &str = self.borrow();
        let rhs: &str = other.borrow();

        lhs.cmp(rhs)
    }
}

#[cfg(test)]
mod test {
    use crate::CowStr;
    use compact_str::CompactString;
    use std::borrow::Borrow;

    const TEST_STR: &str = "hello world";

    #[test]
    fn borrowed() {
        let borrowed_1 = CowStr::borrowed(TEST_STR);
        let borrowed_2: CowStr<'_> = TEST_STR.into();

        assert!(!borrowed_1.is_owned());
        assert!(!borrowed_2.is_owned());

        assert_eq!(borrowed_1, borrowed_2);
        assert_eq!(Borrow::<str>::borrow(&borrowed_1), TEST_STR);
    }

    #[test]
    fn owned() {
        let owned_1 = CowStr::owned(TEST_STR);
        let owned_2: CowStr<'_> = CompactString::from(TEST_STR).into();

        assert!(owned_1.is_owned());
        assert!(owned_2.is_owned());

        assert_eq!(owned_1, owned_2);
        assert_eq!(Borrow::<str>::borrow(&owned_1), TEST_STR);
    }

    #[test]
    fn into_owned() {
        let borrowed = CowStr::borrowed(TEST_STR);
        let owned = {
            let cloned = borrowed.clone();
            assert!(!cloned.is_owned());
            cloned.into_owned()
        };

        assert!(!borrowed.is_owned());
        assert!(owned.is_owned());

        assert_eq!(borrowed, owned);
    }
}
