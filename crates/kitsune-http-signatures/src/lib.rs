//!
//! HTTP signatures library
//!
//! Only supports asymmetric signing schemes (aka. no HMAC and such)
//!

#![feature(iter_intersperse)]
#![forbid(rust_2018_idioms, unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use crate::{header::SignatureHeader, util::UnixTimestampExt};
use http::{header::HeaderName, request::Parts, HeaderValue};
use ring::{
    rand::SystemRandom,
    signature::{EcdsaKeyPair, Ed25519KeyPair, RsaKeyPair, UnparsedPublicKey, RSA_PKCS1_SHA256},
};
use std::{
    error::Error as StdError,
    future::Future,
    time::{Duration, SystemTime},
};
use typed_builder::TypedBuilder;

pub use crate::error::Error;
pub use ring;

mod error;
mod header;
mod util;

type BoxError = Box<dyn StdError + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

static SIGNATURE: HeaderName = HeaderName::from_static("signature");

/// Components of the signature
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SignatureComponent<'a> {
    /// Request target (path and query)
    RequestTarget,

    /// Timestamp the signature was created
    Created,

    /// Timestamp the signature expires
    Expires,

    /// Header of the request
    Header(&'a str),
}

impl<'a> SignatureComponent<'a> {
    fn from_str(raw: &'a str) -> Self {
        match raw {
            "(request-target)" => Self::RequestTarget,
            "(created)" => Self::Created,
            "(expires)" => Self::Expires,
            header => Self::Header(header),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Self::RequestTarget => "(request-target)",
            Self::Created => "(created)",
            Self::Expires => "(expires)",
            Self::Header(header) => header,
        }
    }
}

/// Trait representing a signing key
// TODO: Maybe replace with usage of RustCrypto `signature` traits via `ring-compat`
pub trait SigningKey {
    /// Sign the provided message and return the signature in its byte representation
    fn sign(&self, msg: &[u8]) -> Vec<u8>;
}

impl SigningKey for EcdsaKeyPair {
    fn sign(&self, msg: &[u8]) -> Vec<u8> {
        self.sign(&SystemRandom::new(), msg)
            .unwrap()
            .as_ref()
            .to_vec()
    }
}

impl SigningKey for Ed25519KeyPair {
    fn sign(&self, msg: &[u8]) -> Vec<u8> {
        self.sign(msg).as_ref().to_vec()
    }
}

impl SigningKey for RsaKeyPair {
    fn sign(&self, msg: &[u8]) -> Vec<u8> {
        let mut signature = vec![0; self.public().modulus_len()];
        self.sign(&RSA_PKCS1_SHA256, &SystemRandom::new(), msg, &mut signature)
            .unwrap();

        signature
    }
}

/// Cryptographic key
///
/// Depending on the context its used in, it either represents a private or a public key
#[derive(Clone, TypedBuilder)]
pub struct PrivateKey<'a, K>
where
    K: SigningKey,
{
    /// Unique identifier of the key
    key_id: &'a str,

    /// Signing key
    key: K,
}

struct SignatureString<'a> {
    pub algorithm: &'a str,
    pub components: &'a [SignatureComponent<'a>],
    pub parts: &'a Parts,
    pub created: Option<SystemTime>,
    pub expires: Option<SystemTime>,
}

impl<'a> TryFrom<SignatureString<'a>> for String {
    type Error = Error;

    fn try_from(value: SignatureString<'a>) -> Result<Self, Self::Error> {
        // Error out if the used algorithm isn't "hs2019" but it uses the "(created)"/"(expires)" pseudo-headers
        if value.algorithm != "hs2019"
            && (value.components.contains(&SignatureComponent::Created)
                || value.components.contains(&SignatureComponent::Expires))
        {
            return Err(Error::InvalidSignatureHeader);
        }

        let signature_string = value
            .components
            .iter()
            .map(|component| {
                let component = match component {
                    SignatureComponent::Created => {
                        let timestamp = value
                            .created
                            .ok_or(Error::MissingComponent)?
                            .to_unix_timestamp()?;

                        format!("(created): {timestamp}")
                    }
                    SignatureComponent::Expires => {
                        let timestamp = value
                            .expires
                            .ok_or(Error::MissingComponent)?
                            .to_unix_timestamp()?;

                        format!("(expires): {timestamp}")
                    }
                    SignatureComponent::RequestTarget => {
                        let uri = &value.parts.uri;
                        format!(
                            "(request-target): {} {}",
                            value.parts.method.as_str().to_lowercase(),
                            uri.path_and_query()
                                .map_or_else(|| uri.path(), |path| path.as_str())
                        )
                    }
                    SignatureComponent::Header(header_name) => {
                        let header_value = value
                            .parts
                            .headers
                            .get(*header_name)
                            .ok_or(Error::MissingComponent)?
                            .to_str()?;

                        format!("{}: {}", header_name.to_lowercase(), header_value)
                    }
                };
                Ok(component)
            })
            .collect::<Result<Vec<_>>>()?
            .join("\n");

        Ok(signature_string)
    }
}

