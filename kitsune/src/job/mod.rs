use self::{
    catch_panic::CatchPanic,
    deliver::{
        accept::DeliverAccept, create::DeliverCreate, delete::DeliverDelete,
        favourite::DeliverFavourite, follow::DeliverFollow, unfavourite::DeliverUnfavourite,
        unfollow::DeliverUnfollow, update::DeliverUpdate,
    },
};
use crate::{
    activitypub::Deliverer,
    error::{Error, Result},
    state::Zustand,
};
use async_trait::async_trait;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use enum_dispatch::enum_dispatch;
use futures_util::{stream::FuturesUnordered, TryStreamExt};
use kitsune_db::{
    model::job::{Job as DbJob, JobState, UpdateFailedJob},
    schema::jobs,
    PgPool,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;
use time::OffsetDateTime;
use tokio::task::JoinSet;

mod catch_panic;

pub mod deliver;

const EXECUTION_TIMEOUT_DURATION: Duration = Duration::from_secs(30);
const MAX_CONCURRENT_REQUESTS: usize = 10;
const PAUSE_BETWEEN_QUERIES: Duration = Duration::from_secs(5);

#[enum_dispatch(Runnable)]
#[derive(Deserialize, Serialize)]
pub enum Job {
    DeliverAccept,
    DeliverCreate,
    DeliverDelete,
    DeliverFavourite,
    DeliverFollow,
    DeliverUnfavourite,
    DeliverUnfollow,
    DeliverUpdate,
}

#[derive(Clone, Copy)]
pub struct JobContext<'a> {
    deliverer: &'a Deliverer,
    state: &'a Zustand,
}

#[async_trait]
#[enum_dispatch]
pub trait Runnable: DeserializeOwned + Serialize {
    async fn run(&self, ctx: JobContext<'_>) -> Result<()>;

    /// Defaults to exponential backoff
    fn backoff(&self, previous_tries: u32) -> u64 {
        u64::pow(2, previous_tries)
    }
}

// Takes owned values to make the lifetime of the returned future static
#[instrument(skip_all, fields(job_id = %db_job.id))]
async fn execute_one(db_job: DbJob, state: Zustand, deliverer: Deliverer) -> Result<()> {
    let job: Job = serde_json::from_value(db_job.context.clone())
        .expect("[Bug] Failed to deserialise job context");

    let execution_result = CatchPanic::new(job.run(JobContext {
        deliverer: &deliverer,
        state: &state,
    }))
    .await;

    if let Ok(Err(ref err)) = execution_result {
        error!(error = ?err, "Job execution failed");
    } else if execution_result.is_err() {
        error!("Job execution panicked");
    }

    let mut db_conn = state.db_conn.get().await?;
    #[allow(clippy::cast_possible_truncation)]
    match execution_result {
        Ok(Err(..)) | Err(..) => {
            increment_counter!("failed_jobs");

            let fail_count = db_job.fail_count + 1;
            #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
            let backoff_duration = time::Duration::seconds(job.backoff(fail_count as u32) as i64);

            diesel::update(&db_job)
                .set(UpdateFailedJob {
                    fail_count,
                    state: JobState::Failed,
                    run_at: OffsetDateTime::now_utc() + backoff_duration,
                })
                .execute(&mut db_conn)
                .await?;
        }
        _ => {
            increment_counter!("succeeded_jobs");

            diesel::update(&db_job)
                .set(jobs::state.eq(JobState::Succeeded))
                .execute(&mut db_conn)
                .await?;
        }
    }

    Ok(())
}

async fn get_jobs(db_conn: &PgPool, num_jobs: usize) -> Result<Vec<DbJob>> {
    let mut db_conn = db_conn.get().await?;

    let jobs = db_conn
        .transaction(|tx| {
            async move {
                let jobs = jobs::table
                    .filter(
                        (jobs::state
                            .eq(JobState::Queued)
                            .or(jobs::state.eq(JobState::Failed))
                            .and(jobs::run_at.le(kitsune_db::function::now())))
                        .or(jobs::state.eq(JobState::Running).and(
                            jobs::updated_at
                                .lt(OffsetDateTime::now_utc() - time::Duration::hours(1)),
                        )),
                    )
                    .limit(num_jobs as i64)
                    .order(jobs::id.asc())
                    .for_update()
                    .load(tx)
                    .await?;

                // New scope to ensure `update_jobs` is getting dropped
                // Otherwise this will prevent us from returning the `jobs` list
                {
                    jobs.iter()
                        .map(|job| {
                            diesel::update(job)
                                .set(jobs::state.eq(JobState::Running))
                                .execute(tx)
                        })
                        .collect::<FuturesUnordered<_>>()
                        .map_ok(|_| ())
                        .try_collect::<()>()
                        .await?;
                }

                Ok::<_, Error>(jobs)
            }
            .scope_boxed()
        })
        .await?;

    Ok(jobs)
}

#[instrument(skip(state))]
pub async fn run_dispatcher(state: Zustand, num_job_workers: usize) {
    let deliverer = Deliverer::builder()
        .federation_filter(state.service.federation_filter.clone())
        .build();

    let mut executor = JoinSet::new();
    let mut pause_between_queries = tokio::time::interval(PAUSE_BETWEEN_QUERIES);
    let mut do_pause = false;

    loop {
        if do_pause {
            pause_between_queries.tick().await;
            do_pause = false;
        }

        let num_jobs = num_job_workers - executor.len();
        let jobs = match get_jobs(&state.db_conn, num_jobs).await {
            Ok(jobs) => jobs,
            Err(err) => {
                error!(error = %err, "Failed to get jobs from database");
                continue;
            }
        };

        if jobs.is_empty() && executor.is_empty() {
            do_pause = true;
            continue;
        }

        for job in jobs {
            executor.spawn(execute_one(job, state.clone(), deliverer.clone()));
        }

        if tokio::time::timeout(EXECUTION_TIMEOUT_DURATION, executor.join_next())
            .await
            .is_err()
        {
            debug!("Reached timeout. Waiting for job");
        }
    }
}
