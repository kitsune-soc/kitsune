use http::{header::InvalidHeaderValue, HeaderName, HeaderValue, Response};
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
};
use tower::{Layer, Service};
use triomphe::Arc;

static HEADER_NAME: HeaderName = HeaderName::from_static("x-clacks-overhead");

#[inline]
fn build_names_value<'a, I>(names: I) -> Result<Arc<HeaderValue>, InvalidHeaderValue>
where
    I: IntoIterator<Item = &'a str>,
{
    let names = format!(
        "GNU {}",
        itertools::intersperse(names, ", ").collect::<String>()
    )
    .parse()?;

    Ok(Arc::new(names))
}

pin_project! {
    pub struct XClacksOverheadFuture<F> {
        #[pin]
        future: F,
        names: Arc<HeaderValue>,
    }
}

impl<F, B, E> Future for XClacksOverheadFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        this.future.poll(cx).map_ok(|mut response| {
            response
                .headers_mut()
                .insert(HEADER_NAME.clone(), (**this.names).clone());

            response
        })
    }
}

#[derive(Clone)]
pub struct XClacksOverheadService<S> {
    inner: S,
    names: Arc<HeaderValue>,
}

impl<S> XClacksOverheadService<S> {
    pub fn new<'a, I>(inner: S, names: I) -> Result<Self, InvalidHeaderValue>
    where
        I: IntoIterator<Item = &'a str>,
    {
        Ok(Self {
            inner,
            names: build_names_value(names)?,
        })
    }
}

impl<S, Request, ResBody> Service<Request> for XClacksOverheadService<S>
where
    S: Service<Request, Response = Response<ResBody>>,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = XClacksOverheadFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        XClacksOverheadFuture {
            future: self.inner.call(req),
            names: Arc::clone(&self.names),
        }
    }
}

#[derive(Clone)]
pub struct XClacksOverheadLayer {
    names: Arc<HeaderValue>,
}

impl XClacksOverheadLayer {
    pub fn new<'a, I>(names: I) -> Result<Self, InvalidHeaderValue>
    where
        I: IntoIterator<Item = &'a str>,
    {
        Ok(Self {
            names: build_names_value(names)?,
        })
    }
}

impl<S> Layer<S> for XClacksOverheadLayer {
    type Service = XClacksOverheadService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        XClacksOverheadService {
            inner,
            names: Arc::clone(&self.names),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{XClacksOverheadLayer, HEADER_NAME};
    use http::{Request, Response};
    use std::convert::Infallible;
    use tower::{service_fn, Layer, ServiceExt};

    #[futures_test::test]
    async fn add_header() {
        let service = XClacksOverheadLayer::new(["Johnny"])
            .unwrap()
            .layer(service_fn(|_req: Request<()>| async move {
                Ok::<_, Infallible>(Response::new(()))
            }));

        let response = service.oneshot(Request::new(())).await.unwrap();
        let clacks_overhead = response.headers().get(&HEADER_NAME).unwrap();
        assert_eq!(clacks_overhead.as_bytes(), b"GNU Johnny");
    }
}
