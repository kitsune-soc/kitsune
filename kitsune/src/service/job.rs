use crate::{error::Result, job::Job};
use chrono::{DateTime, Utc};
use kitsune_db::{
    custom::JobState,
    entity::{jobs, prelude::Jobs},
};
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(TypedBuilder)]
pub struct Enqueue<T> {
    job: T,
    #[builder(default, setter(strip_option))]
    run_at: Option<DateTime<Utc>>,
}

#[derive(Clone, TypedBuilder)]
pub struct JobService {
    db_conn: DatabaseConnection,
}

impl JobService {
    pub async fn enqueue<T>(&self, enqueue: Enqueue<T>) -> Result<()>
    where
        T: Into<Job>,
    {
        let context = serde_json::to_value(enqueue.job.into())?;
        let run_at = enqueue.run_at.unwrap_or(Utc::now()).into();

        let job = jobs::Model {
            id: Uuid::now_v7(),
            state: JobState::Queued,
            context,
            run_at,
            fail_count: 0,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        };

        Jobs::insert(job.into_active_model())
            .exec_without_returning(&self.db_conn)
            .await?;

        Ok(())
    }
}
