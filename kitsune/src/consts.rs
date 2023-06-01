use const_format::concatcp;

pub const USER_AGENT: &str = concatcp!(env!("CARGO_PKG_NAME"), "/", VERSION);
pub const VERSION: &str = concatcp!(env!("CARGO_PKG_VERSION"), "-", env!("VERGEN_GIT_SHA"));
