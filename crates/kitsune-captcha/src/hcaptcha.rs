use crate::{error::CaptchaVerification, CaptchaBackend, ChallengeStatus, Result};
use async_trait::async_trait;
use http::Request;
use serde::{Deserialize, Serialize};
use simd_json;
use typed_builder::TypedBuilder;

use kitsune_http_client::Client;

#[derive(Clone, TypedBuilder)]
pub struct Captcha {
    pub verify_url: String,
    pub site_key: String,
    pub secret_key: String,
    #[builder(default = Client::builder().build())]
    client: Client,
}

#[derive(Serialize, Deserialize, TypedBuilder)]
struct Body {
    secret: String,
    response: String,
    #[builder(default = None)]
    remoteip: Option<String>,
    #[builder(default = None)]
    sitekey: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct HCaptchaResponse {
    success: bool, // is the passcode valid, and does it meet security criteria you specified, e.g. sitekey?
    error_codes: Option<Vec<CaptchaVerification>>, // optional: any error codes
}

#[async_trait]
impl CaptchaBackend for Captcha {
    async fn verify(&self, token: &str) -> Result<ChallengeStatus> {
        let body = Body::builder()
            .secret(self.secret_key.to_string())
            .response(token.to_string())
            .build();
        let body = serde_qs::to_string(&body)?;
        let request = Request::post(self.verify_url.clone())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .body(body.into())?;
        let response = self.client.execute(request).await?;
        let mut response_bytes = response.text().await?.into_bytes();
        let verification_result =
            simd_json::serde::from_slice::<HCaptchaResponse>(&mut response_bytes)?;
        if !verification_result.success {
            return Ok(ChallengeStatus::Failed(
                verification_result.error_codes.unwrap_or(Vec::new()),
            ));
        }
        Ok(ChallengeStatus::Verified)
    }
}
