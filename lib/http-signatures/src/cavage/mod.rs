//!
//! Implementation of Cavage-style HTTP signatures
//!
//! Compliant with <https://datatracker.ietf.org/doc/html/draft-cavage-http-signatures-12> with added opinionated hardenings
//!

use derive_builder::Builder;

pub use self::parse::{parse, ParseError};
pub use self::safety_check::{is_safe, SafetyCheckError};
pub use self::serialise::serialise;

mod parse;
mod safety_check;
mod serialise;

#[cfg(feature = "easy")]
pub mod easy;
pub mod signature_string;

#[derive(Builder, Clone)]
#[builder(vis = "pub(crate)")]
pub struct SignatureHeader<'a, I, S> {
    pub key_id: &'a str,
    pub headers: I,
    pub signature: S,
    #[builder(default, setter(strip_option))]
    pub created: Option<u64>,
    #[builder(default, setter(strip_option))]
    pub expires: Option<u64>,
}
