use serde::Serialize;
use std::borrow::Cow;

pub mod authorization;
pub mod refresh;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TokenType {
    Bearer,
}

#[derive(Serialize)]
pub struct TokenResponse<'a> {
    pub access_token: Cow<'a, str>,
    pub token_type: TokenType,
    pub refresh_token: Cow<'a, str>,
    pub expires_in: u64,
}
