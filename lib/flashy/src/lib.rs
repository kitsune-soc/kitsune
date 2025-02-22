use cookie::{Cookie, Expiration, SameSite};
use hex_simd::Out;
use http::HeaderValue;
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    pin::Pin,
    slice, str,
    sync::Mutex,
    task::{self, Poll, ready},
};
use subtle::ConstantTimeEq;
use tower::{Layer, Service};
use triomphe::Arc;
use zeroize::{Zeroize, ZeroizeOnDrop};

const COOKIE_NAME: &str = "FLASHY-FLASH_MESSAGES";
const KEY_CONTEXT: &str = "FLASHY-SIGN_COOKIE-BLAKE3-V1";

type Flash = (Level, String);

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Key([u8; blake3::KEY_LEN]);

impl Key {
    #[inline]
    #[must_use]
    pub fn new(inner: [u8; blake3::KEY_LEN]) -> Self {
        Self(inner)
    }

    #[inline]
    #[must_use]
    pub fn derive_from(data: &[u8]) -> Self {
        Self::new(blake3::derive_key(KEY_CONTEXT, data))
    }

    #[inline]
    #[must_use]
    pub fn generate() -> Self {
        Self::new(rand::random())
    }
}

impl Deref for Key {
    type Target = [u8; blake3::KEY_LEN];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Level {
    Debug,
    Info,
    Success,
    Warning,
    Error,
}

struct HandleInner {
    flashes: Vec<Flash>,
}

#[derive(Clone)]
pub struct FlashHandle(Arc<Mutex<HandleInner>>);

impl FlashHandle {
    #[inline]
    #[allow(clippy::needless_pass_by_value)]
    pub fn push(&self, level: Level, message: impl ToString) {
        let mut guard = self.0.lock().unwrap();
        guard.flashes.push((level, message.to_string()));
    }
}

pub struct FlashIter<'a>(slice::Iter<'a, (Level, String)>);

impl<'a> Iterator for FlashIter<'a> {
    type Item = (Level, &'a str);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (level, msg) = self.0.next()?;
        Some((*level, msg.as_str()))
    }
}

#[derive(Clone)]
pub struct IncomingFlashes(Arc<Vec<Flash>>);

impl IncomingFlashes {
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> FlashIter<'_> {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a IncomingFlashes {
    type IntoIter = FlashIter<'a>;
    type Item = (Level, &'a str);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        FlashIter(self.0.iter())
    }
}

#[derive(Clone)]
pub struct FlashService<S> {
    inner: S,
    key: Key,
}

impl<S> FlashService<S> {
    #[inline]
    #[must_use]
    pub fn new(inner: S, key: Key) -> Self {
        Self { inner, key }
    }
}

pin_project! {
    #[project = FlashFutureProj]
    pub enum FlashFuture<ResBody, Fut> {
        Execute {
            handle: FlashHandle,
            #[pin]
            fut: Fut,
            key: Option<Key>,
        },
        Return {
            handle: FlashHandle,
            resp: Option<http::Response<ResBody>>,
            key: Option<Key>,
        },
    }
}

impl<ResBody, Fut, Error> Future for FlashFuture<ResBody, Fut>
where
    Fut: Future<Output = Result<http::Response<ResBody>, Error>>,
{
    type Output = Result<http::Response<ResBody>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        loop {
            let this = match self.as_mut().project() {
                FlashFutureProj::Execute { handle, fut, key } => {
                    let resp = ready!(fut.poll(cx))?;
                    FlashFuture::Return {
                        handle: handle.clone(),
                        resp: Some(resp),
                        key: key.take(),
                    }
                }
                FlashFutureProj::Return { handle, resp, key } => {
                    let mut resp = resp.take().expect("missing response");
                    let key = key.take().expect("missing key");

                    let encoded_messages = {
                        let guard = handle.0.lock().unwrap();
                        let serialized =
                            sonic_rs::to_string(&guard.flashes).expect("failed to encode messages");

                        let signed = sign_data(&key, &serialized);
                        #[allow(unsafe_code)]
                        // SAFETY: the future returns correctly encoded UTF-8
                        let signed = unsafe { str::from_utf8_unchecked(&signed) };

                        format!("{signed}.{serialized}")
                    };

                    let cookie = Cookie::build((COOKIE_NAME, encoded_messages))
                        .same_site(SameSite::Strict)
                        .expires(Expiration::Session)
                        .build();

                    let encoded = cookie.encoded().to_string();
                    let value = HeaderValue::from_bytes(encoded.as_ref()).unwrap();

                    resp.headers_mut().append(http::header::SET_COOKIE, value);

                    return Poll::Ready(Ok(resp));
                }
            };

            self.set(this);
        }
    }
}

