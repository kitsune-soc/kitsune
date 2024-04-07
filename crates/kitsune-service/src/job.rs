use athena::{JobDetails, JobQueue};
use iso8601_timestamp::Timestamp;
use kitsune_error::Result;
use kitsune_jobs::{Job, KitsuneContextRepo};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Enqueue<T> {
    job: T,
    #[builder(default, setter(strip_option))]
    run_at: Option<Timestamp>,
}

#[derive(Clone, TypedBuilder)]
pub struct JobService {
    job_queue: JobQueue<KitsuneContextRepo>,
}

impl JobService {
    pub async fn enqueue<T>(&self, enqueue: Enqueue<T>) -> Result<()>
    where
        Job: From<T>,
    {
        self.job_queue
            .enqueue(
                JobDetails::builder()
                    .context(enqueue.job)
                    .run_at(enqueue.run_at)
                    .build(),
            )
            .await?;

        Ok(())
    }
}
