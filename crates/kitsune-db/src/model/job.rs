use crate::{
    json::Json,
    schema::{jobs, sql_types},
};
use diesel::{Identifiable, Insertable, Queryable, Selectable, prelude::AsChangeset};
use diesel_derive_enum::DbEnum;
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Clone, Copy, DbEnum, Debug, Deserialize, Serialize)]
#[db_enum(existing_type_path = "sql_types::JobState")]
pub enum JobState {
    Queued,
    Running,
    Failed,
    Completed,
}

#[derive(Clone, Deserialize, Serialize, Identifiable, Queryable, Selectable)]
#[diesel(check_for_backend(diesel::pg::Pg), table_name = jobs)]
pub struct Job<T> {
    pub id: Uuid,
    pub meta: Json<T>,

    pub state: JobState,
    pub fail_count: i32,
    pub run_at: Timestamp,

    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = jobs)]
pub struct NewJob<T> {
    pub id: Uuid,
    pub meta: Json<T>,

    pub state: JobState,
    pub run_at: Timestamp,
}

#[derive(AsChangeset)]
#[diesel(table_name = jobs)]
pub struct RequeueChangeset {
    pub fail_count: i32,
    pub state: JobState,
    pub run_at: Timestamp,
}
