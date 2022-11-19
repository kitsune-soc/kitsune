use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::job::JobState;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "jobs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub state: JobState,
    pub context: JsonValue,
    pub fail_count: u64,
    pub last_failed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
