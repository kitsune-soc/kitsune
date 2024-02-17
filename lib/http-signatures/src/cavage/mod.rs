//!
//! Implementation of Cavage-style HTTP signatures
//!
//! Compliant with <https://datatracker.ietf.org/doc/html/draft-cavage-http-signatures-12> with added opinionated hardenings
//!

use derive_builder::Builder;

pub use self::parse::{parse, ParseError};
pub use self::sign::{sign, SigningKey};

mod parse;
mod sign;

pub mod signature_string;

#[derive(Builder, Clone)]
#[builder(vis = "pub(crate)")]
pub struct SignatureHeader<'a, I> {
    pub key_id: &'a str,
    pub headers: I,
    pub signature: &'a str,
    #[builder(default, setter(strip_option))]
    pub created: Option<u64>,
    #[builder(default, setter(strip_option))]
    pub expires: Option<u64>,
}
