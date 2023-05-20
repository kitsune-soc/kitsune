use crate::{error::Result, job::Job};
use kitsune_db::{
    model::job::{JobState, NewJob},
    schema::jobs,
    PgPool,
};
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
    db_conn: PgPool,
}

impl JobService {
    pub async fn enqueue<T>(&self, enqueue: Enqueue<T>) -> Result<()>
    where
        T: Into<Job>,
    {
        let context = serde_json::to_value(enqueue.job.into())?;

        diesel::insert_into(jobs::table)
            .values(NewJob {
                id: Uuid::now_v7(),
                state: JobState::Queued,
                context,
                run_at: enqueue.run_at.unwrap_or_else(OffsetDateTime::now_utc),
            })
            .execute(&self.db_conn)
            .await?;

        Ok(())
    }
}
