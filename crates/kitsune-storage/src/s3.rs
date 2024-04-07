//!
//! An S3 backed implementation of the [`StorageBackend`] trait
//!

use crate::StorageBackend;
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use kitsune_error::Result;
use rusty_s3::{Bucket, Credentials};
use std::sync::Arc;

#[derive(Clone)]
/// S3-backed storage
pub struct Storage {
    client: Arc<kitsune_s3::Client>,
}

impl Storage {
    /// Create a new storage instance
    #[must_use]
    pub fn new(bucket: Bucket, credentials: Credentials) -> Self {
        let s3_client = kitsune_s3::Client::builder()
            .bucket(bucket)
            .credentials(credentials)
            .build();

        Self::from(Arc::new(s3_client))
    }
}

impl From<Arc<kitsune_s3::Client>> for Storage {
    fn from(client: Arc<kitsune_s3::Client>) -> Self {
        Self { client }
    }
}

impl StorageBackend for Storage {
    async fn delete(&self, path: &str) -> Result<()> {
        self.client.delete_object(path).await
    }

    async fn get(&self, path: &str) -> Result<impl Stream<Item = Result<Bytes>> + 'static> {
        let stream = self.client.get_object(path).await?.boxed();
        Ok(stream)
    }

    async fn put<S>(&self, path: &str, input_stream: S) -> Result<()>
    where
        S: Stream<Item = Result<Bytes>> + Send + Sync + 'static,
    {
        self.client.put_object(path, input_stream).await
    }
}
