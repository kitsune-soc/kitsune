//!
//! HTTP signatures library
//!
//! Only supports asymmetric signing schemes (aka. no HMAC and such)
//!

#![forbid(rust_2018_idioms, unsafe_code)]
#![deny(missing_docs)]

use crate::header::SignatureHeader;
use derive_builder::Builder;
use http::{
    header::{HeaderName, InvalidHeaderName},
    request::Parts,
    HeaderValue,
};
use ring::{
    rand::SystemRandom,
    signature::{EcdsaKeyPair, Ed25519KeyPair, RsaKeyPair, UnparsedPublicKey, RSA_PKCS1_SHA256},
};
use std::{
    error::Error as StdError,
    future::Future,
    time::{Duration, SystemTime},
};

pub use crate::error::Error;
pub use ring;

mod error;
mod header;
mod util;

type BoxError = Box<dyn StdError + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

static SIGNATURE: HeaderName = HeaderName::from_static("signature");

/// Components of the signature
#[derive(Clone)]
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
    fn parse(raw: &'a str) -> Result<Self, InvalidHeaderName> {
        let component = match raw {
            "(request-target)" => Self::RequestTarget,
            "(created)" => Self::Created,
            "(expires)" => Self::Expires,
            header => Self::Header(header),
        };
        Ok(component)
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
        let mut signature = vec![0; self.public_modulus_len()];
        self.sign(&RSA_PKCS1_SHA256, &SystemRandom::new(), msg, &mut signature)
            .unwrap();
        signature
    }
}

/// Cryptographic key
///
/// Depending on the context its used in, it either represents a private or a public key
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct PrivateKey<'a, K>
where
    K: SigningKey,
{
    /// Unique identifier of the key
    key_id: &'a str,

    /// Signing key
    key: K,
}

impl<'a, K> PrivateKey<'a, K>
where
    K: SigningKey,
{
    /// Return a builder of the private key
    pub fn builder() -> PrivateKeyBuilder<'a, K> {
        PrivateKeyBuilder::default()
    }
}

#[allow(dead_code)] // shush.
struct SignatureString<'a> {
    pub components: &'a [SignatureComponent<'a>],
    pub parts: &'a Parts,
    pub created: Option<SystemTime>,
    pub expires: Option<SystemTime>,
}

impl<'a> TryFrom<SignatureString<'a>> for String {
    type Error = Error;

    fn try_from(value: SignatureString<'a>) -> Result<Self, Self::Error> {
        let signature_string = value
            .components
            .iter()
            // Ugly. The signature string isn't supposed to contain created and expires components
            .filter(|component| {
                !matches!(
                    component,
                    SignatureComponent::Created | SignatureComponent::Expires,
                )
            })
            .map(|component| {
                let component = match component {
                    SignatureComponent::RequestTarget => format!(
                        "(request-target): {} {}",
                        value.parts.method.as_str().to_lowercase(),
                        value.parts.uri
                    ),
                    SignatureComponent::Header(header_name) => {
                        let header_value = value
                            .parts
                            .headers
                            .get(*header_name)
                            .ok_or(Error::MissingComponent)?
                            .to_str()?;

                        format!("{}: {}", header_name.to_lowercase(), header_value)
                    }
                    _ => unreachable!(),
                };
                Ok(component)
            })
            .collect::<Result<Vec<_>>>()?
            .join("\n");

        Ok(signature_string)
    }
}

/// HTTP signer/verifier
///
/// The name is a bit unfortunate. It not only signs, it also verifies
#[derive(Builder, Clone)]
pub struct HttpSigner<'a> {
    /// HTTP request parts
    parts: &'a Parts,

    /// Check whether the signature is expired. Only important if you wanna verify something
    #[builder(default = "true")]
    check_expiration: bool,

    /// Duration in which the signature expires. Only important if you wanna sign something
    #[builder(default, setter(strip_option))]
    expires_in: Option<Duration>,
}

impl<'a> HttpSigner<'a> {
    /// Return a builder for the HTTP signer
    pub fn builder() -> HttpSignerBuilder<'a> {
        HttpSignerBuilder::default()
    }
}

impl HttpSigner<'_> {
    /// Sign an HTTP request
    pub async fn sign<K>(
        &self,
        key: PrivateKey<'_, K>,
        components: Vec<SignatureComponent<'_>>,
    ) -> Result<(HeaderName, HeaderValue)>
    where
        K: SigningKey + Send + 'static,
    {
        let created = Some(SystemTime::now());
        let expires = self
            .expires_in
            .map(|expires_in| SystemTime::now() + expires_in);

        let signature_string = SignatureString {
            components: &components,
            parts: self.parts,
            created,
            expires,
        };
        let stringified_signature_string: String = signature_string.try_into()?;
        let signature = tokio::task::spawn_blocking(move || {
            key.key.sign(stringified_signature_string.as_bytes())
        })
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

    /// Verify an HTTP signature
    ///
    /// `key_fn` is a function that obtains a public key (in its DER representation) based in its key ID
    pub async fn verify<F, Fut, B>(&self, key_fn: F) -> Result<()>
    where
        F: FnOnce(&'_ str) -> Fut,
        Fut: Future<Output = Result<UnparsedPublicKey<B>, BoxError>>,
        B: AsRef<[u8]> + Send + 'static,
    {
        let header = self
            .parts
            .headers
            .get(&SIGNATURE)
            .ok_or(Error::MissingSignatureHeader)?;

        let header_str = header.to_str()?;
        let signature_header = SignatureHeader::parse(header_str)?;

        if let Some(ref expires) = signature_header.expires {
            if self.check_expiration && *expires < SystemTime::now() {
                return Err(Error::ExpiredSignature);
            }
        }

        let public_key = key_fn(signature_header.key_id)
            .await
            .map_err(Error::GetKey)?;

        let signature_string = SignatureString {
            components: &signature_header.signature_components,
            created: signature_header.created,
            expires: signature_header.expires,
            parts: self.parts,
        };
        let stringified_signature_string: String = signature_string.try_into()?;

        tokio::task::spawn_blocking(move || {
            public_key.verify(
                stringified_signature_string.as_bytes(),
                &signature_header.signature,
            )
        })
        .await??;

        Ok(())
    }
}
