use crate::job::JobState;
use chrono::{DateTime, Utc};
use sea_orm::prelude::*;
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "jobs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub state: JobState,
    pub context: JsonValue,
    pub run_at: DateTime<Utc>,
    pub fail_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
