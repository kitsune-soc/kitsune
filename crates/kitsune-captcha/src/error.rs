use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, PartialEq, Display, Serialize, Deserialize, EnumString)]
pub enum CaptchaVerification {
    #[strum(serialize = "missing-input-secret")]
    MissingInputSecret,

    #[strum(serialize = "invalid-input-secret")]
    InvalidInputSecret,

    #[strum(serialize = "missing-input-response")]
    MissingInputToken,

    #[strum(serialize = "invalid-input-response")]
    InvalidInputToken,

    #[strum(serialize = "bad-request")]
    BadRequest,

    #[strum(serialize = "invalid-or-already-seen-response")]
    AlreadyCheckedToken,

    #[strum(serialize = "sitekey-secret-mismatch")]
    InvalidConfiguration,

    #[strum(default)]
    Other(String),
}
