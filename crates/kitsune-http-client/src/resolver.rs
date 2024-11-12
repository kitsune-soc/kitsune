use futures_util::{future::BoxFuture, FutureExt};
use hickory_resolver::{lookup_ip::LookupIpIntoIter, TokioResolver};
use hyper_util::client::legacy::connect::dns::Name;
use pin_project::pin_project;
use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    task::{self, Poll},
};
use tower::{BoxError, Service};

pub struct ResolveIter {
    inner: LookupIpIntoIter,
}

impl Iterator for ResolveIter {
    type Item = SocketAddr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        Some(SocketAddr::new(item, 80))
    }
}

#[pin_project]
pub struct ResolveFut {
    #[pin]
    inner: BoxFuture<'static, Result<ResolveIter, BoxError>>,
}

impl Future for ResolveFut {
    type Output = Result<ResolveIter, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx)
    }
}

#[derive(Clone)]
pub struct Resolver {
    inner: TokioResolver,
}

impl From<TokioResolver> for Resolver {
    fn from(value: TokioResolver) -> Self {
        Self { inner: value }
    }
}

impl Service<Name> for Resolver {
    type Error = BoxError;
    type Response = ResolveIter;
    type Future = ResolveFut;

    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Name) -> Self::Future {
        let dns_client = self.inner.clone();
        let inner = async move {
            let lookup_ips = dns_client.lookup_ip(req.as_str()).await?;
            Ok(ResolveIter {
                inner: lookup_ips.into_iter(),
            })
        }
        .boxed();

        ResolveFut { inner }
    }
}
