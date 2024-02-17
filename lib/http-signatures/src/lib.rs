pub mod cavage;
pub mod crypto;
pub mod easy;

pub const REQUIRED_GET_HEADERS: &[&str] = &["host", "date"];
pub const REQUIRED_POST_HEADERS: &[&str] = &["host", "date", "content-type", "digest"];
