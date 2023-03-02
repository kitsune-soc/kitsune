//!
//! File system backed implementation of the [`StorageBackend`] trait
//!

use crate::{Result, StorageBackend};
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{stream::BoxStream, Stream, StreamExt, TryStreamExt};
use std::path::PathBuf;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tokio_util::io::ReaderStream;

/// File system storage
pub struct Storage {
    storage_dir: PathBuf,
}

impl Storage {
    /// Create a new file system storage
    ///
    /// It always requires a directory the operations on it are relative to
    #[must_use]
    pub fn new(storage_dir: PathBuf) -> Self {
        Self { storage_dir }
    }
}

#[async_trait]
impl StorageBackend for Storage {
    async fn delete(&self, path: &str) -> Result<()> {
        fs::remove_file(self.storage_dir.join(path)).await?;
        Ok(())
    }

    async fn get<'a>(&'a self, path: &str) -> Result<BoxStream<'a, Result<Bytes>>> {
        let file = File::open(self.storage_dir.join(path)).await?;
        Ok(ReaderStream::new(file).map_err(Into::into).boxed())
    }

    async fn put(
        &self,
        path: &str,
        input_stream: &mut (dyn Stream<Item = Result<Bytes>> + Send + Unpin),
    ) -> Result<()> {
        let mut file = File::create(self.storage_dir.join(path)).await?;
        while let Some(chunk) = input_stream.next().await.transpose()? {
            file.write_all(&chunk).await?;
        }
        Ok(())
    }
}
