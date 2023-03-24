use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct Field {
    pub name: String,
    pub value: String,
    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct Source {
    pub privacy: String,
    pub sensitive: bool,
    pub language: String,
    pub note: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct Account {
    pub id: Uuid,
    pub acct: String,
    pub username: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub locked: bool,
    pub note: String,
    pub url: String,
    pub avatar: String,
    pub avatar_static: String,
    pub header: Option<String>,
    pub header_static: Option<String>,
    pub followers_count: u64,
    pub following_count: u64,
    pub statuses_count: u64,
    pub source: Source,
}
