use crate::{catch_panic::CatchPanic, container::Service};
use std::{borrow::Cow, future::Future, panic, sync::OnceLock};

pub static CONTAINER_CLIENT: OnceLock<testcontainers::clients::Cli> = OnceLock::new();

#[macro_export]
macro_rules! get_resource {
    ($env_name:literal, $container_fn:path) => {
        ::std::env::var($env_name).map_or_else(
            |_| {
                // Only initialize client if we actually need it
                let client = $crate::resource::CONTAINER_CLIENT.get_or_init(|| {
                    ::testcontainers::clients::Cli::new::<::testcontainers::core::env::Os>()
                });

                $crate::resource::ResourceHandle::Container($container_fn(client))
            },
            $crate::resource::ResourceHandle::Url,
        )
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
    pub fn url(&self) -> Cow<'_, str> {
        match self {
            Self::Container(container) => Cow::Owned(container.url()),
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
