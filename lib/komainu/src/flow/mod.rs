use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use strum::{AsRefStr, EnumString};
use subtle::ConstantTimeEq;

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

#[derive(AsRefStr, Deserialize, EnumString, Serialize)]
#[strum(serialize_all = "snake_case")]
pub enum PkceMethod {
    None,
    #[strum(serialize = "S256")]
    S256,
}

#[derive(Deserialize, Serialize)]
pub struct PkcePayload<'a> {
    pub challenge: Cow<'a, str>,
    pub method: PkceMethod,
}

impl PkcePayload<'_> {
    #[inline]
    fn verify_s256(&self, code_verifier: &str) -> Result<()> {
        let decoded = base64_simd::URL_SAFE
            .decode_to_vec(code_verifier)
            .inspect_err(|error| debug!(?error, "failed to decode pkce payload"))
            .map_err(Error::body)?;

        let hash = Sha256::digest(code_verifier);

        if decoded.ct_eq(hash.as_slice()).into() {
            Ok(())
        } else {
            Err(Error::Unauthorized)
        }
    }

    #[inline]
    fn verify_none(&self, code_verifier: &str) -> Result<()> {
        let challenge_bytes = self.challenge.as_bytes();

        if challenge_bytes.ct_eq(code_verifier.as_bytes()).into() {
            Ok(())
        } else {
            Err(Error::Unauthorized)
        }
    }

    #[inline]
    pub fn verify(&self, code_verifier: &str) -> Result<()> {
        match self.method {
            PkceMethod::None => self.verify_none(code_verifier),
            PkceMethod::S256 => self.verify_s256(code_verifier),
        }
    }
}
