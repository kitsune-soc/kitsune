use crate::{json::Json, schema::job_context};
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Clone, Deserialize, Serialize, Identifiable, Queryable, Selectable)]
#[diesel(table_name = job_context)]
pub struct JobContext<T> {
    pub id: Uuid,
    pub context: Json<T>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Clone, Insertable)]
#[diesel(table_name = job_context)]
pub struct NewJobContext<T> {
    pub id: Uuid,
    pub context: Json<T>,
}
