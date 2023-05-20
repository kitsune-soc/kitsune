use crate::schema::jobs;
use diesel::{
    pg::Pg,
    serialize::{self, Output, ToSql},
    sql_types::Integer,
    AsExpression, FromSqlRow, Identifiable, Insertable, Queryable,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Deserialize, Serialize, Identifiable, Queryable)]
pub struct Job {
    pub id: Uuid,
    pub state: JobState,
    pub context: Value,
    pub run_at: OffsetDateTime,
    pub fail_count: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = jobs)]
pub struct NewJob {
    pub id: Uuid,
    pub state: JobState,
    pub context: Value,
    pub run_at: OffsetDateTime,
}

#[derive(
    AsExpression,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    FromSqlRow,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[diesel(sql_type = Integer)]
/// State a job can be in
pub enum JobState {
    /// Queued
    ///
    /// The job is queued for execution. It has never been executed before
    Queued = 0,
    /// Running
    ///
    /// The job is running at the moment.
    Running = 1,
    /// Failed
    ///
    /// The job has failed before. This is basically equivalent to the `Queued` state
    Failed = 2,
    /// Succeeded
    ///
    /// The job has run to completion and not errored out. The job will not be reprocessed.
    /// This entry is kept for historic purposes and can be deleted at any point in time without impacting anything.
    Succeeded = 3,
}

impl ToSql<Integer, Pg> for JobState
where
    i32: ToSql<Integer, Pg>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <i32 as ToSql<Integer, _>>::to_sql(&(*self as i32), &mut out.reborrow())
    }
}
