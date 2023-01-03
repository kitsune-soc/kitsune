use fancy_regex::Regex;
use once_cell::sync::Lazy;

/// - Capture group 1 -> Username
/// - Caputre group 2 -> Domain
pub static MENTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?<=\W|^)@([\w\.]+)(?:@([\pL\.\-]+[\pL]+))")
        .expect("Failed to compile mention regex")
});
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
