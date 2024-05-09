#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

#[macro_use]
extern crate tracing;

use crate::util::OpaqueDebug;
use async_trait::async_trait;
use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};
use rand::{
    distributions::{Alphanumeric, DistString},
    RngCore,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{convert::Infallible, future::Future, ops::Deref};
use thiserror::Error;
use triomphe::Arc;
use typed_builder::TypedBuilder;
use unsize::{CoerceUnsize, Coercion};

mod util;

type BoxError = Box<dyn std::error::Error + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

const TOKEN_LENGTH: usize = 40;

/// Combined error type
#[derive(Debug, Error)]
pub enum Error {
    /// The resolver returned an error
    #[error(transparent)]
    Resolve(BoxError),

    /// The verification strategy errored out
    #[error(transparent)]
    VerificationStrategy(BoxError),

    /// The domain did not have the required TXT record
    #[error("The domain did not have a TXT record matching the requirements")]
    Unverified,
}

/// Helper trait for coercing the [`DnsResolver`] trait
///
/// Since we don't use the `Arc` provided in the standard library, automatic coercion to an unsized pointer is not possible.
/// To make it easier for you to obtain a `Arc<dyn DnsResolver>` from some type `Arc<T>`, we provide this trait.
pub trait Coerce {
    /// Coerce a concrete type into its unsized counterpart
    fn coerce(self) -> Arc<dyn DnsResolver>;
}

impl<T> Coerce for Arc<T>
where
    T: DnsResolver + 'static,
{
    #[inline]
    fn coerce(self) -> Arc<dyn DnsResolver> {
        self.unsize(Coercion!(to dyn DnsResolver))
    }
}

/// DNS resolver
#[async_trait]
pub trait DnsResolver: Send + Sync {
    /// Resolve an FQDN and return its TXT records
    async fn lookup_txt(&self, fqdn: &str) -> Result<Vec<String>, BoxError>;
}

#[async_trait]
impl DnsResolver for TokioAsyncResolver {
    async fn lookup_txt(&self, fqdn: &str) -> Result<Vec<String>, BoxError> {
        let records =
            self.txt_lookup(fqdn)
                .await?
                .iter()
                .flat_map(|record| {
                    record.txt_data().iter().filter_map(|data| {
                        simdutf8::basic::from_utf8(data).ok().map(ToOwned::to_owned)
                    })
                })
                .collect();

        Ok(records)
    }
}

/// Domain verification strategy
pub trait VerificationStrategy: DeserializeOwned + Serialize {
    /// Error returned by this verification strategy
    type Error: Into<BoxError>;

    /// Verify whether the domain is valid by looking at its TXT records
    fn verify<'a>(
        &self,
        txt_records: impl Iterator<Item = &'a str> + Send,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;
}

/// Dummy strategy that always resolves to `true`
///
/// Only useful for testing
#[derive(Default, Deserialize, Serialize)]
pub struct DummyStrategy {
    _priv: (),
}

impl VerificationStrategy for DummyStrategy {
    type Error = Infallible;

    async fn verify<'a>(
        &self,
        _txt_records: impl Iterator<Item = &'a str> + Send,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

/// The de-facto default strategy
///
/// It checks whether the TXT records contain a value looking like `[key]=[value]`
#[derive(Clone, Deserialize, Serialize)]
pub struct KeyValueStrategy {
    /// Key of the entry
    pub key: String,

    /// Value of the entry
    pub value: String,
}

impl KeyValueStrategy {
    /// Create a [`KeyValueStrategy`] with a randomly generated value
    pub fn generate<R>(rng: &mut R, key: String) -> Self
    where
        R: RngCore,
    {
        Self {
            key,
            value: Alphanumeric.sample_string(rng, TOKEN_LENGTH),
        }
    }
}

impl VerificationStrategy for KeyValueStrategy {
    type Error = Infallible;

    async fn verify(
        &self,
        txt_records: impl Iterator<Item = &str> + Send,
    ) -> Result<bool, Self::Error> {
        Ok(txt_records
            .filter_map(|record| record.split_once('='))
            .any(|(key, value)| key == self.key && value == self.value))
    }
}

/// Construct the default resolver used by this library
#[must_use]
pub fn default_resolver() -> Arc<dyn DnsResolver> {
    Arc::new(TokioAsyncResolver::tokio(
        ResolverConfig::default(),
        ResolverOpts::default(),
    ))
    .unsize(Coercion!(to dyn DnsResolver))
}

/// Verifier for an arbitrary FQDN
#[derive(Clone, Debug, TypedBuilder)]
#[builder(doc)]
pub struct Verifier<S>
where
    S: VerificationStrategy,
{
    #[builder(setter(
        transform = |mut fqdn: String| {
            // Since this is supposed to be a FQDN, we can just push a dot to the end of it
            // This will speed up the query to the resolver
            if !fqdn.ends_with('.') {
                fqdn.push('.');
            }

            fqdn
        }
    ))]
    fqdn: String,
    strategy: S,

    #[builder(
        default = OpaqueDebug(default_resolver()),
        setter(transform = |resolver: Arc<dyn DnsResolver>| OpaqueDebug(resolver))
    )]
    resolver: OpaqueDebug<Arc<dyn DnsResolver>>,
}

