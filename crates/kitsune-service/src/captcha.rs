use kitsune_captcha::{AnyCaptcha, CaptchaBackend, ChallengeStatus};
use kitsune_derive::kitsune_service;
use kitsune_error::Result;

#[kitsune_service]
pub struct CaptchaService {
    #[builder(setter(into))]
    pub backend: Option<AnyCaptcha>,
}

impl CaptchaService {
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.backend.is_some()
    }

    pub async fn verify_token(&self, token: &str) -> Result<ChallengeStatus> {
        match &self.backend {
            Some(backend) => Ok(backend.verify(token).await?),
            _ => Ok(ChallengeStatus::Verified),
        }
    }
}
