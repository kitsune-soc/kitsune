#![doc = include_str!("../README.md")]

#[macro_use]
extern crate tracing;

use cookie::{Cookie, SameSite};
use hex_simd::{AsOut, AsciiCase};
use http::{header, HeaderValue, Request, Response};
use pin_project_lite::pin_project;
use rand::{distributions::Alphanumeric, Rng};
use std::{
    fmt::Display,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{self, ready, Poll},
};
use tower::{Layer, Service};
use zeroize::{Zeroize, ZeroizeOnDrop};

const CSRF_COOKIE_NAME: &str = "CSRF_TOKEN";
const RANDOM_DATA_LEN: usize = 32;

#[aliri_braid::braid]
pub struct Hash;

#[aliri_braid::braid]
pub struct Message;

#[derive(Clone)]
struct CsrfData {
    hash: Hash,
    message: Message,
}

struct Shared {
    read_data: Option<CsrfData>,
    set_data: Option<CsrfData>,
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct CsrfHandle {
    #[zeroize(skip)]
    inner: Arc<Mutex<Shared>>,
    key: [u8; blake3::KEY_LEN],
}

fn raw_verify(key: &[u8; blake3::KEY_LEN], hash: &HashRef, message: &MessageRef) -> bool {
    let (hash, message) = (hash.as_ref(), message.as_ref());
    if hash.len() / 2 != blake3::OUT_LEN {
        return false;
    }

    let mut decoded_hash = [0_u8; blake3::OUT_LEN];
    if hex_simd::decode(hash.as_bytes(), decoded_hash.as_mut().as_out()).is_err() {
        return false;
    }

    let expected_hash = blake3::keyed_hash(key, message.as_bytes());

    // The `PartialEq` implementation on `Hash` is constant-time
    expected_hash == decoded_hash
}

impl CsrfHandle {
    /// Keep the current signature and message inside the cookie
    #[inline]
    pub fn keep_cookie(&self) {
        let mut guard = self.inner.lock().unwrap();
        guard.set_data = guard.read_data.clone();
    }

    /// Create a signature and store it inside a cookie
    ///
    /// **Important**: The data passed into this function should reference an *authenticated session*.
    /// The use of the user ID (or something similarly static) is *discouraged*, use the session ID.
    #[inline]
    pub fn sign<SID>(&self, session_id: SID) -> Message
    where
        SID: AsRef<[u8]> + Display,
    {
        let random = rand::thread_rng()
            .sample_iter(Alphanumeric)
            .map(char::from)
            .take(RANDOM_DATA_LEN)
            .collect::<String>();

        let message = format!("{session_id}!{random}");
        let hash = blake3::keyed_hash(&self.key, message.as_bytes());
        let hash = hex_simd::encode_to_string(hash.as_bytes(), AsciiCase::Lower);

        let message: Message = message.into();
        self.inner.lock().unwrap().set_data = Some(CsrfData {
            hash: hash.into(),
            message: message.clone(),
        });

        message
    }

    /// Verify the CSRF request
    ///
    /// Simply pass in the message that was submitted by the client.
    /// Internally, we will compare this to the
    #[inline]
    #[must_use]
    pub fn verify(&self, message: &MessageRef) -> bool {
        let guard = self.inner.lock().unwrap();
        let Some(ref read_data) = guard.read_data else {
            return false;
        };

        raw_verify(&self.key, &read_data.hash, &read_data.message)
            && raw_verify(&self.key, &read_data.hash, message)
    }
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct CsrfLayer {
    key: [u8; blake3::KEY_LEN],
}

impl CsrfLayer {
    #[must_use]
    pub fn new(key: [u8; blake3::KEY_LEN]) -> Self {
        Self { key }
    }
}

impl<S> Layer<S> for CsrfLayer {
    type Service = CsrfService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CsrfService::new(inner, self.key)
    }
}

pin_project! {
    pub struct ResponseFuture<F> {
        #[pin]
        inner: F,
        handle: CsrfHandle,
    }
}

impl<F, E, ResBody> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let mut response = ready!(this.inner.poll(cx))?;
        let mut cookie = Cookie::build(CSRF_COOKIE_NAME)
            .permanent()
            .same_site(SameSite::Strict)
            .secure(true)
            .build();

        let guard = this.handle.inner.lock().unwrap();
        if let Some(ref set_data) = guard.set_data {
            let value = format!("{}.{}", set_data.hash, set_data.message);
            cookie.set_value(value);
        } else {
            cookie.make_removal();
        }

        let encoded_cookie = cookie.encoded().to_string();
        let header_value = HeaderValue::from_str(&encoded_cookie).unwrap();

        response
            .headers_mut()
            .append(header::SET_COOKIE, header_value);

        Poll::Ready(Ok(response))
    }
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct CsrfService<S> {
    #[zeroize(skip)]
    inner: S,
    key: [u8; blake3::KEY_LEN],
}

impl<S> CsrfService<S> {
    pub fn new(inner: S, key: [u8; blake3::KEY_LEN]) -> Self {
        Self { inner, key }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for CsrfService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;
    type Response = S::Response;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let cookies = req
            .headers()
            .get_all(header::COOKIE)
            .into_iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(Cookie::split_parse_encoded);

        let mut csrf_cookie = None;
        for cookie in cookies {
            let cookie = match cookie {
                Ok(cookie) => cookie,
                Err(error) => {
                    debug!(?error, "failed to decode cookie");
                    continue;
                }
            };

            if cookie.name() == CSRF_COOKIE_NAME {
                csrf_cookie = Some(cookie);
                break;
            }
        }

        let read_data = if let Some(csrf_cookie) = csrf_cookie {
            csrf_cookie
                .value_trimmed()
                .split_once('.')
                .map(|(hash, message)| CsrfData {
                    hash: hash.into(),
                    message: message.into(),
                })
        } else {
            None
        };

        let handle = CsrfHandle {
            inner: Arc::new(Mutex::new(Shared {
                read_data,
                set_data: None,
            })),
            key: self.key,
        };

        req.extensions_mut().insert(handle.clone());

        ResponseFuture {
            inner: self.inner.call(req),
            handle,
        }
    }
}

#[cfg(feature = "axum")]
mod axum_impl {
    use crate::CsrfHandle;
    use async_trait::async_trait;
    use axum_core::extract::FromRequestParts;
    use http::request::Parts;
    use std::convert::Infallible;

    #[async_trait]
    impl<S> FromRequestParts<S> for CsrfHandle {
        type Rejection = Infallible;

        async fn from_request_parts(
            parts: &mut Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            let handle = parts
                .extensions
                .get::<Self>()
                .expect("Service not wrapped by CSRF middleware")
                .clone();

            Ok(handle)
        }
    }
}