impl<S> Verifier<S>
where
    S: VerificationStrategy,
{
    /// Return the FQDN
    #[must_use]
    pub fn fqdn(&self) -> &str {
        &self.fqdn
    }

    /// Return verification strategy
    pub fn strategy(&self) -> &S {
        &self.strategy
    }

    /// Verify whether the TXT records of the FQDN pass the verification strategy
    ///
    /// Returns `Ok(())` when the check succeeded and the token is present
    #[instrument(skip_all, fields(%self.fqdn))]
    pub async fn verify(&self) -> Result<()> {
        let txt_records = self
            .resolver
            .lookup_txt(&self.fqdn)
            .await
            .map_err(Error::Resolve)?;

        let is_valid = self
            .strategy
            .verify(txt_records.iter().map(Deref::deref))
            .await
            .map_err(|err| Error::VerificationStrategy(err.into()))?;

        is_valid.then_some(()).ok_or(Error::Unverified)
    }
}

#[cfg(test)]
mod test {
    use crate::{BoxError, Coerce, DnsResolver, DummyStrategy, Error, KeyValueStrategy, Verifier};
    use async_trait::async_trait;
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;
    use triomphe::Arc;

    const RNG_SEED: [u8; 16] = *b"im breaking down";

    #[derive(Clone)]
    struct TestResolver {
        expected_fqdn: String,
        records: Vec<String>,
    }

    #[async_trait]
    impl DnsResolver for TestResolver {
        async fn lookup_txt(&self, fqdn: &str) -> Result<Vec<String>, BoxError> {
            assert_eq!(fqdn, self.expected_fqdn);
            Ok(self.records.clone())
        }
    }

    #[tokio::test]
    async fn default_resolver_works() {
        let dummy = DummyStrategy::default();
        let resolver = crate::default_resolver();

        let verifier = Verifier::builder()
            .fqdn("example.org.".into())
            .resolver(resolver)
            .strategy(dummy)
            .build();

        assert!(verifier.verify().await.is_ok());
    }

    #[tokio::test]
    async fn dummy_strategy_works() {
        let dummy = DummyStrategy::default();
        let resolver = TestResolver {
            expected_fqdn: "aumetra.xyz.".into(),
            records: vec![],
        };

        let verifier = Verifier::builder()
            .fqdn("aumetra.xyz.".into())
            .resolver(Arc::new(resolver).coerce())
            .strategy(dummy)
            .build();

        assert!(verifier.verify().await.is_ok());
    }

    #[tokio::test]
    async fn rejects_invalid_records() {
        let kv_strategy =
            KeyValueStrategy::generate(&mut XorShiftRng::from_seed(RNG_SEED), "key".into());
        let mut resolver = TestResolver {
            expected_fqdn: "aumetra.xyz.".into(),
            records: vec![],
        };

        let verifier = Verifier::builder()
            .fqdn("aumetra.xyz.".into())
            .resolver(Arc::new(resolver.clone()).coerce())
            .strategy(kv_strategy.clone())
            .build();
        assert!(matches!(verifier.verify().await, Err(Error::Unverified)));

        resolver.records = vec![format!("{}=obviously_not_valid", kv_strategy.key)];

        let verifier = Verifier::builder()
            .fqdn("aumetra.xyz.".into())
            .resolver(Arc::new(resolver.clone()).coerce())
            .strategy(kv_strategy.clone())
            .build();
        assert!(matches!(verifier.verify().await, Err(Error::Unverified)));
    }

    #[tokio::test]
    async fn validation_works() {
        let kv_strategy =
            KeyValueStrategy::generate(&mut XorShiftRng::from_seed(RNG_SEED), "key".into());
        let txt_record = format!("{}={}", kv_strategy.key, kv_strategy.value);

        let resolver = TestResolver {
            expected_fqdn: "aumetra.xyz.".into(),
            records: vec![txt_record],
        };
        let verifier = Verifier::builder()
            .fqdn("aumetra.xyz.".into())
            .resolver(Arc::new(resolver).coerce())
            .strategy(kv_strategy)
            .build();

        let result = verifier.verify().await;
        assert!(result.is_ok());
    }

    #[test]
    fn key_value_strategy_schema() {
        let kv_strategy =
            KeyValueStrategy::generate(&mut XorShiftRng::from_seed(RNG_SEED), "key".into());
        insta::assert_json_snapshot!(kv_strategy);
    }

    #[test]
    fn normalizes_to_fqdn() {
        let domain_verifier = Verifier::builder()
            .fqdn("aumetra.xyz".into())
            .strategy(DummyStrategy::default())
            .build();
        assert_eq!(domain_verifier.fqdn(), "aumetra.xyz.");

        let fqdn_verifier = Verifier::builder()
            .fqdn("aumetra.xyz.".into())
            .strategy(DummyStrategy::default())
            .build();
        assert_eq!(fqdn_verifier.fqdn(), "aumetra.xyz.");
    }
}
