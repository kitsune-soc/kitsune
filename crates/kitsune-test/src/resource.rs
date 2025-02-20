use crate::catch_panic::CatchPanic;
use std::panic;

/// Provide a resource to the `run` closure, catch any panics that may occur while polling the future returned by `run`,
/// then run the `cleanup` closure, and resume any panic unwinds that were caught
pub async fn provide_resource<Resource, Run, Cleanup, RunOutput>(
    resource: Resource,
    run: Run,
    cleanup: Cleanup,
) -> RunOutput
where
    Resource: Clone,
    Run: AsyncFnOnce(Resource) -> RunOutput,
    Cleanup: AsyncFnOnce(Resource),
{
    let out = CatchPanic::new(run(resource.clone())).await;
    cleanup(resource).await;

    match out {
        Ok(ret) => ret,
        Err(err) => panic::resume_unwind(err),
    }
}
