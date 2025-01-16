use crate::{error::Error, flow};
use serde::{Deserialize, Serialize};
use sha2::{
    digest::{typenum, OutputSizeUser},
    Digest, Sha256,
};
use std::borrow::Cow;
use strum::{AsRefStr, EnumString};
use subtle::ConstantTimeEq;

#[derive(AsRefStr, Clone, Debug, Default, Deserialize, EnumString, PartialEq, Serialize)]
#[strum(serialize_all = "snake_case")]
pub enum Method {
    #[default]
    None,
    #[strum(serialize = "S256")]
    S256,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Payload<'a> {
    pub challenge: Cow<'a, str>,
    pub method: Method,
}

impl Payload<'_> {
    #[must_use]
    pub fn into_owned(self) -> Payload<'static> {
        Payload {
            challenge: self.challenge.into_owned().into(),
            method: self.method,
        }
    }

    #[inline]
    #[cfg_attr(not(coverage), instrument(skip(self)))]
    fn verify_s256(&self, code_verifier: &str) -> Result<(), flow::Error> {
        // at least it's zero allocations..
        const B64_ENGINE: base64_simd::Base64 = base64_simd::URL_SAFE_NO_PAD;
        const SHA256_HASH_LEN: usize =
            <<Sha256 as OutputSizeUser>::OutputSize as typenum::Unsigned>::USIZE;

        let decoded_len = B64_ENGINE
            .decoded_length(self.challenge.as_bytes())
            .inspect_err(|error| debug!(?error, "couldnt determine decoded length"))
            .map_err(Error::body)?;

        if decoded_len > SHA256_HASH_LEN {
            return Err(flow::Error::InvalidRequest);
        }

        let mut decoded_buf = [0; SHA256_HASH_LEN];
        let decoded = B64_ENGINE
            .decode(
                self.challenge.as_bytes(),
                base64_simd::Out::from_slice(&mut decoded_buf),
            )
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
    #[cfg_attr(not(coverage), instrument(skip(self)))]
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
