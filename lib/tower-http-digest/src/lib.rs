use futures_util::{
    future::{Either, MapErr},
    TryFutureExt,
};
use http::{HeaderName, Request};
use http_body::{Body as HttpBody, Frame};
use pin_project_lite::pin_project;
use sha2::{digest::FixedOutput, Digest, Sha256, Sha512};
use std::{
    error::Error as StdError,
    future::{self, Ready},
    pin::Pin,
    str::FromStr,
    task::{self, ready, Poll},
};
use strum::{AsRefStr, EnumString};
use subtle::ConstantTimeEq;
use tower_layer::Layer;
use tower_service::Service;

type BoxError = Box<dyn StdError + Send + Sync>;

static DIGEST_HEADER_NAME: HeaderName = HeaderName::from_static("digest");

#[derive(AsRefStr, Clone, EnumString)]
#[non_exhaustive]
pub enum Algorithm {
    #[strum(ascii_case_insensitive, serialize = "sha-256")]
    Sha256(Sha256),

    #[strum(ascii_case_insensitive, serialize = "sha-512")]
    Sha512(Sha512),
}

impl Algorithm {
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

impl Default for Algorithm {
    fn default() -> Self {
        Self::Sha256(Sha256::default())
    }
}

pin_project! {
    pub struct VerifyDigestBody<B> {
        #[pin]
        inner: B,
        algorithm: Option<Algorithm>,
        digest_value: String,
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
        let frame = ready!(this.inner.poll_frame(cx))
            .transpose()
            .map_err(Into::into)?;

        if let Some(frame) = frame.as_ref().and_then(Frame::data_ref) {
            this.algorithm
                .as_mut()
                .expect("[Bug] Missing algorithm")
                .update(frame.as_ref());
        }

        if frame.is_some() {
            return Poll::Ready(frame.map(Ok));
        } else if let Some(algorithm) = this.algorithm.take() {
            let calculated_digest = algorithm.finish();
            let decoded_digest = base64_simd::STANDARD.decode_to_vec(this.digest_value)?;

            if calculated_digest.ct_ne(&decoded_digest).into() {
                return Poll::Ready(Some(Err("Digest mismatch".into())));
            }
        }

        Poll::Ready(None)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
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
        let digest_header = req.headers().get(&DIGEST_HEADER_NAME);

        let (algorithm, digest_value) = if let Some(digest_header) = digest_header {
            let digest_header_str = match digest_header.to_str() {
                Ok(str) => str,
                Err(err) => return Either::Right(future::ready(Err(err.into()))),
            };

            let Some((algorithm_name, digest_value)) = digest_header_str.split_once('=') else {
                return Either::Right(future::ready(Err("Invalid header value".into())));
            };

            match Algorithm::from_str(algorithm_name) {
                Ok(alg) => (alg, digest_value.to_string()),
                Err(err) => return Either::Right(future::ready(Err(err.into()))),
            }
        } else {
            return Either::Right(future::ready(Err("Missing digest header".into())));
        };

        Either::Left(
            self.inner
                .call(req.map(|inner| VerifyDigestBody {
                    inner,
                    algorithm: Some(algorithm),
                    digest_value,
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
