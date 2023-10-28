use crate::consts::API_MAX_LIMIT;

pub mod account;
pub mod attachment;
pub mod captcha;
pub mod federation_filter;
pub mod instance;
pub mod job;
pub mod mailing;
pub mod notification;
pub mod post;
pub mod search;
pub mod timeline;
pub mod url;
pub mod user;

pub struct LimitContext {
    limit: usize,
}

impl Default for LimitContext {
    fn default() -> Self {
        Self {
            limit: API_MAX_LIMIT,
        }
    }
}
