#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use self::{resolver::Resolver, util::BoxCloneService};
use bytes::Buf;
use futures_util::{Stream, StreamExt};
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use http::HeaderValue;
use http_body::Body as HttpBody;
use http_body_util::{BodyExt, BodyStream, Limited};
use hyper::{
    body::Bytes,
    header::{HeaderName, USER_AGENT},
    HeaderMap, Request, Response as HyperResponse, StatusCode, Uri, Version,
};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client as HyperClient},
    rt::TokioExecutor,
};
use kitsune_type::jsonld::RdfNode;
use serde::de::DeserializeOwned;
use std::{error::Error as StdError, fmt, time::Duration};
use tower::{layer::util::Identity, util::Either, BoxError, Service, ServiceBuilder, ServiceExt};
use tower_http::{
    decompression::DecompressionLayer,
    follow_redirect::{FollowRedirectLayer, RequestUri},
    map_response_body::MapResponseBodyLayer,
    timeout::TimeoutLayer,
};

mod body;
mod resolver;
mod util;

type BoxBody<E = BoxError> = http_body_util::combinators::BoxBody<Bytes, E>;
type Result<T, E = Error> = std::result::Result<T, E>;

/// Default body limit of 1MB
const DEFAULT_BODY_LIMIT: usize = 1024 * 1024;

/// Default request timeout of 30s (same as Firefox)
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Alias for our internal HTTP body type
pub use self::body::Body;

/// Response body type
pub type ResponseBody = BoxBody;

/// Client error type
pub struct Error {
    inner: BoxError,
}

impl Error {
    #[inline]
    fn new<E>(inner: E) -> Self
    where
        E: Into<BoxError>,
    {
        Self {
            inner: inner.into(),
        }
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
    dns_resolver: Option<Resolver>,
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
            value.try_into().map_err(Error::new)?,
        );

        Ok(self)
    }

    /// Set a hickory DNS resolver you want this client to use
    ///
    /// Otherwise it creates a new one which connects to Quad9 via DNS-over-TLS
    #[must_use]
    pub fn dns_resolver(mut self, resolver: impl Into<Resolver>) -> Self {
        self.dns_resolver = Some(resolver.into());
        self
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
    pub fn build(mut self) -> Client {
        let resolver = self.dns_resolver.take().unwrap_or_else(|| {
            hickory_resolver::TokioResolver::tokio(
                ResolverConfig::quad9_tls(),
                ResolverOpts::default(),
            )
            .into()
        });

        let connector = HttpsConnectorBuilder::new()
            .with_native_roots()
            .expect("Failed to fetch native certificates")
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .wrap_connector(HttpConnector::new_with_resolver(resolver));

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
            || Either::Left(MapResponseBodyLayer::new(BoxBody::new)),
            |limit| {
                Either::Right(MapResponseBodyLayer::new(move |body| {
                    BoxBody::new(Limited::new(body, limit))
                }))
            },
        );
        let timeout = self.timeout.map_or_else(
            || Either::Left(Identity::new()),
            |duration| Either::Right(TimeoutLayer::new(duration)),
        );

        Client {
            default_headers: self.default_headers,
            inner: BoxCloneService::new(
                ServiceBuilder::new()
                    .layer(content_length_limit)
                    .layer(FollowRedirectLayer::new())
                    .layer(DecompressionLayer::default())
                    .layer(timeout)
                    .service(client)
                    .map_err(BoxError::from),
            ),
        }
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        let builder = ClientBuilder {
            content_length_limit: Some(DEFAULT_BODY_LIMIT),
            default_headers: HeaderMap::default(),
            dns_resolver: None,
            timeout: Some(DEFAULT_REQUEST_TIMEOUT),
        };

        builder
            .user_agent(kitsune_core::consts::USER_AGENT)
            .unwrap()
    }
}

#[derive(Clone)]
/// An opinionated HTTP client
pub struct Client {
    default_headers: HeaderMap,
    inner: BoxCloneService<Request<Body>, HyperResponse<BoxBody>, BoxError>,
}

impl Client {
    /// Build a new client
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    fn prepare_request(&self, mut req: Request<Body>) -> Request<Body> {
        req.headers_mut().extend(self.default_headers.clone());
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

        let ready_svc = self.inner.clone();
        let response = ready_svc.oneshot(req).await.map_err(Error::new)?;

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
    pub async fn execute_signed(
        &self,
        req: Request<Body>,
        key_id: &str,
        private_key_pem: &str,
    ) -> Result<Response> {
        let req =
            http_signatures::cavage::easy::sign(self.prepare_request(req), key_id, private_key_pem)
                .await
                .map_err(Error::new)?;

        self.execute(req).await
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
            .map_err(Error::new)?;

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
    inner: HyperResponse<BoxBody>,
}

impl Response {
    /// Convert the response into its inner `hyper` representation
    #[must_use]
    pub fn into_inner(self) -> HyperResponse<BoxBody> {
        self.inner
    }

    /// Read the body into a `Bytes`
    ///
    /// # Errors
    ///
    /// Reading the body from the remote failed
    pub async fn bytes(self) -> Result<Bytes> {
        Ok(self.inner.collect().await.map_err(Error::new)?.to_bytes())
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
        // `.to_owned()` as the same performance overhead as calling `.to_vec()` on the `Bytes` body.
        // Therefore we can circumvent unsafe usage here by simply calling `.to_owned()` on the string slice at no extra cost.
        simdutf8::basic::from_utf8(&body)
            .map(ToOwned::to_owned)
            .map_err(Error::new)
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
        sonic_rs::from_slice(&bytes).map_err(Error::new)
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
            return Err(Error::new(BoxError::from(
                "Failed to get the server authority",
            )));
        };

        let node: T = self.json().await?;
        if let Some(id) = node.id() {
            if Uri::try_from(id)
                .map_err(Error::new)?
                .authority()
                .is_none_or(|node_authority| *node_authority != server_authority)
            {
                return Err(Error::new(BoxError::from(
                    "Authority of `@id` doesn't belong to the originating server",
                )));
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
        let mut body_stream = BodyStream::new(self.inner.into_body());

        asynk_strim::try_stream_fn(|mut yielder| async move {
            while let Some(frame) = body_stream.next().await {
                match frame.map_err(Error::new)?.into_data() {
                    Ok(val) if val.has_remaining() => yielder.yield_ok(val).await,
                    Ok(..) | Err(..) => {
                        // There was either no remaining data or the frame was no data frame.
                        // Therefore we just discard it.
                    }
                }
            }

            Ok(())
        })
    }

    /// Get the HTTP version the client used
    #[must_use]
    pub fn version(&self) -> Version {
        self.inner.version()
    }
}
