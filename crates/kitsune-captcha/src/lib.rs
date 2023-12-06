#![doc = include_str!("../README.md")]

use self::error::CaptchaVerification;
use enum_dispatch::enum_dispatch;

pub mod error;
pub mod hcaptcha;
pub mod mcaptcha;

pub use self::error::Error;

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
#[enum_dispatch]
#[allow(async_fn_in_trait)] // Because of `enum_dispatch`
pub trait CaptchaBackend: Clone + Send + Sync {
    /// Verify the token provided in the registration form
    async fn verify(&self, token: &str) -> Result<ChallengeStatus>;
}

#[derive(Clone)]
#[enum_dispatch(CaptchaBackend)]
/// Combined captcha enum for enum dispatch
pub enum AnyCaptcha {
    /// hCaptcha
    HCaptcha(hcaptcha::Captcha),

    /// mCaptcha
    MCaptcha(mcaptcha::Captcha),
}