/// HTTP signer
#[derive(Clone, TypedBuilder)]
pub struct HttpSigner {
    /// Include the creation timestamp into the signing header
    #[builder(default)]
    include_creation_timestamp: bool,

    /// Duration in which the signature expires
    #[builder(default, setter(strip_option))]
    expires_in: Option<Duration>,
}

impl HttpSigner {
    /// Sign an HTTP request
    pub async fn sign<K>(
        &self,
        parts: &Parts,
        components: Vec<SignatureComponent<'_>>,
        key: PrivateKey<'_, K>,
    ) -> Result<(HeaderName, HeaderValue)>
    where
        K: SigningKey + Send + 'static,
    {
        let created = self.include_creation_timestamp.then(SystemTime::now);
        let expires = self
            .expires_in
            .map(|expires_in| SystemTime::now() + expires_in);

        let signature_string = SignatureString {
            algorithm: "hs2019",
            components: &components,
            parts,
            created,
            expires,
        };
        let stringified_signature_string: String = signature_string.try_into()?;

        let signature =
            kitsune_blocking::crypto(move || key.key.sign(stringified_signature_string.as_bytes()))
                .await?;

        let signature_header = SignatureHeader {
            key_id: key.key_id,
            signature_components: components,
            signature,
            algorithm: None,
            created,
            expires,
        };
        let stringified_signature_header: String = signature_header.try_into()?;

        Ok((
            SIGNATURE.clone(),
            HeaderValue::from_str(&stringified_signature_header)?,
        ))
    }
}

impl Default for HttpSigner {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// HTTP verifier
#[derive(Clone, TypedBuilder)]
pub struct HttpVerifier {
    /// Check whether the signature is expired
    ///
    /// This just does a basic check if the `(expires)` header exists.
    /// If you want a more aggressive check, use `enforce_expiration`
    #[builder(default = true)]
    check_expiration: bool,

    /// Enforce the signature not being older than this specified duration
    ///
    /// - If the signature doesn't contain an `(created)` or `Date` header, the signature will be rejected
    /// - If the signature contains an `(expires)` header, we enforce the shorter one
    ///
    /// Defaults to 5 minutes
    #[builder(default = Some(Duration::from_secs(5 * 60)))]
    enforce_expiration: Option<Duration>,
}

impl HttpVerifier {
    /// Verify an HTTP signature
    ///
    /// `key_fn` is a function that obtains a public key (in its DER representation) based in its key ID
    pub async fn verify<'a, F, Fut, B>(&self, parts: &'a Parts, key_fn: F) -> Result<()>
    where
        F: FnOnce(&'a str) -> Fut,
        Fut: Future<Output = Result<UnparsedPublicKey<B>, BoxError>> + 'a,
        B: AsRef<[u8]> + Send + 'static,
    {
        let header = parts
            .headers
            .get(&SIGNATURE)
            .ok_or(Error::MissingSignatureHeader)?;

        let header_str = header.to_str()?;
        let signature_header = SignatureHeader::parse(header_str)?;

        if self.check_expiration && signature_header.is_expired() {
            return Err(Error::ExpiredSignature);
        }

        if let Some(enforced_duration) = self.enforce_expiration {
            if signature_header.is_expired_strict(parts, enforced_duration)? {
                return Err(Error::ExpiredSignature);
            }
        }

        let public_key = key_fn(signature_header.key_id)
            .await
            .map_err(Error::GetKey)?;

        let signature_string = SignatureString {
            algorithm: signature_header.algorithm.unwrap_or("hs2019"),
            components: &signature_header.signature_components,
            created: signature_header.created,
            expires: signature_header.expires,
            parts,
        };
        let stringified_signature_string: String = signature_string.try_into()?;

        kitsune_blocking::crypto(move || {
            public_key.verify(
                stringified_signature_string.as_bytes(),
                &signature_header.signature,
            )
        })
        .await??;

        Ok(())
    }
}

impl Default for HttpVerifier {
    fn default() -> Self {
        Self::builder().build()
    }
}
