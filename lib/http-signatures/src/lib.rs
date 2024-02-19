use http::HeaderName;

pub mod cavage;
pub mod crypto;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

pub static SIGNATURE_HEADER: HeaderName = HeaderName::from_static("signature");
