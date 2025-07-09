//!
//! Easy and fool-proof HTTP signature handling
//!
//! Integrates with async and offers an incredibly simplistic interface for signing and verifying HTTP signatures
//!

use crate::{
    BoxError, SIGNATURE_HEADER,
    cavage::{SafetyCheckError, SignatureHeader},
};
use http::{HeaderValue, Method, header::DATE};
use miette::Diagnostic;
use scoped_futures::ScopedFutureWrapper;
use thiserror::Error;
use tracing::{debug, instrument};

const GET_HEADERS: &[&str] = &["host", "date"];
const POST_HEADERS: &[&str] = &["host", "date", "content-type", "digest"];

/// Easy module error
#[derive(Debug, Diagnostic, Error)]
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

    /// Safety check failure
    #[error(transparent)]
    SafetyCheck(#[from] SafetyCheckError),

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
///
/// The key parameter has to be an PEM-encoded private key in the PKCS#8 format
///
/// This will fail if the key algorithm is unsupported. For a list of supported algorithms, check [`crate::crypto::parse::private_key`]
#[inline]
#[cfg_attr(not(coverage), instrument(skip_all, fields(key_id)))]
pub async fn sign<B>(
    mut req: http::Request<B>,
    key_id: &str,
    key: &str,
) -> Result<http::Request<B>, Error> {
    // First, set/overwrite the `Date` header
    let date_header_value =
        HeaderValue::from_str(&httpdate::fmt_http_date(tick_tock_mock::now())).unwrap();
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

    let key = crate::crypto::parse::private_key(key)?;
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
///
/// The closure is expected to return a future which resolves into a result which contains a PEM-encoded PKCS#8 verifying key.
/// You don't need to supply any more information. The library will figure out the rest.
#[inline]
#[cfg_attr(not(coverage), instrument(skip_all))]
pub async fn verify<'a, B, F, Fut, E>(req: &'a http::Request<B>, get_key: F) -> Result<(), Error>
where
    for<'k_id> F: Fn(&'k_id str) -> ScopedFutureWrapper<'k_id, 'a, Fut>,
    Fut: Future<Output = Result<String, E>>,
    E: Into<BoxError>,
{
    let Some(header) = req.headers().get(&SIGNATURE_HEADER) else {
        debug!("Missing 'Signature' header");
        return Err(Error::MissingSignature);
    };

    let signature_header = super::parse(header.to_str()?)?;
    super::is_safe(req, &signature_header)?;

    let signature_string = super::signature_string::construct(req, &signature_header)?;
    let pem_key = get_key(signature_header.key_id)
        .await
        .map_err(|err| Error::GetKey(err.into()))?;

    let encoded_signature = signature_header.signature.to_string();
    let public_key = crate::crypto::parse::public_key(&pem_key)?;

    blowocking::crypto(move || {
        crate::crypto::verify(signature_string.as_bytes(), &encoded_signature, &public_key)
    })
    .await??;

    Ok(())
}
