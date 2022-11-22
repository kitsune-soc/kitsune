use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct Account {
    pub id: Uuid,
    pub acct: String,
    pub username: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub note: String,
    pub url: String,
    pub avatar: String,
    pub avatar_static: String,
    pub header: String,
    pub header_static: String,
    pub followers_count: u64,
    pub following_count: u64,
    pub statuses_count: u64,
}
