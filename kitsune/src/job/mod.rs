use self::{
    catch_panic::CatchPanic,
    deliver::{
        create::CreateDeliveryContext, delete::DeleteDeliveryContext,
        favourite::FavouriteDeliveryContext, unfavourite::UnfavouriteDeliveryContext,
    },
};
use crate::{
    activitypub::Deliverer,
    error::{Error, Result},
    state::Zustand,
};
use chrono::Utc;
use kitsune_db::custom::JobState;
use kitsune_db::entity::jobs;
use once_cell::sync::Lazy;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod catch_panic;

pub mod deliver;

const MAX_CONCURRENT_REQUESTS: usize = 10;
const PAUSE_BETWEEN_QUERIES: Duration = Duration::from_secs(10);
static LINEAR_BACKOFF_DURATION: Lazy<chrono::Duration> = Lazy::new(|| chrono::Duration::minutes(1)); // One minute

#[derive(Deserialize, Serialize)]
pub enum Job {
    DeliverCreate(CreateDeliveryContext),
    DeliverDelete(DeleteDeliveryContext),
    DeliverFavourite(FavouriteDeliveryContext),
    DeliverUnfavourite(UnfavouriteDeliveryContext),
}

async fn get_job(db_conn: &DatabaseConnection) -> Result<Option<jobs::Model>> {
    let txn = db_conn.begin().await?;

    let Some(job) = jobs::Entity::find()
        .filter(
            jobs::Column::State.eq(JobState::Queued)
                .or(jobs::Column::State.eq(JobState::Failed))
                .and(jobs::Column::RunAt.lte(Utc::now()))
                // Re-execute job if it has been running for longer than an hour (probably the worker crashed or something)
                .or(
                    jobs::Column::State.eq(JobState::Running)
                        .and(jobs::Column::UpdatedAt.lt(Utc::now() - chrono::Duration::hours(1))),
                ),
        )
        .order_by_asc(jobs::Column::CreatedAt)
        .lock_exclusive()
        .one(&txn)
        .await
        .map_err(Error::from)?
    else {
        return Ok(None);
    };

    let mut update_job = job.into_active_model();
    update_job.state = ActiveValue::Set(JobState::Running);
    update_job.updated_at = ActiveValue::Set(Utc::now().into());
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
                Job::DeliverCreate(ctx) => {
                    self::deliver::create::run(&state, &deliverer, ctx).await
                }
                Job::DeliverDelete(ctx) => {
                    self::deliver::delete::run(&state, &deliverer, ctx).await
                }
                Job::DeliverFavourite(ctx) => {
                    self::deliver::favourite::run(&state, &deliverer, ctx).await
                }
                Job::DeliverUnfavourite(ctx) => {
                    self::deliver::unfavourite::run(&state, &deliverer, ctx).await
                }
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
}
