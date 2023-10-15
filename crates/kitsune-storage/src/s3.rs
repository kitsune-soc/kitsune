//!
//! An S3 backed implementation of the [`StorageBackend`] trait
//!

use crate::{Result, StorageBackend};
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{stream::BoxStream, Stream, StreamExt, TryStreamExt};
use http::Request;
use hyper::Body;
use kitsune_http_client::Client as HttpClient;
use rusty_s3::{
    actions::{DeleteObject, GetObject, PutObject},
    Bucket, Credentials, S3Action,
};
use std::{sync::Arc, time::Duration};

const FIVE_MINUTES: Duration = Duration::from_secs(5 * 60);

const fn s3_method_to_http(method: rusty_s3::Method) -> http::Method {
    match method {
        rusty_s3::Method::Head => http::Method::HEAD,
        rusty_s3::Method::Get => http::Method::GET,
        rusty_s3::Method::Post => http::Method::POST,
        rusty_s3::Method::Put => http::Method::PUT,
        rusty_s3::Method::Delete => http::Method::DELETE,
    }
}

struct S3Client {
    bucket: Bucket,
    credentials: Credentials,
    http_client: HttpClient,
}

impl S3Client {
    pub async fn delete_object(&self, path: &str) -> Result<()> {
        let delete_action = self.bucket.delete_object(Some(&self.credentials), path);

        let request = Request::builder()
            .uri(String::from(delete_action.sign(FIVE_MINUTES)))
            .method(s3_method_to_http(DeleteObject::METHOD))
            .body(Body::empty())?;

        self.http_client.execute(request).await?;

        Ok(())
    }

    pub async fn get_object(&self, path: &str) -> Result<impl Stream<Item = Result<Bytes>>> {
        let get_action = self.bucket.get_object(Some(&self.credentials), path);

        let request = Request::builder()
            .uri(String::from(get_action.sign(FIVE_MINUTES)))
            .method(s3_method_to_http(GetObject::METHOD))
            .body(Body::empty())?;

        let response = self.http_client.execute(request).await?;

        Ok(response.stream().map_err(Into::into))
    }

    pub async fn put_object<S>(&self, path: &str, stream: S) -> Result<()>
    where
        S: Stream<Item = Result<Bytes>> + Send + 'static,
    {
        let put_action = self.bucket.put_object(Some(&self.credentials), path);

        let request = Request::builder()
            .uri(String::from(put_action.sign(FIVE_MINUTES)))
            .method(s3_method_to_http(PutObject::METHOD))
            .body(Body::wrap_stream(stream))?;

        self.http_client.execute(request).await?;

        Ok(())
    }
}

#[derive(Clone)]
/// S3-backed storage
pub struct Storage {
    client: Arc<S3Client>,
}

impl Storage {
    /// Create a new storage instance
    #[must_use]
    pub fn new(bucket: Bucket, credentials: Credentials) -> Self {
        let http_client = HttpClient::builder().content_length_limit(None).build();

        Self {
            client: Arc::new(S3Client {
                bucket,
                credentials,
                http_client,
            }),
        }
    }
}

#[async_trait]
impl StorageBackend for Storage {
    async fn delete(&self, path: &str) -> Result<()> {
        self.client.delete_object(path).await
    }

    async fn get(&self, path: &str) -> Result<BoxStream<'static, Result<Bytes>>> {
        let stream = self.client.get_object(path).await?.boxed();
        Ok(stream)
    }

    async fn put<S>(&self, path: &str, input_stream: S) -> Result<()>
    where
        S: Stream<Item = Result<Bytes>> + Send + 'static,
    {
        self.client.put_object(path, input_stream).await
    }
}
