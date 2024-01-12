use futures_util::{
    future::{Either, MapErr},
    TryFutureExt,
};
use http::{HeaderName, HeaderValue, Request};
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

struct Verifier {
    algorithm: Algorithm,
    digest_value: Vec<u8>,
}

impl Verifier {
    pub fn from_header_value(header_value: &HeaderValue) -> Result<Self, BoxError> {
        let Some(pos) = memchr(b'=', header_value.as_bytes()) else {
            return Err("Invalid header value".into());
        };

        let (algorithm_name, digest_value) = header_value.as_bytes().split_at(pos);
        let algorithm = Algorithm::from_bytes(algorithm_name)
            .ok_or_else(|| BoxError::from("Unsupported digest"))?;

        let digest_value = base64_simd::STANDARD.decode_to_vec(&digest_value[1..])?;

        Ok(Self {
            algorithm,
            digest_value,
        })
    }

    pub fn update_digest(&mut self, val: &[u8]) {
        self.algorithm.update(val);
    }

    pub fn verify(self) -> bool {
        self.algorithm.finish().ct_eq(&self.digest_value).into()
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
    pub fn finish(self) -> Vec<u8> {
        match self {
            Self::Sha256(digest) => digest.finalize_fixed().to_vec(),
            Self::Sha512(digest) => digest.finalize_fixed().to_vec(),
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

impl<S, B> Service<Request<B>> for VerifyDigestService<S>
where
    S: Service<Request<VerifyDigestBody<B>>>,
    S::Error: Into<BoxError>,
    B: HttpBody,
    B::Data: AsRef<[u8]>,
    B::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future =
        Either<MapErr<S::Future, fn(S::Error) -> BoxError>, Ready<Result<S::Response, BoxError>>>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let Some(digest_header) = req.headers().get(&DIGEST_HEADER_NAME) else {
            return Either::Right(future::ready(Err("Missing digest header".into())));
        };
        let verifier = match Verifier::from_header_value(digest_header) {
            Ok(verifier) => verifier,
            Err(err) => return Either::Right(future::ready(Err(err))),
        };

        Either::Left(
            self.inner
                .call(req.map(|inner| VerifyDigestBody {
                    inner,
                    verifier: Some(verifier),
                }))
                .map_err(Into::into),
        )
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
