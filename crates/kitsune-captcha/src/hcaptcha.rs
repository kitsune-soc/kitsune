use crate::{error::CaptchaVerification, CaptchaBackend, ChallengeStatus, Result};
use bytes::Bytes;
use http::Request;
use http_body_util::Full;
use kitsune_http_client::Client;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct Captcha {
    pub verify_url: String,
    pub site_key: String,
    pub secret_key: String,

    #[builder(default)]
    client: Client,
}

#[derive(Serialize, Deserialize, TypedBuilder)]
struct Body {
    secret: String,
    response: String,

    #[builder(default)]
    remoteip: Option<String>,

    #[builder(default)]
    sitekey: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct HCaptchaResponse {
    success: bool, // is the passcode valid, and does it meet security criteria you specified, e.g. sitekey?
    error_codes: Option<Vec<CaptchaVerification>>, // optional: any error codes
}

impl CaptchaBackend for Captcha {
    async fn verify(&self, token: &str) -> Result<ChallengeStatus> {
        let body = Body::builder()
            .secret(self.secret_key.to_string())
            .response(token.to_string())
            .build();
        let body = serde_urlencoded::to_string(&body)?;

        let request: Request<Full<Bytes>> = Request::post(self.verify_url.clone())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            .body(body.into())?;

        let response = self.client.execute(request).await?;

        let verification_result = response.json::<HCaptchaResponse>().await?;
        if !verification_result.success {
            return Ok(ChallengeStatus::Failed(
                verification_result.error_codes.unwrap_or_default(),
            ));
        }

        Ok(ChallengeStatus::Verified)
    }
}
