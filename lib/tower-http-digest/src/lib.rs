use http::{Request, Response};
use http_body::Body;
use pin_project_lite::pin_project;
use std::{
    error::Error as StdError,
    future::Future,
    pin::Pin,
    task::{self, Poll},
};
use tower_layer::Layer;
use tower_service::Service;

type BoxError = Box<dyn StdError + Send + Sync>;

pin_project! {
    pub struct DigestFuture<B> {
        #[pin]
        body: B,
    }
}

impl<B> Future for DigestFuture<B>
where
    B: Body,
{
    type Output = Result<Response<B>, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        todo!();
    }
}

#[derive(Clone)]
pub struct DigestService<S> {
    inner: S,
}

impl<S> DigestService<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, B> Service<Request<B>> for DigestService<S>
where
    S: Service<Request<B>>,
    S::Error: Into<BoxError>,
    B: Body,
{
    type Response = Response<B>;
    type Error = BoxError;
    type Future = DigestFuture<B>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        todo!();
    }
}

#[derive(Clone, Default)]
pub struct DigestLayer {}

impl<S> Layer<S> for DigestLayer {
    type Service = DigestService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        DigestService::new(inner)
    }
}
