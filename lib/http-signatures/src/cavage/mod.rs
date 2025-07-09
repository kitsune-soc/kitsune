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

pub use self::parse::{ParseError, parse};
pub use self::safety_check::{SafetyCheckError, is_safe};
pub use self::serialise::serialise;

mod parse;
mod safety_check;
mod serialise;

#[cfg(feature = "easy")]
pub mod easy;
pub mod signature_string;

#[derive(Builder, Clone)]
#[builder(vis = "pub(crate)")]
/// Struct representation of the `Signature` HTTP header
pub struct SignatureHeader<'a, I, S> {
    /// Unique identifier of the key this request was signed with
    pub key_id: &'a str,

    /// The headers that are part of the signature
    pub headers: I,

    /// The Base64 encoded signature
    pub signature: S,

    /// (Optional) Unix timestamp in seconds when the signature was created
    #[builder(default, setter(strip_option))]
    pub created: Option<u64>,

    /// (Optional) Unix timestamp in seconds when the signature should be considered invalid
    #[builder(default, setter(strip_option))]
    pub expires: Option<u64>,
}
