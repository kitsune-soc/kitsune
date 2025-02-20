use crate::{CSRF_COOKIE_NAME, CsrfData, CsrfHandle, ResponseFuture, handle::Shared};
use cookie::Cookie;
use http::{Request, Response, header};
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
        let read_data = {
            let mut csrf_data = None;
            'outer: for header in req.headers().get_all(header::COOKIE) {
                let Ok(value_str) = header.to_str() else {
                    continue;
                };

                for cookie in Cookie::split_parse_encoded(value_str) {
                    let Ok(cookie) = cookie else {
                        continue;
                    };

                    if cookie.name() == CSRF_COOKIE_NAME {
                        let Some((hash, message)) = cookie.value_trimmed().split_once('.') else {
                            continue;
                        };

                        csrf_data = Some(CsrfData {
                            hash: hash.into(),
                            message: message.into(),
                        });

                        break 'outer;
                    }
                }
            }

            csrf_data
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
