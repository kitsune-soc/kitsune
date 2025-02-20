use crate::{
    consts::MIN_IDLE_TIME,
    error::{Error, Result},
    JobContextRepository, JobData, JobQueue, JobResult, Outcome, Runnable,
};
use ahash::HashMap;
use futures_util::TryStreamExt;
use just_retry::RetryExt;
use speedy_uuid::Uuid;
use std::{pin::pin, time::Duration};
use tokio::time::Instant;
use tokio_util::task::TaskTracker;
use triomphe::Arc;

type ContextFor<Queue> =
    <<<Queue as JobQueue>::ContextRepository as JobContextRepository>::JobContext as Runnable>::Context;

pub async fn spawn_jobs<Q>(
    queue: &Q,
    max_jobs: usize,
    run_ctx: Arc<ContextFor<Q>>,
    task_tracker: &TaskTracker,
) -> Result<()>
where
    Q: JobQueue + Clone,
{
    let job_data = queue.fetch_job_data(max_jobs).await?;
    let job_ids: Vec<Uuid> = job_data.iter().map(|data| data.job_id).collect();

    let context_stream = queue
        .context_repository()
        .fetch_context(job_ids.into_iter())
        .await
        .map_err(|err| Error::ContextRepository(err.into()))?;
    let mut context_stream = pin!(context_stream);

    // Collect all the job data into a hashmap indexed by the job ID
    // This is because we don't enforce an ordering with the batch fetching
    let job_data = job_data
        .into_iter()
        .map(|data| (data.job_id, data))
        .collect::<HashMap<Uuid, JobData>>();
    let job_data = Arc::new(job_data);

    while let Some((job_id, job_ctx)) = context_stream
        .try_next()
        .await
        .map_err(|err| Error::ContextRepository(err.into()))?
    {
        let queue = queue.clone();
        let job_data = Arc::clone(&job_data);
        let run_ctx = Arc::clone(&run_ctx);

        task_tracker.spawn(async move {
            let job_data = &job_data[&job_id];
            let mut run_fut = pin!(job_ctx.run(&run_ctx));

            let tick_period = MIN_IDLE_TIME - Duration::from_secs(2 * 60);
            let mut tick_interval =
                tokio::time::interval_at(Instant::now() + tick_period, tick_period);

            let result = loop {
                tokio::select! {
                    result = &mut run_fut => break result,
                    _ = tick_interval.tick() => {
                        (|| queue.reclaim_job(job_data))
                            .retry(just_retry::backoff_policy())
                            .await
                            .expect("Failed to reclaim job");
                    }
                }
            };

            let job_state = match result {
                Err(error) => {
                    error!(error = ?error.into(), "Failed run job");
                    JobResult {
                        outcome: Outcome::Fail {
                            fail_count: job_data.fail_count,
                        },
                        job_id,
                        ctx: &job_data.ctx,
                    }
                }
                _ => JobResult {
                    outcome: Outcome::Success,
                    job_id,
                    ctx: &job_data.ctx,
                },
            };

            (|| queue.complete_job(&job_state))
                .retry(just_retry::backoff_policy())
                .await
                .expect("Failed to mark job as completed");
        });
    }

    Ok(())
}
