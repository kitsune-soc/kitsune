use cookie::Key;
use serde::{Deserialize, Serialize};
use std::{
    sync::Mutex,
    task::{self, Poll},
    vec,
};
use tower::Service;
use triomphe::Arc;

#[derive(Deserialize, Serialize)]
pub enum Level {
    Debug,
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Deserialize, Serialize)]
pub struct Flash {
    pub level: Level,
    pub message: String,
}

struct HandleInner {
    flashes: Vec<Flash>,
}

pub struct FlashHandle(Arc<Mutex<HandleInner>>);

impl FlashHandle {
    #[inline]
    pub fn push(&self, level: Level, message: String) {
        let mut guard = self.0.lock().unwrap();
        guard.flashes.push(Flash { level, message });
    }
}

pub struct ReadFlashes(Vec<Flash>);

impl IntoIterator for ReadFlashes {
    type Item = Flash;
    type IntoIter = vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for FlashService<S>
where
    S: Service<http::Request<ReqBody>, Response = http::Response<ResBody>>,
{
    type Error = S::Error;
    type Response = http::Response<ResBody>;
    type Future = todo!();

    #[inline]
    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        todo!();
    }
}
