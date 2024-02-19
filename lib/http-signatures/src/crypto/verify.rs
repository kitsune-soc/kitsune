use miette::Diagnostic;
use ring::signature::UnparsedPublicKey;
use thiserror::Error;

/// Verification error
#[derive(Debug, Diagnostic, Error)]
pub enum VerifyError {
    /// Failed to decode the Base64 payload
    #[error(transparent)]
    Base64(#[from] base64_simd::Error),
}

/// Verify that the message corresponds with the signature using the provided verifying key
#[inline]
pub fn verify<B>(
    msg: &[u8],
    encoded_signature: &str,
    key: &UnparsedPublicKey<B>,
) -> Result<bool, VerifyError>
where
    B: AsRef<[u8]>,
{
    let signature = base64_simd::STANDARD.decode_to_vec(encoded_signature)?;
    Ok(key.verify(msg, &signature).is_ok())
}
