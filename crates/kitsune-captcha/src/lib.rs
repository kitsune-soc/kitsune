#![doc = include_str!("../README.md")]
#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(forbidden_lint_groups)]

use crate::error::Error;
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use error::CaptchaVerification;

pub mod error;
pub mod hcaptcha;
pub mod mcaptcha;

/// Result alias where the error defaults to [`BoxError`]
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Status of the captcha challenge verification
#[derive(PartialEq)]
pub enum ChallengeStatus {
    /// Input successfully sent and verified
    Verified,

    /// Request was successfull but token verification failed
    Failed(Vec<CaptchaVerification>),
}

/// Trait abstraction over captcha backends
#[async_trait]
#[enum_dispatch]
pub trait CaptchaBackend: Clone + Send + Sync {
    /// Verify the token provided in the registration form
    async fn verify(&self, token: &str) -> Result<ChallengeStatus>;
}

#[derive(Clone)]
#[enum_dispatch(CaptchaBackend)]
/// Combined captcha enum for enum dispatch
pub enum Captcha {
    /// hCaptcha
    HCaptcha(hcaptcha::Captcha),

    /// mCaptcha
    MCaptcha(mcaptcha::Captcha),
}
