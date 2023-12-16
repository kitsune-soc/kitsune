#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use self::util::BoxCloneService;
use async_stream::try_stream;
use bytes::Buf;
use futures_core::Stream;
use headers::{Date, HeaderMapExt};
use http_body::Body as HttpBody;
use http_body_util::{combinators::BoxBody, BodyExt, BodyStream, Empty, Limited};
use hyper::{
    body::Bytes,
    header::{HeaderName, USER_AGENT},
    http::{self, HeaderValue},
    HeaderMap, Request, Response as HyperResponse, StatusCode, Uri, Version,
};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::{client::legacy::Client as HyperClient, rt::TokioExecutor};
use kitsune_http_signatures::{HttpSigner, PrivateKey, SignatureComponent, SigningKey};
use kitsune_type::jsonld::RdfNode;
use serde::de::DeserializeOwned;
use std::{
    error::Error as StdError,
    fmt,
    time::{Duration, SystemTime},
};
use tower::{layer::util::Identity, util::Either, BoxError, Service, ServiceBuilder, ServiceExt};
use tower_http::{
    decompression::DecompressionLayer,
    follow_redirect::{FollowRedirectLayer, RequestUri},
    map_response_body::MapResponseBodyLayer,
    timeout::TimeoutLayer,
};

mod util;

type Body = BoxBody<Bytes, BoxError>;
type Result<T, E = Error> = std::result::Result<T, E>;

/// Default body limit of 1MB
const DEFAULT_BODY_LIMIT: usize = 1024 * 1024;

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
            .expect("Failed to fetch native certificates")
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();

        let client = HyperClient::builder(TokioExecutor::new())
            .build(connector)
            .map_response(|resp| {
                let (parts, body) = resp.into_parts();
                let body = BoxBody::new(body);
                HyperResponse::from_parts(parts, body)
            });

        self.service(client)
    }

    /// Build the HTTP client by wrapping another HTTP client service
    #[must_use]
    pub fn service<S, B>(self, client: S) -> Client
    where
        S: Service<Request<Body>, Response = HyperResponse<B>> + Clone + Send + Sync + 'static,
        S::Error: StdError + Send + Sync + 'static,
        S::Future: Send,
        B: HttpBody + Default + Send + Sync + 'static,
        B::Data: Send + Sync,
        B::Error: StdError + Send + Sync + 'static,
    {
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
        req.headers_mut().extend(self.default_headers.clone());
        req.headers_mut()
            .typed_insert(Date::from(SystemTime::now()));

        req
    }

    /// Execute an HTTP request
    ///
    /// # Errors
    ///
    /// - The inner client service isn't ready
    /// - The request failed
    pub async fn execute<B>(&self, req: Request<B>) -> Result<Response>
    where
        B: HttpBody<Data = Bytes> + Send + Sync + 'static,
        B::Error: Into<BoxError> + 'static,
    {
        let (parts, body) = req.into_parts();
        let body = BoxBody::new(body.map_err(Into::into));
        let req = Request::from_parts(parts, body);

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
    pub async fn execute_signed<B, K>(
        &self,
        req: Request<B>,
        private_key: PrivateKey<'_, K>,
    ) -> Result<Response>
    where
        B: HttpBody<Data = Bytes> + Send + Sync + 'static,
        B::Error: Into<BoxError> + 'static,
        K: SigningKey + Send + 'static,
    {
        let (parts, body) = req.into_parts();
        let body = BoxBody::new(body.map_err(Into::into));
        let req = Request::from_parts(parts, body);

        let req = self.prepare_request(req);
        let (mut parts, body) = req.into_parts();

        let (name, value) = HttpSigner::builder()
            .include_creation_timestamp(true)
            .expires_in(Duration::from_secs(30)) // Make the signature expire in 30 seconds
            .build()
            .sign(
                &parts,
                vec![
                    SignatureComponent::RequestTarget,
                    SignatureComponent::Header("Date"),
                    SignatureComponent::Header("Digest"),
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
            .body(Body::new(Empty::new().map_err(Into::into)))
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
#[derive(Debug)]
pub struct Response {
    inner: HyperResponse<BoxBody<Bytes, BoxError>>,
}

impl Response {
    /// Convert the response into its inner `hyper` representation
    #[must_use]
    pub fn into_inner(self) -> HyperResponse<BoxBody<Bytes, BoxError>> {
        self.inner
    }

    /// Read the body into a `Bytes`
    ///
    /// # Errors
    ///
    /// Reading the body from the remote failed
    pub async fn bytes(self) -> Result<Bytes> {
        Ok(self.inner.collect().await?.to_bytes())
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
        Ok(simd_json::from_reader(bytes.reader()).map_err(BoxError::from)?)
    }

    /// Read the body and deserialise it as JSON-LD node and verify the returned node's `@id`
    ///
    /// # Errors
    ///
    /// - Reading the body from the remote failed
    /// - Deserialising the body into the structure failed
    /// - The authority part of the returned JSON-LD node's `@id` doesn't belong to the originating server
    pub async fn jsonld<T>(mut self) -> Result<T>
    where
        T: DeserializeOwned + RdfNode,
    {
        let Some(server_authority) = self
            .inner
            .extensions_mut()
            .remove()
            .and_then(|RequestUri(uri)| uri.authority().cloned())
        else {
            // This only happens if the `FollowRedirect` middleware neglect to insert the extension
            // or the URI doesn't contain the authority part, which won't occur in the current
            // version of `tower-http`
            return Err(BoxError::from("Failed to get the server authority").into());
        };

        let node: T = self.json().await?;
        if let Some(id) = node.id() {
            if Uri::try_from(id)
                .map_err(BoxError::from)?
                .authority()
                .map_or(true, |node_authority| *node_authority != server_authority)
            {
                return Err(BoxError::from(
                    "Authority of `@id` doesn't belong to the originating server",
                )
                .into());
            }
        }

        Ok(node)
    }

    /// Get the status of the request
    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    /// Stream the body
    pub fn stream(self) -> impl Stream<Item = Result<Bytes>> {
        try_stream! {
            let body_stream = BodyStream::new(self.inner.into_body());

            for await frame in body_stream {
                match frame?.into_data() {
                    Ok(val) if val.has_remaining() => yield val,
                    _ => (),
                }
            }
        }
    }

    /// Get the HTTP version the client used
    #[must_use]
    pub fn version(&self) -> Version {
        self.inner.version()
    }
}
