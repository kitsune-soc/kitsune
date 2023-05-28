#![doc = include_str!("../README.md")]
#![forbid(rust_2018_idioms, unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(forbidden_lint_groups)]

use self::util::BoxCloneService;
use futures_core::Stream;
use headers::{Date, HeaderMapExt};
use http_body::{combinators::BoxBody, Body as HttpBody, Limited};
use hyper::{
    body::Bytes,
    client::HttpConnector,
    header::{HeaderName, HOST, USER_AGENT},
    http::{self, HeaderValue},
    Body, Client as HyperClient, HeaderMap, Request, Response as HyperResponse, StatusCode, Uri,
    Version,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use kitsune_http_signatures::{HttpSigner, PrivateKey, SignatureComponent, SigningKey};
use pin_project_lite::pin_project;
use serde::de::DeserializeOwned;
use std::{
    error::Error as StdError,
    fmt,
    pin::Pin,
    task::{self, Poll},
    time::{Duration, SystemTime},
};
use tower::{layer::util::Identity, util::Either, BoxError, Service, ServiceBuilder, ServiceExt};
use tower_http::{
    decompression::DecompressionLayer, follow_redirect::FollowRedirectLayer,
    map_response_body::MapResponseBodyLayer, timeout::TimeoutLayer,
};

mod util;

type Result<T, E = Error> = std::result::Result<T, E>;

/// Default body limit of 1MB
const DEFAULT_BODY_LIMIT: usize = 1024 * 1024;

pin_project! {
    struct BodyStream<B> {
        #[pin]
        inner: B,
    }
}

impl<B> Stream for BodyStream<B>
where
    B: HttpBody<Data = Bytes, Error = BoxError>,
{
    type Item = Result<Bytes>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_data(cx).map_err(Into::into)
    }
}

/// Client error type
pub struct Error {
    inner: BoxError,
}

impl From<BoxError> for Error {
    fn from(value: BoxError) -> Self {
        Self { inner: value }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl StdError for Error {}

/// Builder for the HTTP client
pub struct ClientBuilder {
    content_length_limit: Option<usize>,
    default_headers: HeaderMap,
    timeout: Option<Duration>,
}

impl ClientBuilder {
    /// Set the content length limit
    ///
    /// This is enforced at the body level, regardless of whether the `Content-Type` header is set or not.
    ///
    /// Defaults to 1MB
    #[must_use]
    pub fn content_length_limit(self, content_length_limit: Option<usize>) -> Self {
        Self {
            content_length_limit,
            ..self
        }
    }

    /// Set a default header
    ///
    /// These headers are added to every HTTP request that is sent via this client
    ///
    /// # Errors
    ///
    /// - The header name failed to convert
    /// - The header value failed to convert
    pub fn default_header<K, V>(mut self, key: K, value: V) -> Result<Self>
    where
        K: TryInto<HeaderName>,
        K::Error: Into<BoxError>,
        V: TryInto<HeaderValue>,
        V::Error: Into<BoxError>,
    {
        self.default_headers.insert(
            key.try_into().map_err(|e| Error { inner: e.into() })?,
            value.try_into().map_err(Into::into)?,
        );

        Ok(self)
    }

    /// Set the User-Agent header
    ///
    /// Defaults to `kitsune-http-client`
    ///
    /// # Errors
    ///
    /// - The header value failed to convert
    pub fn user_agent<V>(self, value: V) -> Result<Self>
    where
        V: TryInto<HeaderValue>,
        V::Error: Into<BoxError>,
    {
        self.default_header(USER_AGENT, value)
    }

    /// Set a timeout
    ///
    /// By default there is no timeout
    #[must_use]
    pub fn timeout(self, timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
            ..self
        }
    }

    /// Build the HTTP client
    ///
    /// Yes, this operation is infallible
    #[must_use]
    pub fn build(self) -> Client {
        let connector = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            //.enable_http2() TODO: Find out why HTTP/2 causes a "400 Bad Request" with some servers.
            .build();

        let client: HyperClient<HttpsConnector<HttpConnector>, Body> =
            HyperClient::builder().build(connector);

        let content_length_limit = self.content_length_limit.map_or_else(
            || Either::B(MapResponseBodyLayer::new(BoxBody::new)),
            |limit| {
                Either::A(MapResponseBodyLayer::new(move |body| {
                    BoxBody::new(Limited::new(body, limit))
                }))
            },
        );
        let timeout = self.timeout.map_or_else(
            || Either::B(Identity::new()),
            |duration| Either::A(TimeoutLayer::new(duration)),
        );

        Client {
            default_headers: self.default_headers,
            inner: BoxCloneService::new(
                ServiceBuilder::new()
                    .layer(content_length_limit)
                    .layer(FollowRedirectLayer::new())
                    .layer(DecompressionLayer::default())
                    .layer(timeout)
                    .service(client),
            ),
        }
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        let builder = ClientBuilder {
            content_length_limit: Some(DEFAULT_BODY_LIMIT),
            default_headers: HeaderMap::default(),
            timeout: Option::default(),
        };

        builder.user_agent("kitsune-http-client").unwrap()
    }
}

#[derive(Clone)]
/// An opinionated HTTP client
pub struct Client {
    default_headers: HeaderMap,
    inner: BoxCloneService<Request<Body>, HyperResponse<BoxBody<Bytes, BoxError>>, BoxError>,
}

impl Client {
    /// Build a new client
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    fn prepare_request(&self, mut req: Request<Body>) -> Request<Body> {
        req.headers_mut()
            .extend(self.default_headers.clone().into_iter());

        req.headers_mut()
            .typed_insert(Date::from(SystemTime::now()));

        if let Some(host) = req.uri().host() {
            let value = HeaderValue::from_str(host).unwrap();
            req.headers_mut().insert(HOST, value);
        }

        req
    }

