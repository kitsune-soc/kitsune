use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Create,
    Delete,
    Update,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct PostEvent {
    pub r#type: EventType,
    pub post_id: Uuid,
}
