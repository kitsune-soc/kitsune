use async_trait::async_trait;
use bytes::Bytes;
use futures_util::Stream;
use http::header::CONTENT_TYPE;
use kitsune_http_client::Body as HttpBody;
use meilisearch_sdk::{errors::Error as MeilisearchError, request::Method};
use pin_project_lite::pin_project;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    io,
    pin::Pin,
    task::{self, Poll, ready},
};

const BUFFER_SIZE: usize = 1024;

pin_project! {
    struct AsyncReadBridge<R> {
        #[pin]
        inner: R,
        buf: Vec<u8>,
    }
}

impl<R> AsyncReadBridge<R> {
    pub fn new(reader: R, buf_size: usize) -> Self {
        Self {
            inner: reader,
            buf: vec![0; buf_size],
        }
    }
}

impl<R> Stream for AsyncReadBridge<R>
where
    R: futures_util::AsyncRead,
{
    type Item = io::Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let amount_read = match ready!(this.inner.poll_read(cx, this.buf)) {
            Ok(0) => return Poll::Ready(None),
            Ok(amount_read) => amount_read,
            Err(err) => return Poll::Ready(Some(Err(err))),
        };

        let bytes = Bytes::copy_from_slice(&this.buf[..amount_read]);
        this.buf.clear();
        this.buf.fill(0);

        Poll::Ready(Some(Ok(bytes)))
    }
}

#[derive(Clone)]
pub struct HttpClient {
    pub inner: kitsune_http_client::Client,
}

#[async_trait]
impl meilisearch_sdk::request::HttpClient for HttpClient {
    async fn stream_request<
        Query: Serialize + Send + Sync,
        Body: futures_util::AsyncRead + Send + Sync + 'static,
        Output: DeserializeOwned + 'static,
    >(
        &self,
        url: &str,
        method: Method<Query, Body>,
        content_type: &str,
        expected_status_code: u16,
    ) -> Result<Output, MeilisearchError> {
        let url = format!(
            "{url}?{}",
            serde_urlencoded::to_string(method.query())
                .map_err(|err| MeilisearchError::Other(err.into()))?
        );

        let request = http::Request::builder()
            .uri(&url)
            .header(CONTENT_TYPE, content_type);

        let request = match method {
            Method::Get { .. } => request.method(http::Method::GET),
            Method::Post { .. } => request.method(http::Method::POST),
            Method::Patch { .. } => request.method(http::Method::PATCH),
            Method::Put { .. } => request.method(http::Method::PUT),
            Method::Delete { .. } => request.method(http::Method::DELETE),
        };

        let body = method
            .map_body(|body| HttpBody::stream(AsyncReadBridge::new(body, BUFFER_SIZE)))
            .into_body()
            .unwrap_or_default();

        let request = request
            .body(body)
            .map_err(|err| MeilisearchError::Other(err.into()))?;

        let response = self
            .inner
            .execute(request)
            .await
            .map_err(|err| MeilisearchError::Other(err.into()))?;

        if response.status().as_u16() != expected_status_code {
            return Err(meilisearch_sdk::errors::MeilisearchCommunicationError {
                status_code: response.status().as_u16(),
                message: response.text().await.ok(),
                url,
            }
            .into());
        }

        response
            .json()
            .await
            .map_err(|err| MeilisearchError::Other(err.into()))
    }
}
