use super::{Backend, BucketBackend};
use color_eyre::eyre;
use std::path::Path;

pub struct FsBackend {
    inner: sled::Db,
}

impl FsBackend {
    pub fn from_path<P>(path: P) -> eyre::Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            inner: sled::open(path)?,
        })
    }
}

impl Backend for FsBackend {
    type Bucket = FsBucketBackend;

    async fn open(&self, module_name: &str, name: &str) -> eyre::Result<Self::Bucket> {
        self.inner
            .open_tree(format!("{module_name}:{name}"))
            .map(|tree| FsBucketBackend { inner: tree })
            .map_err(Into::into)
    }
}

pub struct FsBucketBackend {
    inner: sled::Tree,
}

impl BucketBackend for FsBucketBackend {
    async fn exists(&self, key: &str) -> eyre::Result<bool> {
        self.inner.contains_key(key).map_err(Into::into)
    }

    async fn delete(&self, key: &str) -> eyre::Result<()> {
        self.inner.remove(key)?;
        Ok(())
    }

    async fn get(&self, key: &str) -> eyre::Result<Option<Vec<u8>>> {
        self.inner
            .get(key)
            .map(|maybe_val| maybe_val.map(|val| val.to_vec()))
            .map_err(Into::into)
    }

    async fn set(&self, key: &str, value: &[u8]) -> eyre::Result<()> {
        self.inner.insert(key, value)?;
        Ok(())
    }
}
