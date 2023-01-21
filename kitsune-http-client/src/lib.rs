#![doc = include_str!("../README.md")]
#![forbid(rust_2018_idioms)]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

use http_body::{combinators::BoxBody, Limited};
use hyper::{
    body::Bytes,
    client::HttpConnector,
    header::{HeaderName, USER_AGENT},
    http::{self, HeaderValue},
    service::Service,
    Body, Client as HyperClient, HeaderMap, Request, Response as HyperResponse, StatusCode, Uri,
    Version,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use kitsune_http_signatures::{HttpSigner, PrivateKey, SignatureComponent, SigningKey};
use serde::de::DeserializeOwned;
use std::{
    error::Error as StdError,
    fmt,
    ops::{Deref, DerefMut},
    time::Duration,
};
use tower::{
    layer::util::Identity,
    util::{BoxCloneService, Either},
    BoxError, ServiceBuilder, ServiceExt,
};
use tower_http::{
    decompression::DecompressionLayer, follow_redirect::FollowRedirectLayer,
    map_response_body::MapResponseBodyLayer, timeout::TimeoutLayer,
};

type Result<T, E = Error> = std::result::Result<T, E>;

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
    /// This is enforced at the body level, regardless of whether the `Content-Type` header is set or not
    pub fn content_length_limit(self, content_length_limit: usize) -> Self {
        Self {
            content_length_limit: Some(content_length_limit),
            ..self
        }
    }

    /// Set a default header
    ///
    /// These headers are added to every HTTP request that is sent via this client
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
    pub fn timeout(self, timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
            ..self
        }
    }

    /// Build the HTTP client
    ///
    /// Yes, this operation is infallible
    pub fn build(self) -> Client {
        let connector = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
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

        unsafe {
            Client {
                default_headers: self.default_headers,
                inner: UnsafeSyncWrapper::new(BoxCloneService::new(
                    ServiceBuilder::new()
                        .layer(content_length_limit)
                        .layer(FollowRedirectLayer::new())
                        .layer(DecompressionLayer::default())
                        .layer(timeout)
                        .service(client),
                )),
            }
        }
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        let builder = ClientBuilder {
            content_length_limit: Option::default(),
            default_headers: HeaderMap::default(),
            timeout: Option::default(),
        };

        builder.user_agent("kitsune-http-client").unwrap()
    }
}

/// An opinionated HTTP client
#[derive(Clone)]
pub struct Client {
    default_headers: HeaderMap,
    inner: UnsafeSyncWrapper<
        BoxCloneService<Request<Body>, HyperResponse<BoxBody<Bytes, BoxError>>, BoxError>,
    >,
}

impl Client {
    /// Build a new client
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    fn prepare_request(&self, mut req: Request<Body>) -> Request<Body> {
        req.headers_mut()
            .extend(self.default_headers.clone().into_iter());
        req
    }

    /// Execute an HTTP request
    pub async fn execute(&self, req: Request<Body>) -> Result<Response> {
        let req = self.prepare_request(req);
        let response = self.inner.clone().ready().await?.call(req).await?;

        Ok(Response { inner: response })
    }

    /// Sign an HTTP request via HTTP signatures and execute it
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
        let http_signer = HttpSigner::builder().parts(&parts).build().unwrap();

        let (name, value) = http_signer
            .sign(
                private_key,
                vec![
                    SignatureComponent::RequestTarget,
                    SignatureComponent::Created,
                    SignatureComponent::Header("Date"),
                    SignatureComponent::Header("Digest"),
                ],
            )
            .await
            .map_err(BoxError::from)?;

        parts.headers.insert(name, value);
        let req = Request::from_parts(parts, body);

        self.execute(req).await
    }

    /// Shorthand for creating a GET request
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

/// HTTP response
pub struct Response {
    inner: HyperResponse<BoxBody<Bytes, BoxError>>,
}

impl Response {
    /// Read the body into a `Bytes`
    pub async fn bytes(self) -> Result<Bytes> {
        hyper::body::to_bytes(self.inner).await.map_err(Into::into)
    }

    /// Get a reference to the headers
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /// Read the body and deserialise it as JSON into a `serde` enabled structure
    pub async fn json<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let bytes = self.bytes().await?;
        Ok(serde_json::from_slice(&bytes).map_err(BoxError::from)?)
    }

    /// Get the status of the request
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    /// Get the HTTP version the client used
    pub fn version(&self) -> Version {
        self.inner.version()
    }
}

#[derive(Clone)]
struct UnsafeSyncWrapper<T> {
    inner: T,
}

impl<T> UnsafeSyncWrapper<T> {
    pub unsafe fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> Deref for UnsafeSyncWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for UnsafeSyncWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

unsafe impl<T> Sync for UnsafeSyncWrapper<T> {}
