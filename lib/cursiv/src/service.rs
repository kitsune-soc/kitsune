use crate::{handle::Shared, CsrfData, CsrfHandle, ResponseFuture, CSRF_COOKIE_NAME};
use cookie::Cookie;
use http::{header, Request, Response};
use std::{
    sync::Mutex,
    task::{self, Poll},
};
use tower::Service;
use triomphe::Arc;
use zeroize::{Zeroize, ZeroizeOnDrop};

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
        let csrf_cookie = req
            .headers()
            .get_all(header::COOKIE)
            .into_iter()
            .filter_map(|value| value.to_str().ok()) // Filter out all the values that aren't valid UTF-8
            .flat_map(Cookie::split_parse_encoded) // Parse all the cookie headers and flatten the resulting iterator into a contiguous one
            .flatten() // Call `.flatten()` to turn `Result<Cookie, Error>` -> `Cookie`, ignoring all the errors
            .find(|cookie| cookie.name() == CSRF_COOKIE_NAME); // Find the cookie with the name of our CSRF cookie

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
