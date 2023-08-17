use async_trait::async_trait;
use http::Request;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::Result;
use crate::{CaptchaBackend, ChallengeStatus};

use kitsune_http_client::Client;

#[derive(Clone, TypedBuilder)]
/// File system storage
pub struct Captcha {
    pub widget_link: String,
    pub verify_url: String,
    pub secret_key: String,
    pub site_key: String,
    #[builder(default = Client::builder().build())]
    client: Client,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct MCaptchaResponse {
    valid: bool,
}

#[derive(Serialize, Deserialize, TypedBuilder)]
struct Body {
    token: String,
    key: String,
    secret: String,
}

#[async_trait]
impl CaptchaBackend for Captcha {
    async fn verify(&self, token: &str) -> Result<ChallengeStatus> {
        let body = Body::builder()
            .secret(self.secret_key.to_string())
            .token(token.to_string())
            .key(self.site_key.to_string())
            .build();
        let body = simd_json::to_string(&body)?;
        let request = Request::post(self.verify_url.to_owned())
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(body.into())?;
        let response = self.client.execute(request).await?;
        println!("{:#?}", response);
        let mut response_bytes = response.text().await?.into_bytes();
        println!("{:#?}", response_bytes);
        let verification_result =
            simd_json::serde::from_slice::<MCaptchaResponse>(&mut response_bytes)?;
        if !verification_result.valid {
            return Ok(ChallengeStatus::Failed(Vec::new()));
        }
        Ok(ChallengeStatus::Verified)
    }
}
