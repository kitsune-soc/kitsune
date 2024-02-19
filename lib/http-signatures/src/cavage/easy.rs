use crate::{BoxError, SIGNATURE_HEADER};
use std::future::Future;
use thiserror::Error;
use tracing::{debug, instrument};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Blocking(#[from] blowocking::Error),

    #[error(transparent)]
    GetKey(BoxError),

    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::ToStrError),

    #[error(transparent)]
    InvalidKey(#[from] crate::crypto::parse::Error),

    #[error(transparent)]
    InvalidSignatureHeader(#[from] super::ParseError),

    #[error("Missing signature")]
    MissingSignature,

    #[error(transparent)]
    SignatureStringConstruction(#[from] super::signature_string::Error),

    #[error(transparent)]
    Verify(#[from] crate::crypto::VerifyError),
}

#[inline]
#[instrument(skip_all)]
pub async fn sign<B>(req: http::Request<B>) -> http::Request<B> {
    todo!();
}

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
