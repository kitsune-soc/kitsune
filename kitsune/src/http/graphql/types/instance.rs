use async_graphql::Enum;
use async_graphql::SimpleObject;

#[derive(Debug, SimpleObject)]
pub struct CaptchaInfo {
    backend: CaptchaBackend,
    key: String,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum CaptchaBackend {
    HCaptcha,
    MCaptcha,
}

impl From<kitsune_captcha::AnyCaptcha> for CaptchaInfo {
    fn from(e: kitsune_captcha::AnyCaptcha) -> Self {
        match e {
            kitsune_captcha::AnyCaptcha::HCaptcha(config) => Self {
                backend: CaptchaBackend::HCaptcha,
                key: config.site_key,
            },
            kitsune_captcha::AnyCaptcha::MCaptcha(config) => Self {
                backend: CaptchaBackend::MCaptcha,
                key: config.widget_link,
            },
        }
    }
}

#[derive(SimpleObject)]
pub struct Instance {
    pub captcha: Option<CaptchaInfo>,
    pub character_limit: usize,
    pub description: String,
    pub domain: String,
    pub local_post_count: u64,
    pub name: String,
    pub registrations_open: bool,
    pub user_count: u64,
    pub version: &'static str,
}
