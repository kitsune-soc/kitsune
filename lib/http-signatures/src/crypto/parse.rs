//!
//! Parse cryptographic keys for use in the HTTP signature implementations
//!

use super::SigningKey as SigningKeyTrait;
use const_oid::db::{rfc5912::RSA_ENCRYPTION, rfc8410::ID_ED_25519};
use miette::Diagnostic;
use pkcs8::{Document, PrivateKeyInfo, SecretDocument, SubjectPublicKeyInfoRef};
use ring::signature::{
    ED25519, Ed25519KeyPair, RSA_PKCS1_2048_8192_SHA256, RsaKeyPair, UnparsedPublicKey,
    VerificationAlgorithm,
};
use thiserror::Error;

/// Key parsing error
#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    /// Malformed DER structure
    #[error(transparent)]
    Der(#[from] pkcs8::der::Error),

    /// Key rejected
    #[error(transparent)]
    KeyRejected(#[from] ring::error::KeyRejected),

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

/// Enum dispatch over various signing keys
#[non_exhaustive]
pub enum SigningKey {
    /// Ed25519
    Ed25519(Ed25519KeyPair),

    /// RSA
    Rsa(RsaKeyPair),
}

impl SigningKeyTrait for SigningKey {
    type Output = Vec<u8>;

    fn sign(&self, msg: &[u8]) -> Self::Output {
        match self {
            Self::Ed25519(key) => key.sign(msg).as_ref().to_vec(),
            Self::Rsa(key) => SigningKeyTrait::sign(key, msg),
        }
    }
}

/// Parse a private key from its PKCS#8 PEM form.
/// This function uses constant-time PEM decoding and zeroizes any temporary allocations, following good cryptographic hygiene practices.
///
/// When working with this library, prefer using this function over your own decoding logic.
///
/// Currently supported algorithms:
///
/// - RSA
/// - Ed25519
#[inline]
pub fn private_key(pem: &str) -> Result<SigningKey, Error> {
    let (_tag_line, document) = SecretDocument::from_pem(pem)?;
    let private_key_raw: PrivateKeyInfo<'_> = document.decode_msg()?;

    let signing_key = if private_key_raw.algorithm.oid == RSA_ENCRYPTION {
        SigningKey::Rsa(RsaKeyPair::from_der(private_key_raw.private_key)?)
    } else if private_key_raw.algorithm.oid == ID_ED_25519 {
        SigningKey::Ed25519(Ed25519KeyPair::from_seed_and_public_key(
            private_key_raw.private_key,
            private_key_raw.public_key.ok_or(Error::MalformedKey)?,
        )?)
    } else {
        return Err(Error::UnknownKeyType);
    };

    Ok(signing_key)
}
