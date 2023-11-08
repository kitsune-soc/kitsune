use http::{header::InvalidHeaderValue, HeaderName, HeaderValue, Response};
use itertools::Itertools;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{self, Poll},
};
use tower_layer::Layer;
use tower_service::Service;

static HEADER_NAME: HeaderName = HeaderName::from_static("x-clacks-overhead");

#[inline]
fn build_names_value<'a, I>(names: I) -> Result<Arc<HeaderValue>, InvalidHeaderValue>
where
    I: Iterator<Item = &'a str>,
{
    let names = format!(
        "GNU {}",
        Itertools::intersperse(names, ", ").collect::<String>()
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
        I: Iterator<Item = &'a str>,
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
        I: Iterator<Item = &'a str>,
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
