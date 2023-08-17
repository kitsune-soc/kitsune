use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use thiserror::Error;

#[derive(Debug, PartialEq, Display, Serialize, Deserialize, EnumString, Error)]
pub enum CaptchaVerification {
    #[strum(serialize = "missing-input-secret")]
    MissingInputSecret,
    #[strum(serialize = "invalid-input-secret")]
    InvalidInputSecret,
    #[strum(serialize = "missing-input-response")]
    MissingInputToken,
    #[strum(serialize = "invalid-input-response")]
    InvalidInputToken,
    BadRequest,
    AlreadyCheckedToken,
    InvalidConfiguration,
    #[strum(default)]
    Other(String),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    CaptchaVerification(#[from] CaptchaVerification),

    #[error(transparent)]
    SimdJson(#[from] simd_json::Error),

    #[error(transparent)]
    Http(#[from] http::Error),

    #[error(transparent)]
    HttpClient(#[from] kitsune_http_client::Error),

    #[error(transparent)]
    HttpForm(#[from] serde_qs::Error),
}
