use crate::catch_panic::CatchPanic;
use std::{future::Future, panic};

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
