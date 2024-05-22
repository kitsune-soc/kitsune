use super::{Account, Status};
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use speedy_uuid::Uuid;

#[derive(Copy, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    Mention,
    Status,
    Reblog,
    Follow,
    FollowRequest,
    Favourite,
    Update,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Notification {
    pub id: Uuid,
    pub r#type: NotificationType,
    pub created_at: Timestamp,
    pub account: Account,
    pub status: Option<Status>,
}
