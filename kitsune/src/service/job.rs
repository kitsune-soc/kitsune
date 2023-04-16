use crate::{error::Result, job::Job};
use kitsune_db::{
    custom::JobState,
    entity::{jobs, prelude::Jobs},
};
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel};
use time::OffsetDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(TypedBuilder)]
pub struct Enqueue<T> {
    job: T,
    #[builder(default, setter(strip_option))]
    run_at: Option<OffsetDateTime>,
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

        let job = jobs::Model {
            id: Uuid::now_v7(),
            state: JobState::Queued,
            context,
            run_at: enqueue.run_at.unwrap_or_else(OffsetDateTime::now_utc),
            fail_count: 0,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };

        Jobs::insert(job.into_active_model())
            .exec_without_returning(&self.db_conn)
            .await?;

        Ok(())
    }
}
