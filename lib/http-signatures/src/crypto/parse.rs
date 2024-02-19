//!
//! Parse cryptographic keys for use in the HTTP signature implementations
//!

use const_oid::db::{rfc5912::RSA_ENCRYPTION, rfc8410::ID_ED_25519};
use miette::Diagnostic;
use pkcs8::{Document, SubjectPublicKeyInfoRef};
use ring::signature::{
    UnparsedPublicKey, VerificationAlgorithm, ED25519, RSA_PKCS1_2048_8192_SHA256,
};
use thiserror::Error;

/// Key parsing error
#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    /// Malformed DER structure
    #[error(transparent)]
    Der(#[from] pkcs8::der::Error),

    /// Malformed key
    #[error("Malformed key")]
    MalformedKey,

    /// Malformed PKCS#8 document
    #[error(transparent)]
    Pkcs8(#[from] pkcs8::Error),

    /// Unknown key type
    #[error("Unknown key type")]
    UnknownKeyType,
}

/// Parse a public key from its PKCS#8 PEM form
///
/// Currently supported algorithms:
///
/// - RSA
/// - Ed25519
#[inline]
pub fn public_key(pem: &str) -> Result<UnparsedPublicKey<Vec<u8>>, Error> {
    let (_pem_tag, document) = Document::from_pem(pem)?;
    let spki: SubjectPublicKeyInfoRef<'_> = document.decode_msg()?;

    let verify_algo: &dyn VerificationAlgorithm = if spki.algorithm.oid == RSA_ENCRYPTION {
        &RSA_PKCS1_2048_8192_SHA256
    } else if spki.algorithm.oid == ID_ED_25519 {
        &ED25519
    } else {
        return Err(Error::UnknownKeyType);
    };

    let raw_bytes = spki
        .subject_public_key
        .as_bytes()
        .ok_or(Error::MalformedKey)?
        .to_vec();

    Ok(UnparsedPublicKey::new(verify_algo, raw_bytes))
}
