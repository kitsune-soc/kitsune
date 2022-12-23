use self::{
    catch_panic::CatchPanic, deliver_create::CreateDeliveryContext,
    deliver_delete::DeleteDeliveryContext,
};
use crate::{
    activitypub::Deliverer,
    db::model::job,
    error::{Error, Result},
    state::Zustand,
};
use chrono::Utc;
use once_cell::sync::Lazy;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DeriveActiveEnum, EntityTrait,
    EnumIter, IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod catch_panic;

pub mod deliver_create;
pub mod deliver_delete;

const MAX_CONCURRENT_REQUESTS: usize = 10;
const PAUSE_BETWEEN_QUERIES: Duration = Duration::from_secs(10);
static LINEAR_BACKOFF_DURATION: Lazy<chrono::Duration> = Lazy::new(|| chrono::Duration::minutes(1)); // One minute

#[derive(Deserialize, Serialize)]
pub enum Job {
    DeliverCreate(CreateDeliveryContext),
    DeliverDelete(DeleteDeliveryContext),
}

#[derive(Clone, Debug, DeriveActiveEnum, EnumIter, Eq, Ord, PartialEq, PartialOrd)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum JobState {
    Queued = 0,
    Running = 1,
    Failed = 2,
    Succeeded = 3,
}

async fn get_job(db_conn: &DatabaseConnection) -> Result<Option<job::Model>> {
    let txn = db_conn.begin().await?;

    let Some(job) = job::Entity::find()
        .filter(
            job::Column::State.eq(JobState::Queued)
                .or(job::Column::State.eq(JobState::Failed))
                .and(job::Column::RunAt.lte(Utc::now()))
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

    let mut update_job = job.into_active_model();
    update_job.state = ActiveValue::Set(JobState::Running);
    update_job.updated_at = ActiveValue::Set(Utc::now());
    let job = update_job.update(&txn).await?;

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

        let db_job = match get_job(&state.db_conn).await {
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
                Job::DeliverCreate(ctx) => self::deliver_create::run(&state, &deliverer, ctx).await,
                Job::DeliverDelete(ctx) => self::deliver_delete::run(&state, &deliverer, ctx).await,
            }
        })
        .await;

        if let Ok(Err(ref err)) = execution_result {
            error!(error = %err, "Job execution failed");
        }

        let mut update_model = db_job.clone().into_active_model();
        #[allow(clippy::cast_possible_truncation)]
        match execution_result {
            Ok(Err(..)) | Err(..) => {
                update_model.state = ActiveValue::Set(JobState::Failed);
                update_model.fail_count = ActiveValue::Set(db_job.fail_count + 1);
                update_model.run_at = ActiveValue::Set(
                    Utc::now() + (*LINEAR_BACKOFF_DURATION * (db_job.fail_count as i32)),
                );
                update_model.updated_at = ActiveValue::Set(Utc::now());
            }
            _ => {
                update_model.state = ActiveValue::Set(JobState::Succeeded);
                update_model.updated_at = ActiveValue::Set(Utc::now());
            }
        }

        if let Err(err) = update_model.update(&state.db_conn).await {
            error!(error = %err, "Failed to update job information");
        }
    }
}
