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
pub struct StatusEvent {
    pub r#type: EventType,
    pub status_id: Uuid,
}
