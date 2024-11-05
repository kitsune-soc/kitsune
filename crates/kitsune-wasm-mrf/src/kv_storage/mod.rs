use crate::mrf_wit::v1::fep::mrf::keyvalue;
use async_trait::async_trait;
use color_eyre::eyre;
use derive_more::From;
use enum_dispatch::enum_dispatch;
use std::future::Future;
use wasmtime::component::Resource;

pub use self::{
    fs::{FsBackend, FsBucketBackend},
    redis::{RedisBackend, RedisBucketBackend},
};

mod fs;
mod redis;

pub trait Backend {
    type Bucket: BucketBackend;

    fn open(
        &self,
        module_name: &str,
        name: &str,
    ) -> impl Future<Output = eyre::Result<Self::Bucket>> + Send;
}

#[enum_dispatch]
#[allow(async_fn_in_trait)]
pub trait BucketBackend {
    async fn exists(&self, key: &str) -> eyre::Result<bool>;
    async fn delete(&self, key: &str) -> eyre::Result<()>;
    async fn get(&self, key: &str) -> eyre::Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: &[u8]) -> eyre::Result<()>;
}

#[derive(From)]
pub enum BackendDispatch {
    Fs(FsBackend),
    Redis(RedisBackend),
}

impl Backend for BackendDispatch {
    type Bucket = BucketBackendDispatch;

    async fn open(&self, module_name: &str, name: &str) -> eyre::Result<Self::Bucket> {
        match self {
            Self::Fs(fs) => fs.open(module_name, name).await.map(Into::into),
            Self::Redis(redis) => redis.open(module_name, name).await.map(Into::into),
        }
    }
}

#[enum_dispatch(BucketBackend)]
pub enum BucketBackendDispatch {
    Fs(FsBucketBackend),
    Redis(RedisBucketBackend),
}

#[async_trait]
impl keyvalue::HostBucket for crate::ctx::Context {
    async fn open(
        &mut self,
        name: String,
    ) -> Result<Resource<keyvalue::Bucket>, Resource<keyvalue::Error>> {
        let module_name = self
            .kv_ctx
            .module_name
            .as_ref()
            .expect("[Bug] Module name not set");

        let bucket = match self.kv_ctx.storage.open(&name, module_name).await {
            Ok(bucket) => bucket,
            Err(error) => {
                error!(?error, %module_name, %name, "failed to open bucket");
                return Err(Resource::new_own(0));
            }
        };

        let idx = self.kv_ctx.buckets.insert(bucket);
        Ok(Resource::new_own(idx as u32))
    }

    async fn get(
        &mut self,
        bucket: Resource<keyvalue::Bucket>,
        key: String,
    ) -> Result<Option<Vec<u8>>, Resource<keyvalue::Error>> {
        match self.kv_ctx.get_bucket(&bucket).get(&key).await {
            Ok(value) => Ok(value),
            Err(error) => {
                error!(?error, %key, "failed to get key from storage");
                Err(Resource::new_own(0))
            }
        }
    }

    async fn set(
        &mut self,
        bucket: Resource<keyvalue::Bucket>,
        key: String,
        value: Vec<u8>,
    ) -> Result<(), Resource<keyvalue::Error>> {
        match self.kv_ctx.get_bucket(&bucket).set(&key, &value).await {
            Ok(value) => Ok(value),
            Err(error) => {
                error!(?error, %key, "failed to set key in storage");
                Err(Resource::new_own(0))
            }
        }
    }

    async fn delete(
        &mut self,
        bucket: Resource<keyvalue::Bucket>,
        key: String,
    ) -> Result<(), Resource<keyvalue::Error>> {
        match self.kv_ctx.get_bucket(&bucket).delete(&key).await {
            Ok(value) => Ok(value),
            Err(error) => {
                error!(?error, %key, "failed to delete key from storage");
                Err(Resource::new_own(0))
            }
        }
    }

    async fn exists(
        &mut self,
        bucket: Resource<keyvalue::Bucket>,
        key: String,
    ) -> Result<bool, Resource<keyvalue::Error>> {
        match self.kv_ctx.get_bucket(&bucket).exists(&key).await {
            Ok(value) => Ok(value),
            Err(error) => {
                error!(?error, %key, "failed to check existence of key in storage");
                Err(Resource::new_own(0))
            }
        }
    }

    async fn drop(&mut self, rep: Resource<keyvalue::Bucket>) -> wasmtime::Result<()> {
        self.kv_ctx.buckets.remove(rep.rep() as usize);
        Ok(())
    }
}

#[async_trait]
impl keyvalue::HostError for crate::ctx::Context {
    async fn drop(&mut self, _rep: Resource<keyvalue::Error>) -> wasmtime::Result<()> {
        Ok(())
    }
}

#[async_trait]
impl keyvalue::Host for crate::ctx::Context {}
