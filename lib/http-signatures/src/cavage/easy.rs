//!
//! Easy and fool-proof HTTP signature handling
//!
//! Integrates with async and offers an incredibly simplistic interface for signing and verifying HTTP signatures
//!

use crate::{cavage::SignatureHeader, crypto::SigningKey, BoxError, SIGNATURE_HEADER};
use http::{header::DATE, HeaderValue, Method};
use std::{future::Future, time::SystemTime};
use thiserror::Error;
use tracing::{debug, instrument};

const GET_HEADERS: &[&str] = &["host", "date"];
const POST_HEADERS: &[&str] = &["host", "date", "content-type", "digest"];

/// Easy module error
#[derive(Debug, Error)]
pub enum Error {
    /// Blocking pool communication failure
    #[error(transparent)]
    Blocking(#[from] blowocking::Error),

    /// Couldn't get key from user-provided closure
    #[error(transparent)]
    GetKey(BoxError),

    /// Invalid HTTP header value (non UTF-8 value)
    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::ToStrError),

    /// Public key failed to parse
    #[error(transparent)]
    InvalidKey(#[from] crate::crypto::parse::Error),

    /// Signature header parsing failed
    #[error(transparent)]
    InvalidSignatureHeader(#[from] super::ParseError),

    /// Signature header is missing
    #[error("Missing signature")]
    MissingSignature,

    /// Signature string construction failure
    #[error(transparent)]
    SignatureStringConstruction(#[from] super::signature_string::Error),

    /// HTTP method is unsupported
    #[error("Unsupported HTTP method")]
    UnsupportedHttpMethod,

    /// Verification failed
    #[error(transparent)]
    Verify(#[from] crate::crypto::VerifyError),
}

/// Sign an HTTP request using the provided signing key using opinionated defaults
#[inline]
#[instrument(skip_all)]
pub async fn sign<B, SK>(
    mut req: http::Request<B>,
    key_id: &str,
    key: SK,
) -> Result<http::Request<B>, Error>
where
    SK: SigningKey + Send + 'static,
{
    // First, set/overwrite the `Date` header
    let date_header_value =
        HeaderValue::from_str(&httpdate::fmt_http_date(SystemTime::now())).unwrap();
    req.headers_mut().insert(DATE, date_header_value);

    let headers = match *req.method() {
        Method::GET => GET_HEADERS.iter().copied(),
        Method::POST => POST_HEADERS.iter().copied(),
        _ => return Err(Error::UnsupportedHttpMethod),
    };

    let signature_header = SignatureHeader {
        key_id,
        headers,
        signature: (),
        created: None,
        expires: None,
    };

    debug_assert!(super::is_safe(&req, &signature_header).is_ok());

    let signature_string = super::signature_string::construct(&req, &signature_header)?;
    let signature =
        blowocking::crypto(move || crate::crypto::sign(signature_string.as_bytes(), &key)).await?;

    let signature_header = SignatureHeader {
        key_id: signature_header.key_id,
        headers: signature_header.headers,
        signature,
        created: signature_header.created,
        expires: signature_header.expires,
    };

    let signature_header_value =
        HeaderValue::from_str(&super::serialise(signature_header)).unwrap();

    req.headers_mut()
        .insert(&SIGNATURE_HEADER, signature_header_value);

    Ok(req)
}

/// Verify an HTTP request using opinionated defaults
#[inline]
#[instrument(skip_all)]
pub async fn verify<B, F, Fut, E>(req: &http::Request<B>, get_key: F) -> Result<bool, Error>
where
    F: Fn(&str) -> Fut,
    Fut: Future<Output = Result<String, E>>,
    E: Into<BoxError>,
{
    let Some(header) = req.headers().get(&SIGNATURE_HEADER) else {
        debug!("Missing 'Signature' header");
        return Err(Error::MissingSignature);
    };

    let signature_header = super::parse(header.to_str()?)?;
    if super::is_safe(req, &signature_header).is_err() {
        return Ok(false);
    }

    let signature_string = super::signature_string::construct(req, &signature_header)?;
    let pem_key = get_key(signature_header.key_id)
        .await
        .map_err(|err| Error::GetKey(err.into()))?;

    let encoded_signature = signature_header.signature.to_string();
    let public_key = crate::crypto::parse::public_key(&pem_key)?;

    let is_valid = blowocking::crypto(move || {
        crate::crypto::verify(signature_string.as_bytes(), &encoded_signature, &public_key)
    })
    .await??;

    Ok(is_valid)
}
