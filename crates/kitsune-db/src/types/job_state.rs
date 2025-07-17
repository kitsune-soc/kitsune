use crate::schema::sql_types;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, DbEnum, Debug, Deserialize, Serialize)]
#[db_enum(existing_type_path = "sql_types::JobState")]
pub enum JobState {
    Queued,
    Running,
    Failed,
    Completed,
}
