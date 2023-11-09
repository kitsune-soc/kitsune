use crate::error::Result;
use kitsune_captcha::{AnyCaptcha, CaptchaBackend, ChallengeStatus};
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
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
        if let Some(backend) = &self.backend {
            Ok(backend.verify(token).await?)
        } else {
            Ok(ChallengeStatus::Verified)
        }
    }
}
