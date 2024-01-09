use bytes::{BufMut, BytesMut};
use http::{HeaderName, HeaderValue, Request, Response};
use http_body::Body;
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
    #[strum(serialize = "sha-256", serialize = "id-sha-256")]
    Sha256,

    #[strum(serialize = "sha-512", serialize = "id-sha-512")]
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
    pub struct DigestFuture<S, B> {
        inner: S,

        algorithm: Algorithm,
        parts: Option<http::request::Parts>,
        #[pin]
        body: B,
        body_accumulator: BytesMut,
    }
}

impl<S, B> Future for DigestFuture<S, B>
where
    S: Service<Request<B>>,
    S::Error: Into<BoxError>,
    B: Body,
    B::Error: Into<BoxError>,
{
    type Output = Result<S::Response, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        while let Some(frame) = ready!(this.body.as_mut().poll_frame(cx))
            .transpose()
            .map_err(Into::into)?
        {
            if let Ok(data) = frame.into_data() {
                this.body_accumulator.put(data);
            }
        }

        let digest_header = this.parts.as_ref().unwrap().headers.get(DIGEST_HEADER_NAME);

        let digest_header_value = if let Some(digest_header) = digest_header {
            let (algorithm_name, _) = digest_header.to_str()?.split_once('=');
            let algorithm = Algorithm::from_str(algorithm_name)?;
        } else {
            let hash = this.algorithm.digest(this.body_accumulator);
            let encoded_digest = base64_simd::STANDARD.encode_to_string(hash);
            let header_value = format!("{}={}", this.algorithm.as_ref(), encoded_digest);

            HeaderValue::from_str(&header_value).unwrap()
        };

        let req = Request::from_parts(this.parts.take().expect("[Bug] Missing parts"), todo!());
        this.inner.call(req)
    }
}

#[derive(Clone)]
pub struct DigestService<S> {
    inner: S,
}

impl<S> DigestService<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, B> Service<Request<B>> for DigestService<S>
where
    S: Service<Request<B>>,
    S::Error: Into<BoxError>,
    B: Body,
    B::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = DigestFuture<S, B>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        todo!();
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
        DigestService::new(inner)
    }
}