    /// Execute an HTTP request
    ///
    /// # Errors
    ///
    /// - The inner client service isn't ready
    /// - The request failed
    pub async fn execute(&self, req: Request<Body>) -> Result<Response> {
        let req = self.prepare_request(req);
        let response = self.inner.clone().ready().await?.call(req).await?;

        Ok(Response { inner: response })
    }

    /// Sign an HTTP request via HTTP signatures and execute it
    ///
    /// The headers need to include a `Digest` header, otherwise this function will error out.
    ///
    /// # Errors
    ///
    /// - Signing the request failed
    /// - Executing the request failed
    ///
    /// # Panics
    ///
    /// This should never panic. If it does, please open an issue.
    pub async fn execute_signed<K>(
        &self,
        req: Request<Body>,
        private_key: PrivateKey<'_, K>,
    ) -> Result<Response>
    where
        K: SigningKey + Send + 'static,
    {
        let req = self.prepare_request(req);
        let (mut parts, body) = req.into_parts();
        let (name, value) = HttpSigner::builder()
            .include_creation_timestamp(true)
            .expires_in(Duration::from_secs(30)) // Make the signature expire in 30 seconds
            .build()
            .unwrap()
            .sign(
                &parts,
                vec![
                    SignatureComponent::RequestTarget,
                    SignatureComponent::Header("Date"),
                    SignatureComponent::Header("Digest"),
                    SignatureComponent::Header("Host"),
                ],
                private_key,
            )
            .await
            .map_err(BoxError::from)?;

        parts.headers.insert(name, value);
        self.execute(Request::from_parts(parts, body)).await
    }

    /// Shorthand for creating a GET request
    ///
    /// # Errors
    ///
    /// - Creating the request with the provided URL failed
    /// - Request execution failed
    pub async fn get<U>(&self, uri: U) -> Result<Response>
    where
        Uri: TryFrom<U>,
        <Uri as TryFrom<U>>::Error: Into<http::Error>,
    {
        let req = Request::builder()
            .uri(uri)
            .body(Body::empty())
            .map_err(BoxError::from)?;

        self.execute(req).await
    }
}

impl Default for Client {
    fn default() -> Self {
        ClientBuilder::default().build()
    }
}

/// HTTP response
pub struct Response {
    inner: HyperResponse<BoxBody<Bytes, BoxError>>,
}

impl Response {
    /// Read the body into a `Bytes`
    ///
    /// # Errors
    ///
    /// Reading the body from the remote failed
    pub async fn bytes(self) -> Result<Bytes> {
        hyper::body::to_bytes(self.inner).await.map_err(Into::into)
    }

    /// Get a reference to the headers
    #[must_use]
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /// Read the body and attempt to interpret it as a UTF-8 encoded string
    ///
    /// # Errors
    ///
    /// - Reading the body from the remote failed
    /// - The body isn't a UTF-8 encoded string
    pub async fn text(self) -> Result<String> {
        let body = self.bytes().await?;
        Ok(String::from_utf8(body.to_vec()).map_err(BoxError::from)?)
    }

    /// Read the body and deserialise it as JSON into a `serde` enabled structure
    ///
    /// # Errors
    ///
    /// - Reading the body from the remote failed
    /// - Deserialising the body into the structure failed
    pub async fn json<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let bytes = self.bytes().await?;
        Ok(serde_json::from_slice(&bytes).map_err(BoxError::from)?)
    }

    /// Get the status of the request
    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    /// Stream the body
    pub fn stream(self) -> impl Stream<Item = Result<Bytes>> {
        BodyStream {
            inner: self.inner.into_body(),
        }
    }

    /// Get the HTTP version the client used
    #[must_use]
    pub fn version(&self) -> Version {
        self.inner.version()
    }
}
