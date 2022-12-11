use self::{catch_panic::CatchPanic, deliver_activity::DeliveryContext};
use crate::{
    activitypub::Deliverer,
    db::model::job,
    error::{Error, Result},
    state::Zustand,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DeriveActiveEnum, EntityTrait, EnumIter,
    IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod catch_panic;
mod deliver_activity;

const LINEAR_BACKOFF_DURATION: Duration = Duration::from_secs(60); // One minute
const PAUSE_BETWEEN_QUERIES: Duration = Duration::from_secs(10);

#[derive(Deserialize, Serialize)]
pub enum Job {
    DeliverActivity(DeliveryContext),
}

#[derive(Clone, Debug, DeriveActiveEnum, EnumIter, Eq, Ord, PartialEq, PartialOrd)]
#[sea_orm(rs_type = "u64", db_type = "BigUnsigned")]
pub enum JobState {
    Queued = 0,
    Running = 1,
    Failed = 2,
    Succeeded = 3,
}

async fn get_job(db_conn: &DatabaseConnection) -> Result<Option<job::Model>> {
    let txn = db_conn.begin().await?;

    let Some(mut job) = job::Entity::find()
        .filter(
            job::Column::State.eq(JobState::Queued)
                // Re-execute failed job after waiting for the linear backoff
                // TODO: Linear backoff based on "fail_count" column value (math inside SQL query)
                .or(
                    job::Column::State.eq(JobState::Failed)
                        .and(job::Column::LastFailedAt.lte(Utc::now())),
                )
                // Re-execute job if it has been running for longer than an hour (probably the worker crashed or something)
                .or(
                    job::Column::State.eq(JobState::Running)
                        .and(job::Column::UpdatedAt.lt(Utc::now() - chrono::Duration::hours(1))),
                ),
        )
        .order_by_asc(job::Column::CreatedAt)
        .lock_exclusive()
        .one(&txn)
        .await
        .map_err(Error::from)?
    else {
        return Ok(None);
    };

    job.state = JobState::Running;
    job.updated_at = Utc::now();
    job.clone().into_active_model().update(&txn).await?;

    txn.commit().await?;

    Ok(Some(job))
}

#[instrument(skip(state))]
pub async fn run(state: Zustand) {
    let mut interval = tokio::time::interval(PAUSE_BETWEEN_QUERIES);
    let deliverer = Deliverer::default();

    let mut found_job = false;

    loop {
        if !found_job {
            interval.tick().await;
            found_job = true;
        }

        let mut db_job = match get_job(&state.db_conn).await {
            Ok(Some(job)) => job,
            Ok(None) => {
                found_job = false;
                continue;
            }
            Err(err) => {
                error!(error = %err, "Failed to load job from database");
                continue;
            }
        };

        let job: Job = serde_json::from_value(db_job.context.clone())
            .expect("[Bug] Failed to deserialise job context");

        let execution_result = CatchPanic::new(async {
            match job {
                Job::DeliverActivity(ctx) => {
                    self::deliver_activity::run(&state, &deliverer, ctx).await
                }
            }
        })
        .await;

        match execution_result {
            Ok(Err(err)) => {
                error!(error = %err, "Job execution failed");

                db_job.state = JobState::Failed;
                db_job.fail_count += 1;
                db_job.last_failed_at = Some(Utc::now());
                db_job.updated_at = Utc::now();
            }
            Err(..) => {
                error!("Job execution panicked");

                db_job.state = JobState::Failed;
                db_job.fail_count += 1;
                db_job.last_failed_at = Some(Utc::now());
                db_job.updated_at = Utc::now();
            }
            _ => {
                db_job.state = JobState::Succeeded;
                db_job.updated_at = Utc::now();
            }
        }

        if let Err(err) = db_job.into_active_model().update(&state.db_conn).await {
            error!(error = %err, "Failed to update job information");
        }
    }
}
