#![doc = include_str!("../README.md")]
#![forbid(missing_docs, rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(forbidden_lint_groups, clippy::needless_pass_by_value)]

#[macro_use]
extern crate tracing;

use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    error::ResolveError,
    TokioAsyncResolver,
};
use rand::{
    distributions::{Alphanumeric, DistString},
    RngCore,
};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, error::Error as StdError, future::Future, str};

type BoxError = Box<dyn StdError + Send + Sync>;
type Result<T, E = Error> = std::result::Result<T, E>;

/// Combined error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The builder was incomplete
    ///
    /// The field is the name of the missing field
    #[error("Incomplete Builder: Field \"{0}\" is missing from the builder")]
    IncompleteBuilder(&'static str),

    /// The resolver returned an error
    #[error(transparent)]
    Resolve(#[from] ResolveError),

    /// The verification strategy errored out
    #[error(transparent)]
    VerificationStrategy(BoxError),

    /// The domain did not have the required TXT record
    #[error("The domain did not have a TXT record matching the requirements")]
    Unverified,
}

/// Domain verification strategy
pub trait VerificationStrategy {
    /// Error returned by this verification strategy
    type Error: StdError + Send + Sync + 'static;

    /// Verify whether the domain is valid by looking at its TXT records
    fn verify<'a>(
        &self,
        txt_records: impl Iterator<Item = &'a str> + Send,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send;
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
            value: Alphanumeric.sample_string(rng, 40),
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

/// Verifier for a domain
///
/// De-/Serializable via `serde` for easy storage. Changes to the serialised structure are considered semver breaking
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Verifier<S>
where
    S: VerificationStrategy,
{
    domain: String,
    strategy: S,
}

impl<S> Verifier<S>
where
    S: VerificationStrategy,
{
    /// Create a new verifier
    pub fn new(domain: String, strategy: S) -> Self {
        Self { domain, strategy }
    }

    /// Return the domain
    #[must_use]
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Return verification strategy
    pub fn strategy(&self) -> &S {
        &self.strategy
    }

    /// Verify whether the domain has the specified token in the TXT records
    ///
    /// Returns `Ok(())` when the check succeeded and the token is present
    #[instrument(skip_all, fields(%self.domain))]
    pub async fn verify(&self) -> Result<()> {
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
        let txt_records = resolver.txt_lookup(&self.domain).await?;

        let txt_record_iter = txt_records.iter().flat_map(|record| {
            record
                .txt_data()
                .iter()
                .filter_map(|data| str::from_utf8(data).ok())
        });

        let is_valid = self
            .strategy
            .verify(txt_record_iter)
            .await
            .map_err(|err| Error::VerificationStrategy(err.into()))?;

        is_valid.then_some(()).ok_or(Error::Unverified)
    }
}
