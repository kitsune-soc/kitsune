use super::{Backend, BoxError, BucketBackend};
use miette::IntoDiagnostic;
use std::path::Path;

pub struct FsBackend {
    inner: sled::Db,
}

impl FsBackend {
    pub fn from_path<P>(path: P) -> miette::Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            inner: sled::open(path).into_diagnostic()?,
        })
    }
}

impl Backend for FsBackend {
    type Bucket = FsBucketBackend;

    async fn open(&self, name: &str) -> Result<Self::Bucket, BoxError> {
        self.inner
            .open_tree(name)
            .map(|tree| FsBucketBackend { inner: tree })
            .map_err(Into::into)
    }
}

pub struct FsBucketBackend {
    inner: sled::Tree,
}

impl BucketBackend for FsBucketBackend {
    async fn exists(&self, key: &str) -> Result<bool, BoxError> {
        self.inner.contains_key(key).map_err(Into::into)
    }

    async fn delete(&self, key: &str) -> Result<(), BoxError> {
        self.inner.remove(key)?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, BoxError> {
        self.inner
            .get(key)
            .map(|maybe_val| maybe_val.map(|val| val.to_vec()))
            .map_err(Into::into)
    }

    async fn set(&self, key: &str, value: &[u8]) -> Result<(), BoxError> {
        self.inner.insert(key, value)?;
        Ok(())
    }
}
