use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;
use utoipa::ToSchema;

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct Field {
    pub name: String,
    pub value: String,
    pub verified_at: Option<Timestamp>,
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
    pub bot: bool,
    pub group: bool,
    pub username: String,
    pub display_name: String,
    pub created_at: Timestamp,
    pub locked: bool,
    pub note: String,
    pub url: String,
    pub avatar: String,
    pub avatar_static: String,
    pub header: String,
    pub header_static: String,
    pub followers_count: u64,
    pub following_count: u64,
    pub statuses_count: u64,
    pub source: Source,
}
