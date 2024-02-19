use http::HeaderName;

pub mod cavage;
pub mod crypto;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

static SIGNATURE_HEADER: HeaderName = HeaderName::from_static("signature");
