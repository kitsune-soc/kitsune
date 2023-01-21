#![doc = include_str!("../README.md")]
#![forbid(rust_2018_idioms, unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

use http_body::{combinators::UnsyncBoxBody, Limited};
use hyper::{
    body::Bytes, client::HttpConnector, header::HeaderName, http::HeaderValue, service::Service,
    Body, Client as HyperClient, HeaderMap, Request, Response as HyperResponse, StatusCode,
    Version,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use kitsune_http_signatures::{HttpSigner, PrivateKey, SignatureComponent, SigningKey};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tower::{
    layer::util::Identity,
    util::{BoxService, Either},
    BoxError, ServiceBuilder, ServiceExt,
};
use tower_http::{
    decompression::DecompressionLayer, follow_redirect::FollowRedirectLayer,
    map_response_body::MapResponseBodyLayer, timeout::TimeoutLayer,
};

type Result<T, E = BoxError> = std::result::Result<T, E>;

/// Builder for the HTTP client
#[derive(Default)]
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
            key.try_into().map_err(Into::into)?,
            value.try_into().map_err(Into::into)?,
        );

        Ok(self)
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
            || Either::B(MapResponseBodyLayer::new(UnsyncBoxBody::new)),
            |limit| {
                Either::A(MapResponseBodyLayer::new(move |body| {
                    UnsyncBoxBody::new(Limited::new(body, limit))
                }))
            },
        );
        let timeout = self.timeout.map_or_else(
            || Either::B(Identity::new()),
            |duration| Either::A(TimeoutLayer::new(duration)),
        );

        Client {
            default_headers: self.default_headers,
            inner: BoxService::new(
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

/// An opinionated HTTP client
pub struct Client {
    default_headers: HeaderMap,
    inner: BoxService<Request<Body>, HyperResponse<UnsyncBoxBody<Bytes, BoxError>>, BoxError>,
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
    pub async fn execute(&mut self, req: Request<Body>) -> Result<Response> {
        let req = self.prepare_request(req);
        let response = self.inner.ready().await?.call(req).await?;

        Ok(Response { inner: response })
    }

    /// Sign an HTTP request via HTTP signatures and execute it
    pub async fn execute_signed<K>(
        &mut self,
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
            .await?;

        parts.headers.insert(name, value);
        let req = Request::from_parts(parts, body);

        self.execute(req).await
    }
}

/// HTTP response
pub struct Response {
    inner: HyperResponse<UnsyncBoxBody<Bytes, BoxError>>,
}

impl Response {
    /// Read the body into a `Bytes`
    pub async fn bytes(self) -> Result<Bytes> {
        hyper::body::to_bytes(self.inner).await
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
        serde_json::from_slice(&bytes).map_err(Into::into)
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
