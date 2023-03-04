//!
//! An S3 backed implementation of the [`StorageBackend`] trait
//!

use crate::{BoxError, Result, StorageBackend};
use async_trait::async_trait;
use aws_sdk_s3::{types::ByteStream, Client, Config};
use aws_smithy_http::body::{BoxBody, SdkBody};
use bytes::Bytes;
use futures_util::{stream::BoxStream, Stream, StreamExt, TryStreamExt};
use http::HeaderMap;
use http_body::Body;
use pin_project_lite::pin_project;
use std::{
    pin::Pin,
    task::{self, Poll},
};
use sync_wrapper::SyncWrapper;

pin_project! {
    struct StreamBody<S> {
        #[pin]
        inner: SyncWrapper<S>,
    }
}

impl<S> Body for StreamBody<S>
where
    S: Stream<Item = Result<Bytes>>,
{
    type Data = Bytes;
    type Error = BoxError;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();
        this.inner.get_pin_mut().poll_next(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}

/// S3-backed storage
pub struct Storage {
    bucket_name: String,
    client: Client,
}

impl Storage {
    /// Create a new storage instance
    #[must_use]
    pub fn new(bucket_name: String, config: Config) -> Self {
        Self {
            bucket_name,
            client: Client::from_conf(config),
        }
    }
}

#[async_trait]
impl StorageBackend for Storage {
    async fn delete(&self, path: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(path)
            .send()
            .await?;

        Ok(())
    }

    async fn get(&self, path: &str) -> Result<BoxStream<'static, Result<Bytes>>> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(path)
            .send()
            .await?;

        Ok(response.body.map_err(Into::into).boxed())
    }

    async fn put(&self, path: &str, input_stream: BoxStream<'static, Result<Bytes>>) -> Result<()> {
        let body = BoxBody::new(StreamBody {
            inner: SyncWrapper::new(input_stream),
        });

        self.client
            .put_object()
            .bucket(&self.bucket_name)
            .key(path)
            .body(ByteStream::new(SdkBody::from_dyn(body)))
            .send()
            .await?;

        Ok(())
    }
}
