use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct HCaptchaConfiguration {
    pub verify_url: SmolStr,
    pub site_key: SmolStr,
    pub secret_key: SmolStr,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct MCaptchaConfiguration {
    pub widget_link: SmolStr,
    pub site_key: SmolStr,
    pub secret_key: SmolStr,
    pub verify_url: SmolStr,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum Configuration {
    HCaptcha(HCaptchaConfiguration),
    MCaptcha(MCaptchaConfiguration),
}
