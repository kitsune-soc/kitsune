use crate::{error::Result, job::Job};
use diesel_async::RunQueryDsl;
use iso8601_timestamp::Timestamp;
use kitsune_db::{
    json::Json,
    model::job::{JobState, NewJob},
    schema::jobs,
    PgPool,
};
use speedy_uuid::Uuid;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Enqueue<T> {
    job: T,
    #[builder(default, setter(strip_option))]
    run_at: Option<Timestamp>,
}

#[derive(Clone, TypedBuilder)]
pub struct JobService {
    db_conn: PgPool,
}

impl JobService {
    pub async fn enqueue<T>(&self, enqueue: Enqueue<T>) -> Result<()>
    where
        Job: From<T>,
    {
        let mut db_conn = self.db_conn.get().await?;
        diesel::insert_into(jobs::table)
            .values(NewJob {
                id: Uuid::now_v7(),
                state: JobState::Queued,
                context: Json(Job::from(enqueue.job)),
                run_at: enqueue.run_at.unwrap_or_else(Timestamp::now_utc),
            })
            .on_conflict_do_nothing()
            .execute(&mut db_conn)
            .await?;

        Ok(())
    }
}
