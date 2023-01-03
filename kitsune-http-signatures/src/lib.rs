// TODO: Add support for other signature schemes

use http::{
    header::{HeaderName, InvalidHeaderValue, ToStrError},
    HeaderMap, HeaderValue, Method, Uri,
};
use ring::{
    rand::SystemRandom,
    signature::{RsaKeyPair, UnparsedPublicKey, RSA_PKCS1_2048_8192_SHA256, RSA_PKCS1_SHA256},
};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),

    #[error(transparent)]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[error(transparent)]
    KeyRejected(#[from] ring::error::KeyRejected),

    #[error("Malformed signature header")]
    MalformedSignatureHeader,

    #[error("Missing header")]
    MissingHeader,

    #[error("Missing signature header")]
    MissingSignatureHeader,

    #[error(transparent)]
    RingUnspecified(#[from] ring::error::Unspecified),

    #[error(transparent)]
    ToStr(#[from] ToStrError),
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Request<'a> {
    pub headers: &'a HeaderMap,
    pub uri: &'a Uri,
    pub method: &'a Method,
}

fn construct_signing_string(
    req: Request<'_>,
    parsed_signature_header: HashMap<&str, &str>,
) -> Result<String> {
    let mut signing_string = String::new();
    for header in parsed_signature_header
        .get("headers")
        .ok_or(Error::MalformedSignatureHeader)?
        .split_whitespace()
    {
        match header {
            header @ "(request-target)" => {
                signing_string.push_str(header);
                signing_string.push_str(": ");
                signing_string.push_str(&req.method.as_str().to_lowercase());
                signing_string.push(' ');
                signing_string.push_str(&req.uri.to_string());
            }
            header @ "(created)" => {
                let created = parsed_signature_header
                    .get("created")
                    .ok_or(Error::MalformedSignatureHeader)?;
                signing_string.push_str(header);
                signing_string.push_str(": ");
                signing_string.push_str(created);
            }
            header @ "(expires)" => {
                let expires = parsed_signature_header
                    .get("expires")
                    .ok_or(Error::MalformedSignatureHeader)?;
                signing_string.push_str(header);
                signing_string.push_str(": ");
                signing_string.push_str(expires);
            }
            header => {
                let header_value = req
                    .headers
                    .get(header)
                    .ok_or(Error::MissingHeader)?
                    .to_str()?
                    .trim();
                signing_string.push_str(header);
                signing_string.push_str(": ");
                signing_string.push_str(header_value);
            }
        }

        signing_string.push('\n');
    }

    Ok(signing_string)
}

pub fn sign(
    req: Request<'_>,
    headers: &[&str],
    key_id: &str,
    private_key: &[u8],
) -> Result<HeaderValue> {
    // TODO: Add expires and created support

    let signing_string = construct_signing_string(req, HashMap::new())?;
    let private_key = RsaKeyPair::from_der(private_key)?;

    let mut signature = vec![0; private_key.public_modulus_len()];
    private_key.sign(
        &RSA_PKCS1_SHA256,
        &SystemRandom::new(),
        signing_string.as_bytes(),
        &mut signature,
    )?;
    let signature = base64::encode(signature);

    let headers = headers.join(" ");
    let signature_header =
        format!("keyId=\"{key_id}\",signature=\"{signature}\",headers=\"{headers}\"");

    Ok(HeaderValue::from_str(&signature_header)?)
}

pub fn verify(req: Request<'_>, public_key: &[u8]) -> Result<bool> {
    let Some(signature_header) = req.headers.get(HeaderName::from_static("Signature")) else {
        return Err(Error::MissingSignatureHeader);
    };
    let signature_header = signature_header.to_str()?;

    let parsed_signature_header: HashMap<&str, &str> = signature_header
        .split("\",")
        .filter_map(|kv_pair| {
            let (key, value) = kv_pair.split_once('=')?;
            Some((key, value.trim_start_matches('"')))
        })
        .collect();

    let signature = base64::decode(
        parsed_signature_header
            .get("signature")
            .ok_or(Error::MalformedSignatureHeader)?,
    )?;

    let signing_string = construct_signing_string(req, parsed_signature_header)?;
    let public_key = UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, public_key);
    Ok(public_key
        .verify(signing_string.as_bytes(), &signature)
        .is_ok())
}
