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

#[inline]
fn get_bucket<'a>(
    ctx: &'a crate::ctx::Context,
    rep: &Resource<keyvalue::Bucket>,
) -> &'a BucketBackendDispatch {
    &ctx.kv_ctx.buckets[rep.rep() as usize]
}

pub trait Backend {
    type Bucket: BucketBackend;

    fn open(
        &self,
        module_name: &str,
        name: &str,
    ) -> impl Future<Output = Result<Self::Bucket, BoxError>> + Send;
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

    async fn open(&self, module_name: &str, name: &str) -> Result<Self::Bucket, BoxError> {
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
        match get_bucket(self, &bucket).get(&key).await {
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
        match get_bucket(self, &bucket).set(&key, &value).await {
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
        match get_bucket(self, &bucket).delete(&key).await {
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
        match get_bucket(self, &bucket).exists(&key).await {
            Ok(value) => Ok(value),
            Err(error) => {
                error!(?error, %key, "failed to check existence of key in storage");
                Err(Resource::new_own(0))
            }
        }
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
impl keyvalue::Host for crate::ctx::Context {}
