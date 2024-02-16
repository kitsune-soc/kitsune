use derive_builder::Builder;

pub use self::parse::{parse, ParseError};

mod parse;

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
