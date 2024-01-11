use axum_core::{body::Body, RequestExt};
use bytes::{BufMut, BytesMut};
use http::{HeaderName, HeaderValue, Request};
use http_body::Body as HttpBody;
use pin_project_lite::pin_project;
use sha2::{Digest, Sha256, Sha512};
use std::{
    error::Error as StdError,
    future::Future,
    pin::Pin,
    str::FromStr,
    task::{self, ready, Poll},
};
use strum::{AsRefStr, EnumString};
use tower_layer::Layer;
use tower_service::Service;

type BoxError = Box<dyn StdError + Send + Sync>;

static DIGEST_HEADER_NAME: HeaderName = HeaderName::from_static("digest");

#[derive(AsRefStr, Clone, Copy, Default, EnumString)]
#[non_exhaustive]
pub enum Algorithm {
    #[default]
    #[strum(ascii_case_insensitive, serialize = "sha-256")]
    Sha256,

    #[strum(ascii_case_insensitive, serialize = "sha-512")]
    Sha512,
}

impl Algorithm {
    pub fn digest(&self, data: impl AsRef<[u8]>) -> Vec<u8> {
        match self {
            Self::Sha256 => Sha256::digest(data).to_vec(),
            Self::Sha512 => Sha512::digest(data).to_vec(),
        }
    }
}

pin_project! {
    #[project = DigestFutureProj]
    pub enum DigestFuture<S, F> {
        ParseHeader {
            service: Option<S>,

            algorithm: Algorithm,
            parts: Option<http::request::Parts>,
            body: Option<Body>,
        },
        BuildingDigest {
            service: S,

            algorithm: Algorithm,
            parts: Option<http::request::Parts>,

            #[pin]
            body: Body,
            body_accumulator: Option<BytesMut>,
        },
        PollServiceFuture {
            #[pin]
            future: F,
        },
    }
}

impl<S> Future for DigestFuture<S, S::Future>
where
    S: Service<Request<Body>>,
    S::Error: Into<BoxError>,
{
    type Output = Result<S::Response, BoxError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.as_mut().project() {
                DigestFutureProj::ParseHeader {
                    service,
                    algorithm,
                    parts,
                    body,
                } => {
                    let digest_header = parts.as_ref().unwrap().headers.get(&DIGEST_HEADER_NAME);
                    let algorithm = if let Some(digest_header) = digest_header {
                        let Some((algorithm_name, ..)) = digest_header.to_str()?.split_once('=')
                        else {
                            return Poll::Ready(Err("Invalid header value".into()));
                        };
                        Algorithm::from_str(algorithm_name)?
                    } else {
                        *algorithm
                    };

                    let new_state = DigestFuture::BuildingDigest {
                        service: service.take().unwrap(),
                        algorithm,
                        parts: parts.take(),
                        body: body.take().unwrap(),
                        body_accumulator: Some(BytesMut::new()),
                    };
                    self.set(new_state);
                }
                DigestFutureProj::BuildingDigest {
                    service,
                    algorithm,
                    parts,
                    mut body,
                    body_accumulator,
                } => {
                    while let Some(frame) = ready!(body.as_mut().poll_frame(cx))
                        .transpose()
                        .map_err(BoxError::from)?
                    {
                        if let Ok(data) = frame.into_data() {
                            let accumulator = body_accumulator
                                .as_mut()
                                .expect("[Bug] Missing accumulator");

                            accumulator.put(data);
                        }
                    }

                    let accumulator = body_accumulator
                        .take()
                        .expect("[Bug] Missing accumulator")
                        .freeze();

                    let hash = algorithm.digest(&accumulator);
                    let encoded_digest = base64_simd::STANDARD.encode_to_string(hash);

                    let header_value = format!("{}={}", algorithm.as_ref(), encoded_digest);
                    let header_value = HeaderValue::from_str(&header_value).unwrap();

                    let mut parts = parts.take().expect("[Bug] Missing parts");
                    parts.headers.insert(&DIGEST_HEADER_NAME, header_value);

                    let req = Request::from_parts(parts, accumulator.into());
                    let future = service.call(req);

                    self.set(DigestFuture::PollServiceFuture { future });
                }
                DigestFutureProj::PollServiceFuture { future } => {
                    return future.poll(cx).map_err(Into::into);
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct DigestService<S> {
    inner: S,
    algorithm: Algorithm,
}

impl<S> DigestService<S> {
    pub fn new(inner: S, algorithm: Algorithm) -> Self {
        Self { inner, algorithm }
    }
}

impl<S> Service<Request<Body>> for DigestService<S>
where
    S: Service<Request<Body>> + Clone,
    S::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = DigestFuture<S, S::Future>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let (parts, body) = req.with_limited_body().into_parts();

        DigestFuture::ParseHeader {
            service: Some(self.inner.clone()),
            algorithm: self.algorithm,
            parts: Some(parts),
            body: Some(body),
        }
    }
}

#[derive(Clone, Default)]
pub struct DigestLayer {
    algorithm: Algorithm,
}

impl DigestLayer {
    #[must_use]
    pub fn new(algorithm: Algorithm) -> Self {
        Self { algorithm }
    }
}

impl<S> Layer<S> for DigestLayer {
    type Service = DigestService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        DigestService::new(inner, self.algorithm)
    }
}
