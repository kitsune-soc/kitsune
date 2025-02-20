use crate::{CsrfHandle, CSRF_COOKIE_NAME};
use cookie::{Cookie, Expiration, SameSite};
use http::{header, HeaderValue, Response};
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{self, ready, Poll},
};

pin_project! {
    pub struct ResponseFuture<F> {
        #[pin]
        pub(crate) inner: F,
        pub(crate) handle: CsrfHandle,
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
            .expires(Expiration::Session)
            .same_site(SameSite::Strict)
            .build();

        let guard = this.handle.inner.lock().unwrap();
        match guard.set_data {
            Some(ref set_data) => {
                let value = format!("{}.{}", set_data.hash, set_data.message);
                cookie.set_value(value);
            }
            _ => {
                cookie.make_removal();
            }
        }

        let encoded_cookie = cookie.encoded().to_string();
        let header_value = HeaderValue::from_str(&encoded_cookie).unwrap();

        response
            .headers_mut()
            .append(header::SET_COOKIE, header_value);

        Poll::Ready(Ok(response))
    }
}
