//!
//! Utility functions
//!

use std::ops::Bound;

/// Extension trait on [`Bound`]
pub trait BoundExt<T> {
    /// Vendored until [rust-lang/rust#86026](https://github.com/rust-lang/rust/issues/86026) is stabilised
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Bound<U>;
}

impl<T> BoundExt<T> for Bound<T> {
    #[inline]
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Bound<U> {
        match self {
            Bound::Unbounded => Bound::Unbounded,
            Bound::Included(x) => Bound::Included(f(x)),
            Bound::Excluded(x) => Bound::Excluded(f(x)),
        }
    }
}

/// Vendored until [rust-lang/rust#88581](https://github.com/rust-lang/rust/issues/88581) is stabilised
#[inline]
#[must_use]
pub const fn div_ceil(lhs: usize, rhs: usize) -> usize {
    let d = lhs / rhs;
    let r = lhs % rhs;
    if r > 0 && rhs > 0 {
        d + 1
    } else {
        d
    }
}
