mod parse;

pub use self::parse::parse;

#[derive(Clone)]
pub struct SignatureHeader<'a, I> {
    pub key_id: &'a str,
    pub headers: I,
    pub signature: &'a str,
    pub created: Option<u64>,
    pub expires: Option<u64>,
}
