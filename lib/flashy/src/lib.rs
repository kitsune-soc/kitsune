use cookie::{Cookie, CookieJar, Expiration, Key, SameSite};
use http::HeaderValue;
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    pin::Pin,
    sync::Mutex,
    task::{self, ready, Poll},
};
use tower::Service;
use triomphe::Arc;

const COOKIE_NAME: &str = "FLASH_MESSAGES";

type Flash = (Level, String);

#[derive(Clone, Copy, Deserialize, Serialize)]
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
    pub fn push(&self, level: Level, message: String) {
        let mut guard = self.0.lock().unwrap();
        guard.flashes.push((level, message));
    }
}

#[derive(Clone)]
pub struct ReadFlashes(Arc<Vec<Flash>>);

impl ReadFlashes {
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
    pub fn iter(&self) -> impl Iterator<Item = (Level, &str)> {
        self.0
            .iter()
            .map(|(level, message)| (*level, message.as_str()))
    }
}

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
            let this = {
                match self.as_mut().project() {
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
                            sonic_rs::to_string(&guard.flashes).expect("failed to encode messages")
                        };

                        let mut cookie = Cookie::new(COOKIE_NAME, encoded_messages);
                        cookie.set_same_site(SameSite::Strict);
                        cookie.set_secure(true);
                        cookie.set_expires(Expiration::Session);

                        let mut jar = CookieJar::new();
                        let mut signed_jar = jar.signed_mut(&key);
                        signed_jar.add(cookie);

                        for cookie in jar.iter() {
                            let encoded = cookie.encoded().to_string();
                            let value = HeaderValue::from_bytes(encoded.as_ref()).unwrap();

                            resp.headers_mut().insert(http::header::SET_COOKIE, value);
                        }

                        return Poll::Ready(Ok(resp));
                    }
                }
            };

            self.set(this);
        }
    }
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
        let mut jar = CookieJar::new();
        for header in req.headers().get_all(http::header::COOKIE) {
            let Ok(cookie_str) = header.to_str() else {
                continue;
            };

            let cookies = Cookie::split_parse_encoded(cookie_str.to_string());
            for cookie in cookies {
                let Ok(cookie) = cookie else {
                    continue;
                };

                jar.add_original(cookie);
            }
        }

        let signed_jar = jar.signed(&self.key);
        let flashes = signed_jar
            .get(COOKIE_NAME)
            .and_then(|cookie| sonic_rs::from_str(cookie.value()).ok())
            .unwrap_or_default();

        let read_flashes = ReadFlashes(Arc::new(flashes));
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
