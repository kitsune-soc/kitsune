//!
//! Utility functions
//!

/// Vendored until [rust-lang/rust#88581](https://github.com/rust-lang/rust/issues/88581) is stabilised
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
