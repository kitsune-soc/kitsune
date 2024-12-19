use crate::{error::Error, flow};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use strum::{AsRefStr, EnumString};
use subtle::ConstantTimeEq;

#[derive(AsRefStr, Default, Deserialize, EnumString, Serialize)]
#[strum(serialize_all = "snake_case")]
pub enum Method {
    #[default]
    None,
    #[strum(serialize = "S256")]
    S256,
}

#[derive(Deserialize, Serialize)]
pub struct Payload<'a> {
    pub challenge: Cow<'a, str>,
    pub method: Method,
}

impl Payload<'_> {
    #[inline]
    fn verify_s256(&self, code_verifier: &str) -> Result<(), flow::Error> {
        let decoded = base64_simd::URL_SAFE_NO_PAD
            .decode_to_vec(self.challenge.as_bytes())
            .inspect_err(|error| debug!(?error, "failed to decode pkce payload"))
            .map_err(Error::body)?;

        let hash = Sha256::digest(code_verifier);
        if decoded.ct_eq(hash.as_slice()).into() {
            Ok(())
        } else {
            Err(flow::Error::InvalidGrant)
        }
    }

    #[inline]
    fn verify_none(&self, code_verifier: &str) -> Result<(), flow::Error> {
        let challenge_bytes = self.challenge.as_bytes();
        if challenge_bytes.ct_eq(code_verifier.as_bytes()).into() {
            Ok(())
        } else {
            Err(flow::Error::InvalidGrant)
        }
    }

    #[inline]
    pub fn verify(&self, code_verifier: &str) -> Result<(), flow::Error> {
        match self.method {
            Method::None => self.verify_none(code_verifier),
            Method::S256 => self.verify_s256(code_verifier),
        }
    }
}
