use super::{Result, SearchBackend, SearchIndex, SearchItem, SearchResultReference};
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::Stream;
use http::header::CONTENT_TYPE;
use meilisearch_sdk::{client::Client, indexes::Index, settings::Settings};
use pin_project_lite::pin_project;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use speedy_uuid::Uuid;
use std::{
    io,
    pin::Pin,
    task::{self, ready, Poll},
};
use strum::IntoEnumIterator;

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
    R: futures_io::AsyncRead,
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
struct HttpClient {
    inner: kitsune_http_client::Client,
}

#[async_trait]
impl meilisearch_sdk::request::HttpClient for HttpClient {
    async fn stream_request<
        Query: Serialize + Send + Sync,
        Body: futures_io::AsyncRead + Send + Sync + 'static,
        Output: DeserializeOwned + 'static,
    >(
        &self,
        url: &str,
        method: meilisearch_sdk::request::Method<Query, Body>,
        content_type: &str,
        expected_status_code: u16,
    ) -> Result<Output, meilisearch_sdk::errors::Error> {
        let url = format!(
            "{url}?{}",
            serde_urlencoded::to_string(method.query())
                .map_err(|err| meilisearch_sdk::errors::Error::Other(err.into()))?
        );

        let request = http::Request::builder()
            .uri(&url)
            .header(CONTENT_TYPE, content_type);

        let request = match method {
            meilisearch_sdk::request::Method::Get { .. } => request.method(http::Method::GET),
            meilisearch_sdk::request::Method::Post { .. } => request.method(http::Method::POST),
            meilisearch_sdk::request::Method::Patch { .. } => request.method(http::Method::PATCH),
            meilisearch_sdk::request::Method::Put { .. } => request.method(http::Method::PUT),
            meilisearch_sdk::request::Method::Delete { .. } => request.method(http::Method::DELETE),
        };

        let body = method
            .map_body(|body| {
                kitsune_http_client::Body::stream(AsyncReadBridge::new(body, BUFFER_SIZE))
            })
            .into_body()
            .unwrap_or_else(kitsune_http_client::Body::empty);

        let request = request
            .body(body)
            .map_err(|err| meilisearch_sdk::errors::Error::Other(err.into()))?;

        let response = self
            .inner
            .execute(request)
            .await
            .map_err(|err| meilisearch_sdk::errors::Error::Other(err.into()))?;

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
            .map_err(|err| meilisearch_sdk::errors::Error::Other(err.into()))
    }
}

#[derive(Deserialize)]
struct MeilisearchResult {
    id: Uuid,
}

#[derive(Clone)]
pub struct MeiliSearchService {
    client: Client<HttpClient>,
}

impl MeiliSearchService {
    /// Connect to the Meilisearch instance and initialise the indices
    ///
    /// # Errors
    ///
    /// - Failed to connect to the instance
    pub async fn new(host: &str, api_key: &str) -> Result<Self> {
        let http_client = HttpClient {
            inner: kitsune_http_client::Client::builder()
                .content_length_limit(None)
                .build(),
        };
        let service = Self {
            client: Client::new_with_client(host, Some(api_key), http_client),
        };

        let settings = Settings::new()
            .with_filterable_attributes(["created_at"])
            .with_sortable_attributes(["id"]);

        for index in SearchIndex::iter() {
            service
                .get_index(index)
                .set_settings(&settings)
                .await?
                .wait_for_completion(&service.client, None, None)
                .await?;
        }

        Ok(service)
    }

    fn get_index(&self, index: SearchIndex) -> Index<HttpClient> {
        self.client.index(index.as_ref())
    }
}

impl SearchBackend for MeiliSearchService {
    #[instrument(skip_all)]
    async fn add_to_index(&self, item: SearchItem) -> Result<()> {
        self.get_index(item.index())
            .add_documents(&[item], Some("id"))
            .await?
            .wait_for_completion(&self.client, None, None)
            .await?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn remove_from_index(&self, item: &SearchItem) -> Result<()> {
        match item {
            SearchItem::Account(account) => {
                self.get_index(SearchIndex::Account)
                    .delete_document(account.id)
                    .await?
            }
            SearchItem::Post(post) => {
                self.get_index(SearchIndex::Post)
                    .delete_document(post.id)
                    .await?
            }
        }
        .wait_for_completion(&self.client, None, None)
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn reset_index(&self, index: SearchIndex) -> Result<()> {
        self.get_index(index)
            .delete_all_documents()
            .await?
            .wait_for_completion(&self.client, None, None)
            .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn search(
        &self,
        index: SearchIndex,
        query: &str,
        max_results: u64,
        offset: u64,
        min_id: Option<Uuid>,
        max_id: Option<Uuid>,
    ) -> Result<Vec<SearchResultReference>> {
        let min_timestamp = min_id.map_or(u64::MIN, |id| {
            let (created_at_secs, _) = id.get_timestamp().unwrap().to_unix();
            created_at_secs
        });
        let max_timestamp = max_id.map_or(u64::MAX, |id| {
            let (created_at_secs, _) = id.get_timestamp().unwrap().to_unix();
            created_at_secs
        });

        let filter = format!("created_at > {min_timestamp} AND created_at < {max_timestamp}");
        #[allow(clippy::cast_possible_truncation)]
        let results = self
            .get_index(index)
            .search()
            .with_query(query)
            .with_filter(&filter)
            .with_sort(&["id:desc"])
            .with_offset(offset as usize)
            .with_limit(max_results as usize)
            .execute::<MeilisearchResult>()
            .await?;

        Ok(results
            .hits
            .into_iter()
            .map(|item| SearchResultReference {
                index,
                id: item.result.id,
            })
            .collect())
    }

    #[instrument(skip_all)]
    async fn update_in_index(&self, item: SearchItem) -> Result<()> {
        self.get_index(item.index())
            .add_or_update(&[item], Some("id"))
            .await?
            .wait_for_completion(&self.client, None, None)
            .await?;

        Ok(())
    }
}
