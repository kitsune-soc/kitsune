use crate::{catch_panic::CatchPanic, container::Service};
use std::{borrow::Cow, future::Future, panic};

#[macro_export]
macro_rules! get_resource {
    ($env_name:literal, $container_fn:path) => {
        if let Ok(url) = ::std::env::var($env_name) {
            $crate::resource::ResourceHandle::Url(url)
        } else {
            let container = $container_fn().await;
            $crate::resource::ResourceHandle::Container(container)
        }
    };
}

pub enum ResourceHandle<S>
where
    S: Service,
{
    Container(S),
    Url(String),
}

impl<S> ResourceHandle<S>
where
    S: Service,
{
    pub async fn url(&self) -> Cow<'_, str> {
        match self {
            Self::Container(container) => Cow::Owned(container.url().await),
            Self::Url(ref url) => Cow::Borrowed(url),
        }
    }
}

/// Provide a resource to the `run` closure, catch any panics that may occur while polling the future returned by `run`,
/// then run the `cleanup` closure, and resume any panic unwinds that were caught
pub async fn provide_resource<Resource, Run, Cleanup, RunFut, CleanupFut>(
    resource: Resource,
    run: Run,
    cleanup: Cleanup,
) -> RunFut::Output
where
    Resource: Clone,
    Run: FnOnce(Resource) -> RunFut,
    RunFut: Future,
    Cleanup: FnOnce(Resource) -> CleanupFut,
    CleanupFut: Future,
{
    let out = CatchPanic::new(run(resource.clone())).await;
    cleanup(resource).await;

    match out {
        Ok(ret) => ret,
        Err(err) => panic::resume_unwind(err),
    }
}