#[inline]
fn sign_data(key: &Key, value: &str) -> [u8; blake3::OUT_LEN * 2] {
    let hash = blake3::keyed_hash(key, value.as_bytes());

    let mut out = [0; blake3::OUT_LEN * 2];
    let enc_slice = hex_simd::encode(
        hash.as_bytes(),
        Out::from_slice(&mut out),
        hex_simd::AsciiCase::Lower,
    );
    assert_eq!(enc_slice.len(), out.len());

    out
}

#[inline]
fn verify_data(key: &Key, mac: &str, value: &str) -> bool {
    let mut out = [0; blake3::KEY_LEN];
    let Ok(decoded_mac) = hex_simd::decode(mac.as_ref(), Out::from_slice(&mut out)) else {
        return false;
    };

    blake3::keyed_hash(key, value.as_bytes())
        .as_bytes()
        .ct_eq(decoded_mac)
        .into()
}

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for FlashService<S>
where
    S: Service<http::Request<ReqBody>, Response = http::Response<ResBody>>,
{
    type Error = S::Error;
    type Response = http::Response<ResBody>;
    type Future = FlashFuture<ResBody, S::Future>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, mut req: http::Request<ReqBody>) -> Self::Future {
        let mut flash_cookie = None;
        'outer: for header in req.headers().get_all(http::header::COOKIE) {
            let Ok(cookie_str) = header.to_str() else {
                continue;
            };

            let cookies = Cookie::split_parse_encoded(cookie_str.to_string());
            for cookie in cookies {
                let Ok(cookie) = cookie else {
                    continue;
                };

                if cookie.name() == COOKIE_NAME {
                    flash_cookie = Some(cookie);
                    break 'outer;
                }
            }
        }

        let flashes = if let Some(flash_cookie) = flash_cookie {
            if let Some((mac, value)) = flash_cookie.value().split_once('.') {
                if verify_data(&self.key, mac, value) {
                    sonic_rs::from_str(value).unwrap()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let read_flashes = IncomingFlashes(Arc::new(flashes));
        let handle = FlashHandle(Arc::new(Mutex::new(HandleInner {
            flashes: Vec::new(),
        })));

        req.extensions_mut().insert(read_flashes);
        req.extensions_mut().insert(handle.clone());

        FlashFuture::Execute {
            handle,
            fut: self.inner.call(req),
            key: Some(self.key.clone()),
        }
    }
}

#[derive(Clone)]
pub struct FlashLayer {
    key: Key,
}

impl FlashLayer {
    #[inline]
    #[must_use]
    pub fn new(key: Key) -> Self {
        Self { key }
    }

    #[inline]
    #[must_use]
    pub fn generate() -> Self {
        Self::new(Key::generate())
    }
}

impl<S> Layer<S> for FlashLayer {
    type Service = FlashService<S>;

    #[inline]
    fn layer(&self, inner: S) -> Self::Service {
        FlashService::new(inner, self.key.clone())
    }
}

#[cfg(feature = "axum")]
mod axum_impl {
    use crate::{FlashHandle, IncomingFlashes};
    use axum_core::extract::FromRequestParts;
    use std::convert::Infallible;

    impl<S> FromRequestParts<S> for IncomingFlashes
    where
        S: Sync,
    {
        type Rejection = Infallible;

        #[inline]
        async fn from_request_parts(
            parts: &mut http::request::Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            let flashes = parts
                .extensions
                .get::<Self>()
                .expect("missing ReadFlashes. is FlashLayer mounted?")
                .clone();

            Ok(flashes)
        }
    }

    impl<S> FromRequestParts<S> for FlashHandle
    where
        S: Sync,
    {
        type Rejection = Infallible;

        #[inline]
        async fn from_request_parts(
            parts: &mut http::request::Parts,
            _state: &S,
        ) -> Result<Self, Self::Rejection> {
            let handle = parts
                .extensions
                .get::<Self>()
                .expect("missing FlashHandle. is FlashLayer mounted?")
                .clone();

            Ok(handle)
        }
    }
}
