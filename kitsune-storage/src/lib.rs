#![doc = include_str!("../README.md")]
#![forbid(missing_docs, rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{stream::BoxStream, Stream};
use std::error::Error;

pub mod fs;
pub mod s3;

/// Boxed error
pub type BoxError = Box<dyn Error + Send + Sync>;

/// Result alias where the error defaults to [`BoxError`]
pub type Result<T, E = BoxError> = std::result::Result<T, E>;

/// Trait abstraction over storage backends
#[async_trait]
pub trait StorageBackend {
    /// Delete something from the object storage
    async fn delete(&self, path: &str) -> Result<()>;

    /// Stream something from the object storage
    async fn get<'a>(&'a self, path: &str) -> Result<BoxStream<'a, Result<Bytes>>>;

    /// Stream something onto the object storage
    async fn put(
        &self,
        path: &str,
        input_stream: &mut (dyn Stream<Item = Result<Bytes>> + Send + Unpin),
    ) -> Result<()>;
}

#[cfg(test)]
mod test {
    use crate::StorageBackend;

    #[test]
    fn assert_object_safety() {
        let _: Box<dyn StorageBackend>;
    }
}
