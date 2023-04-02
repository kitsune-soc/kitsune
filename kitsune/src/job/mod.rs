use self::{
    catch_panic::CatchPanic,
    deliver::{
        create::DeliverCreate, delete::DeliverDelete, favourite::DeliverFavourite,
        unfavourite::DeliverUnfavourite,
    },
};
use crate::{
    activitypub::Deliverer,
    error::{Error, Result},
    state::Zustand,
};
use async_trait::async_trait;
use chrono::Utc;
use enum_dispatch::enum_dispatch;
use kitsune_db::entity::jobs;
use kitsune_db::{custom::JobState, entity::prelude::Jobs};
use once_cell::sync::Lazy;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;
use tokio::task::JoinSet;

mod catch_panic;

pub mod deliver;

const PAUSE_BETWEEN_QUERIES: Duration = Duration::from_secs(5);
const MAX_CONCURRENT_REQUESTS: usize = 10;
const EXECUTION_TIMEOUT_DURATION: Duration = Duration::from_secs(30);
static LINEAR_BACKOFF_DURATION: Lazy<chrono::Duration> = Lazy::new(|| chrono::Duration::minutes(1)); // One minute

#[enum_dispatch(JobRunner)]
#[derive(Deserialize, Serialize)]
pub enum Job {
    DeliverCreate,
    DeliverDelete,
    DeliverFavourite,
    DeliverUnfavourite,
}

pub struct JobContext<'a> {
    deliverer: &'a Deliverer,
    state: &'a Zustand,
}

#[async_trait]
#[enum_dispatch]
pub trait JobRunner: DeserializeOwned {
    async fn run(self, ctx: JobContext<'_>) -> Result<()>;
}

// Takes owned values to make the lifetime of the returned future static
async fn execute_one(db_job: jobs::Model, state: Zustand, deliverer: Deliverer) {
    let job: Job = serde_json::from_value(db_job.context.clone())
        .expect("[Bug] Failed to deserialise job context");

    let execution_result = CatchPanic::new(job.run(JobContext {
        deliverer: &deliverer,
        state: &state,
    }))
    .await;

    if let Ok(Err(ref err)) = execution_result {
        error!(error = %err, "Job execution failed");
    }

    let mut update_model = db_job.clone().into_active_model();
    #[allow(clippy::cast_possible_truncation)]
    match execution_result {
        Ok(Err(..)) | Err(..) => {
            increment_counter!("failed_jobs");

            update_model.state = ActiveValue::Set(JobState::Failed);
            update_model.fail_count = ActiveValue::Set(db_job.fail_count + 1);
            update_model.run_at = ActiveValue::Set(
                (Utc::now() + (*LINEAR_BACKOFF_DURATION * db_job.fail_count)).into(),
            );
            update_model.updated_at = ActiveValue::Set(Utc::now().into());
        }
        _ => {
            increment_counter!("succeeded_jobs");

            update_model.state = ActiveValue::Set(JobState::Succeeded);
            update_model.updated_at = ActiveValue::Set(Utc::now().into());
        }
    }

    if let Err(err) = update_model.update(&state.db_conn).await {
        error!(error = %err, "Failed to update job information");
    }
}

async fn get_jobs(db_conn: &DatabaseConnection, num_jobs: usize) -> Result<Vec<jobs::Model>> {
    let txn = db_conn.begin().await?;

    let jobs = Jobs::find()
        .filter(
            jobs::Column::State
                .eq(JobState::Queued)
                .or(jobs::Column::State.eq(JobState::Failed))
                .and(jobs::Column::RunAt.lte(Utc::now()))
                // Re-execute job if it has been running for longer than an hour (probably the worker crashed or something)
                .or(jobs::Column::State
                    .eq(JobState::Running)
                    .and(jobs::Column::UpdatedAt.lt(Utc::now() - chrono::Duration::hours(1)))),
        )
        .limit(num_jobs as u64)
        .order_by_asc(jobs::Column::CreatedAt)
        .lock_exclusive()
        .all(&txn)
        .await
        .map_err(Error::from)?;

    let update_jobs = jobs.iter().map(|job| {
        let mut update_job = job.clone().into_active_model();
        update_job.state = ActiveValue::Set(JobState::Running);
        update_job.updated_at = ActiveValue::Set(Utc::now().into());
        update_job
    });

    for update_job in update_jobs {
        Jobs::update(update_job).exec(&txn).await?;
    }

    txn.commit().await?;

    Ok(jobs)
}

#[instrument(skip(state))]
pub async fn run_dispatcher(state: Zustand, num_job_workers: usize) {
    let deliverer = Deliverer::default();
    let mut executor = JoinSet::new();
    let mut execution_timeout = tokio::time::interval(EXECUTION_TIMEOUT_DURATION);

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

        loop {
            tokio::select! {
                None = executor.join_next() => break,
                _ = execution_timeout.tick() => break,
            }
        }
    }
}
