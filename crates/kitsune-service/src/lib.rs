#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions
)]

#[macro_use]
extern crate tracing;

use kitsune_core::consts::API_MAX_LIMIT;

pub mod account;
pub mod attachment;
pub mod captcha;
pub mod custom_emoji;
pub mod error;
pub mod instance;
pub mod job;
pub mod mailing;
pub mod notification;
pub mod post;
pub mod prepare;
pub mod search;
pub mod timeline;
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
