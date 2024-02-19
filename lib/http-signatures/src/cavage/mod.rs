//!
//! Implementation of Cavage-style HTTP signatures
//!
//! Compliant with <https://datatracker.ietf.org/doc/html/draft-cavage-http-signatures-12> with added opinionated hardenings
//!
//! ## Note
//!
//! The only supported HTTP methods for our hardening checks and the [`easy`] module are GET and POST.
//! This is enough for the intended purpose of this library which is to allow for ActivityPub federation.
//! If you need more methods, feel free to open an issue.
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
