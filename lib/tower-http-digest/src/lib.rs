#[macro_use]
extern crate tracing;

use bytes::Bytes;
use either::Either;
use http::{HeaderName, HeaderValue, Request, Response, StatusCode};
use http_body::{Body as HttpBody, Frame};
use memchr::memchr;
use pin_project_lite::pin_project;
use sha2::{digest::FixedOutput, Digest, Sha256, Sha512};
use std::{
    error::Error as StdError,
    future::{self, Ready},
    pin::Pin,
    task::{self, ready, Poll},
};
use subtle::ConstantTimeEq;
use tower_layer::Layer;
use tower_service::Service;

type BoxError = Box<dyn StdError + Send + Sync>;

static DIGEST_HEADER_NAME: HeaderName = HeaderName::from_static("digest");

static MISSING_DIGEST_HEADER_BODY: Bytes = Bytes::from_static(b"Missing digest header");
static UNSUPPORTED_DIGEST_BODY: Bytes = Bytes::from_static(b"Unsupported digest");

fn handle_single(bytes: &[u8]) -> Result<Option<Verifier>, BoxError> {
    let Some(pos) = memchr(b'=', bytes) else {
        return Err("Invalid header value".into());
    };

    let (algorithm_name, digest_value) = bytes.split_at(pos);
    let Some(algorithm) = Algorithm::from_bytes(algorithm_name) else {
        return Ok(None);
    };

    let digest_value = base64_simd::STANDARD.decode_to_vec(&digest_value[1..])?;

    Ok(Some(Verifier {
        algorithm,
        digest_value,
    }))
}

fn handle_multiple(mut bytes: &[u8]) -> Result<Option<Verifier>, BoxError> {
    while let Some(split_pos) = memchr(b',', bytes) {
        let (algo, rest) = bytes.split_at(split_pos);

        if let Some(verifier) = handle_single(algo)? {
            return Ok(Some(verifier));
        }

        bytes = &rest[1..];
    }

    // And run one last time over the remaining bytes
    handle_single(bytes)
}

struct Verifier {
    algorithm: Algorithm,
    digest_value: Vec<u8>,
}

impl Verifier {
    pub fn from_header_value(header_value: &HeaderValue) -> Result<Self, BoxError> {
        let header_bytes = header_value.as_bytes();

        if memchr(b',', header_bytes).is_some() {
            handle_multiple(header_bytes)
        } else {
            handle_single(header_bytes)
        }
        .transpose()
        .ok_or_else(|| BoxError::from("No compatible digest found"))?
    }

    pub fn update_digest(&mut self, val: &[u8]) {
        self.algorithm.update(val);
    }

    pub fn verify(self) -> bool {
        self.algorithm
            .finish()
            .as_ref()
            .ct_eq(&self.digest_value)
            .into()
    }
}

#[derive(Clone)]
#[non_exhaustive]
enum Algorithm {
    Sha256(Sha256),
    Sha512(Sha512),
}

impl Algorithm {
    pub fn from_bytes(val: &[u8]) -> Option<Self> {
        let algorithm = if b"sha-256".eq_ignore_ascii_case(val) {
            Self::Sha256(Sha256::default())
        } else if b"sha-512".eq_ignore_ascii_case(val) {
            Self::Sha512(Sha512::default())
        } else {
            return None;
        };

        Some(algorithm)
    }

    pub fn update(&mut self, data: &[u8]) {
        match self {
            Self::Sha256(digest) => digest.update(data),
            Self::Sha512(digest) => digest.update(data),
        }
    }

    #[must_use]
    pub fn finish(self) -> impl AsRef<[u8]> {
        match self {
            Self::Sha256(digest) => Either::Left(digest.finalize_fixed()),
            Self::Sha512(digest) => Either::Right(digest.finalize_fixed()),
        }
    }
}

pin_project! {
    pub struct VerifyDigestBody<B> {
        #[pin]
        inner: B,
        verifier: Option<Verifier>,
    }
}

impl<B> HttpBody for VerifyDigestBody<B>
where
    B: HttpBody,
    B::Data: AsRef<[u8]>,
    B::Error: Into<BoxError>,
{
    type Data = B::Data;
    type Error = BoxError;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        let Some(frame) = ready!(this.inner.poll_frame(cx))
            .transpose()
            .map_err(Into::into)?
        else {
            if let Some(verifier) = this.verifier.take() {
                if !verifier.verify() {
                    return Poll::Ready(Some(Err("Digest mismatch".into())));
                }
            }

            return Poll::Ready(None);
        };

        if let Some(frame) = frame.data_ref() {
            this.verifier
                .as_mut()
                .expect("[Bug] Missing verifier")
                .update_digest(frame.as_ref());
        }

        Poll::Ready(Some(Ok(frame)))
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}

#[derive(Clone)]
pub struct VerifyDigestService<S> {
    inner: S,
}

impl<S> VerifyDigestService<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for VerifyDigestService<S>
where
    S: Service<Request<VerifyDigestBody<ReqBody>>, Response = Response<ResBody>>,
    ResBody: From<Bytes>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Either<S::Future, Ready<Result<S::Response, S::Error>>>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let Some(digest_header) = req.headers().get(&DIGEST_HEADER_NAME) else {
            debug!("Missing digest header");
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(MISSING_DIGEST_HEADER_BODY.clone().into())
                .unwrap();

            return Either::Right(future::ready(Ok(response)));
        };

        let verifier = match Verifier::from_header_value(digest_header) {
            Ok(verifier) => verifier,
            Err(error) => {
                debug!(?error, "Unsupported digest");
                let response = Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(UNSUPPORTED_DIGEST_BODY.clone().into())
                    .unwrap();

                return Either::Right(future::ready(Ok(response)));
            }
        };

        Either::Left(self.inner.call(req.map(|inner| VerifyDigestBody {
            inner,
            verifier: Some(verifier),
        })))
    }
}

#[derive(Clone, Default)]
pub struct VerifyDigestLayer {
    _priv: (),
}

impl<S> Layer<S> for VerifyDigestLayer {
    type Service = VerifyDigestService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        VerifyDigestService::new(inner)
    }
}
