use crate::mrf_wit::v1::fep::mrf::keyvalue;
use async_trait::async_trait;
use derive_more::From;
use enum_dispatch::enum_dispatch;
use std::{error::Error, future::Future};
use wasmtime::component::Resource;

pub use self::{
    fs::{FsBackend, FsBucketBackend},
    redis::{RedisBackend, RedisBucketBackend},
};

mod fs;
mod redis;

type BoxError = Box<dyn Error + Send + Sync>;

pub trait Backend {
    type Bucket: BucketBackend;

    fn open(&self, name: &str) -> impl Future<Output = Result<Self::Bucket, BoxError>> + Send;
}

#[enum_dispatch]
#[allow(async_fn_in_trait)]
pub trait BucketBackend {
    async fn exists(&self, key: &str) -> Result<bool, BoxError>;
    async fn delete(&self, key: &str) -> Result<(), BoxError>;
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, BoxError>;
    async fn set(&self, key: &str, value: &[u8]) -> Result<(), BoxError>;
}

#[derive(From)]
pub enum BackendDispatch {
    Fs(FsBackend),
    Redis(RedisBackend),
}

impl Backend for BackendDispatch {
    type Bucket = BucketBackendDispatch;

    async fn open(&self, name: &str) -> Result<Self::Bucket, BoxError> {
        match self {
            Self::Fs(fs) => fs.open(name).await.map(Into::into),
            Self::Redis(redis) => redis.open(name).await.map(Into::into),
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
    async fn open_bucket(
        &mut self,
        name: String,
    ) -> wasmtime::Result<Result<Resource<keyvalue::Bucket>, Resource<keyvalue::Error>>> {
        let bucket = match self.kv_ctx.storage.open(&name).await {
            Ok(bucket) => bucket,
            Err(error) => {
                error!(?error, "failed to open bucket");
                return Ok(Err(Resource::new_own(0)));
            }
        };

        let idx = self.kv_ctx.buckets.insert(bucket);
        Ok(Ok(Resource::new_own(idx as u32)))
    }

    fn drop(&mut self, rep: Resource<keyvalue::Bucket>) -> wasmtime::Result<()> {
        self.kv_ctx.buckets.remove(rep.rep() as usize);
        Ok(())
    }
}

#[async_trait]
impl keyvalue::HostError for crate::ctx::Context {
    fn drop(&mut self, _rep: Resource<keyvalue::Error>) -> wasmtime::Result<()> {
        Ok(())
    }
}

#[async_trait]
impl keyvalue::Host for crate::ctx::Context {
    async fn get(
        &mut self,
        bucket: Resource<keyvalue::Bucket>,
        key: String,
    ) -> wasmtime::Result<Result<Option<Vec<u8>>, Resource<keyvalue::Error>>> {
        let bucket = &self.kv_ctx.buckets[bucket.rep() as usize];
        match bucket.get(&key).await {
            Ok(val) => Ok(Ok(val)),
            Err(error) => {
                error!(?error, %key, "failed to get key from storage");
                Ok(Err(Resource::new_own(0)))
            }
        }
    }

    async fn set(
        &mut self,
        bucket: Resource<keyvalue::Bucket>,
        key: String,
        value: Vec<u8>,
    ) -> wasmtime::Result<Result<(), Resource<keyvalue::Error>>> {
        let bucket = &self.kv_ctx.buckets[bucket.rep() as usize];
        match bucket.set(&key, &value).await {
            Ok(()) => Ok(Ok(())),
            Err(error) => {
                error!(?error, %key, "failed to set key in storage");
                Ok(Err(Resource::new_own(0)))
            }
        }
    }

    async fn delete(
        &mut self,
        bucket: Resource<keyvalue::Bucket>,
        key: String,
    ) -> wasmtime::Result<Result<(), Resource<keyvalue::Error>>> {
        let bucket = &self.kv_ctx.buckets[bucket.rep() as usize];
        match bucket.delete(&key).await {
            Ok(()) => Ok(Ok(())),
            Err(error) => {
                error!(?error, %key, "failed to delete key from storage");
                Ok(Err(Resource::new_own(0)))
            }
        }
    }

    async fn exists(
        &mut self,
        bucket: Resource<keyvalue::Bucket>,
        key: String,
    ) -> wasmtime::Result<Result<bool, Resource<keyvalue::Error>>> {
        let bucket = &self.kv_ctx.buckets[bucket.rep() as usize];
        match bucket.exists(&key).await {
            Ok(exists) => Ok(Ok(exists)),
            Err(error) => {
                error!(?error, %key, "failed to check existence of key in storage");
                Ok(Err(Resource::new_own(0)))
            }
        }
    }
}
