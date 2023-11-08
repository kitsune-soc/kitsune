#![doc = include_str!("../README.md")]
#![forbid(missing_docs, rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(forbidden_lint_groups)]

use bytes::Bytes;
use enum_dispatch::enum_dispatch;
use futures_util::{stream::BoxStream, Stream};
use std::error::Error;

pub mod fs;
pub mod s3;

/// Boxed error
pub type BoxError = Box<dyn Error + Send + Sync>;

/// Result alias where the error defaults to [`BoxError`]
pub type Result<T, E = BoxError> = std::result::Result<T, E>;

/// Trait abstraction over storage backends
#[enum_dispatch]
#[allow(async_fn_in_trait)] // Because of `enum_dispatch`
pub trait StorageBackend: Clone + Send + Sync {
    /// Delete something from the object storage
    async fn delete(&self, path: &str) -> Result<()>;

    /// Stream something from the object storage
    async fn get(&self, path: &str) -> Result<BoxStream<'static, Result<Bytes>>>;

    /// Stream something onto the object storage
    async fn put<T>(&self, path: &str, input_stream: T) -> Result<()>
    where
        T: Stream<Item = Result<Bytes>> + Send + 'static;
}

#[derive(Clone)]
#[enum_dispatch(StorageBackend)]
/// Combined storage enum for enum dispatch
pub enum Storage {
    /// File system-backed storage
    Fs(fs::Storage),

    /// S3-backed storage
    S3(s3::Storage),
}
