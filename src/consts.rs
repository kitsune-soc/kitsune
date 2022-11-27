use once_cell::sync::Lazy;
use regex::Regex;

/// - Capture group 1 -> Username
/// - Caputre group 2 -> Domain
pub static MENTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:^|\W)@([\w\.]+)(?:@(.+\.[[:alnum:]]+))?")
        .expect("Failed to compile mention regex")
});
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
