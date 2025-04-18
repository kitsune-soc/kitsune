#![doc = include_str!("../README.md")]

use bytes::Bytes;
use derive_more::From;
use futures_util::{Stream, StreamExt};
use kitsune_error::Result;

pub mod fs;
pub mod s3;

/// Trait abstraction over storage backends
pub trait StorageBackend: Clone + Send + Sync {
    /// Delete something from the object storage
    fn delete(&self, path: &str) -> impl Future<Output = Result<()>>;

    /// Stream something from the object storage
    fn get(
        &self,
        path: &str,
    ) -> impl Future<Output = Result<impl Stream<Item = Result<Bytes>> + 'static>>;

    /// Stream something onto the object storage
    fn put<T>(&self, path: &str, input_stream: T) -> impl Future<Output = Result<()>>
    where
        T: Stream<Item = Result<Bytes>> + Send + Sync + 'static;
}

#[derive(Clone, From)]
/// Combined storage enum for enum dispatch
pub enum AnyStorageBackend {
    /// File system-backed storage
    Fs(fs::Storage),

    /// S3-backed storage
    S3(s3::Storage),
}

impl StorageBackend for AnyStorageBackend {
    async fn delete(&self, path: &str) -> Result<()> {
        match self {
            Self::Fs(fs) => fs.delete(path).await,
            Self::S3(s3) => s3.delete(path).await,
        }
    }

    async fn get(&self, path: &str) -> Result<impl Stream<Item = Result<Bytes>> + 'static> {
        match self {
            Self::Fs(fs) => fs.get(path).await.map(StreamExt::left_stream),
            Self::S3(s3) => s3.get(path).await.map(StreamExt::right_stream),
        }
    }

    async fn put<T>(&self, path: &str, input_stream: T) -> Result<()>
    where
        T: Stream<Item = Result<Bytes>> + Send + Sync + 'static,
    {
        match self {
            Self::Fs(fs) => fs.put(path, input_stream).await,
            Self::S3(s3) => s3.put(path, input_stream).await,
        }
    }
}
