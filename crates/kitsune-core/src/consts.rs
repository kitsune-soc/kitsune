use const_format::concatcp;
use git_version::git_version;

pub const API_MAX_LIMIT: usize = 40;
pub const MAX_EMOJI_SHORTCODE_LENGTH: usize = 64;
pub const MAX_MEDIA_DESCRIPTION_LENGTH: usize = 5000;
pub const PROJECT_IDENTIFIER: &str = env!("CARGO_PRIMARY_PACKAGE");
pub const USER_AGENT: &str = concatcp!(PROJECT_IDENTIFIER, "/", VERSION);
pub const VERSION: &str = concatcp!(env!("CARGO_PKG_VERSION"), "-", git_version!());
