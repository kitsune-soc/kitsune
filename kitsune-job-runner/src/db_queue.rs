use async_trait::async_trait;
use athena::{
    JobContextRepository, JobData, JobDetails, JobResult, KeeperOfTheSecrets, Outcome,
    consts::{MAX_RETRIES, MIN_IDLE_TIME},
};
use color_eyre::eyre;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use iso8601_timestamp::Timestamp;
use just_retry::{
    JustRetryPolicy, StartTime,
    retry_policies::{Jitter, policies::ExponentialBackoff},
};
use kitsune_db::{
    PgPool,
    function::now,
    json::Json,
    model::job::{Job, JobState, NewJob, RequeueChangeset},
    schema::jobs,
    with_connection, with_transaction,
};
use kitsune_jobs::KitsuneContextRepo;
use std::{ops::ControlFlow, time::SystemTime};
use trials::attempt;
use typed_builder::TypedBuilder;

type Result<T, E = athena::Error> = std::result::Result<T, E>;

#[derive(TypedBuilder)]
pub struct DbQueue {
    context_repo: KitsuneContextRepo,
    db_pool: PgPool,
    #[builder(default = MAX_RETRIES)]
    max_retries: u32,
}

#[async_trait]
impl athena::JobQueue for DbQueue {
    type ContextRepository = KitsuneContextRepo;

    fn context_repository(&self) -> &Self::ContextRepository {
        &self.context_repo
    }

    async fn enqueue(
        &self,
        job_details: JobDetails<<Self::ContextRepository as JobContextRepository>::JobContext>,
    ) -> Result<()> {
        self.context_repository()
            .store_context(job_details.job_id, job_details.context)
            .await
            .map_err(|err| athena::Error::ContextRepository(err.into()))?;

        let result: eyre::Result<()> = attempt! { async
            with_connection!(self.db_pool, |conn| {
                diesel::insert_into(jobs::table)
                    .values(&NewJob {
                        id: job_details.job_id,
                        meta: Json(KeeperOfTheSecrets::empty()),
                        state: JobState::Queued,
                        run_at: job_details.run_at.unwrap_or_else(Timestamp::now_utc),
                    })
                    .execute(conn)
                    .await?;

                eyre::Ok(())
            })?
        };

        result.map_err(|err| athena::Error::Other(err.into()))
    }

    async fn fetch_job_data(&self, max_jobs: usize) -> Result<Vec<JobData>> {
        let result: eyre::Result<Vec<Job<KeeperOfTheSecrets>>> = attempt! { async
            with_transaction!(self.db_pool, |tx| {
                let jobs: Vec<Job<KeeperOfTheSecrets>> = jobs::table
                    .filter(
                        jobs::state.eq_any([JobState::Queued, JobState::Failed])
                            .and(jobs::run_at.le(now()))
                    )
                    .or_filter(
                        jobs::state.eq(JobState::Running)
                            .and(jobs::updated_at.lt(Timestamp::now_utc() - MIN_IDLE_TIME))
                    )
                    .limit(max_jobs as i64)
                    .for_update()
                    .skip_locked()
                    .select(Job::as_select())
                    .get_results(tx)
                    .await?;

                let job_ids = jobs.iter().map(|job| job.id);

                diesel::update(jobs::table.filter(jobs::id.eq_any(job_ids)))
                    .set(jobs::state.eq(JobState::Running))
                    .execute(tx)
                    .await?;

                eyre::Ok(jobs)
            })?
        };

        let jobs = result.map_err(|err| athena::Error::Other(err.into()))?;

        let data: Vec<JobData> = jobs
            .into_iter()
            .map(|job| JobData {
                job_id: job.id,
                fail_count: job.fail_count as u32,
                ctx: job.meta.0,
            })
            .collect();

        Ok(data)
    }

    async fn reclaim_job(&self, job_data: &JobData) -> Result<()> {
        let result: eyre::Result<()> = attempt! { async
            with_connection!(self.db_pool, |conn| {
                diesel::update(jobs::table.find(job_data.job_id))
                    .set(jobs::updated_at.eq(Timestamp::now_utc()))
                    .execute(conn)
                    .await
            })?;
        };

        result.map_err(|err| athena::Error::Other(err.into()))
    }

    async fn complete_job(&self, state: &JobResult<'_>) -> Result<()> {
        // TODO: wrap the whole thing in a transaction, i guess?

        let delete_job = match state.outcome {
            Outcome::Fail { fail_count } => {
                let backoff = ExponentialBackoff::builder()
                    .jitter(Jitter::Bounded)
                    .build_with_max_retries(self.max_retries);

                if let ControlFlow::Continue(delta) =
                    backoff.should_retry(StartTime::Irrelevant, fail_count)
                {
                    let backoff_timestamp = Timestamp::from(SystemTime::now() + delta);

                    let result: eyre::Result<()> = attempt! { async
                        with_connection!(self.db_pool, |conn| {
                            diesel::update(jobs::table)
                                .set(
                                    RequeueChangeset {
                                        fail_count: (fail_count + 1) as i32,
                                        state: JobState::Failed,
                                        run_at: backoff_timestamp,
                                    }
                                )
                                .execute(conn)
                                .await
                        })?;
                    };

                    result.map_err(|err| athena::Error::Other(err.into()))?;

                    false // Do not delete the job if we were able to retry it one more time
                } else {
                    true // We hit the maximum amount of retries, we won't re-enqueue the job, so we can just delete it
                }
            }
            Outcome::Success => true, // Execution succeeded, we don't need the job anymore
        };

        if delete_job {
            let result: eyre::Result<()> = attempt! { async
                with_connection!(self.db_pool, |conn| {
                    diesel::delete(jobs::table.find(state.job_id))
                        .execute(conn)
                        .await
                })?;
            };

            result.map_err(|err| athena::Error::Other(err.into()))?;

            self.context_repository()
                .remove_context(state.job_id)
                .await
                .map_err(|err| athena::Error::ContextRepository(err.into()))?;
        }

        Ok(())
    }
}
