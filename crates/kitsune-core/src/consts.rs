use const_format::concatcp;

pub const API_MAX_LIMIT: usize = 40;
pub const MAX_EMOJI_SHORTCODE_LENGTH: usize = 64;
pub const MAX_MEDIA_DESCRIPTION_LENGTH: usize = 5000;
pub const USER_AGENT: &str = concatcp!(env!("CARGO_PKG_NAME"), "/", VERSION);
pub const VERSION: &str = concatcp!(env!("CARGO_PKG_VERSION"), "-", env!("VERGEN_GIT_SHA"));
