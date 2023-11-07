use super::object::MediaAttachment;
use crate::jsonld::RdfNode;
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use simd_json::OwnedValue;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Emoji {
    #[serde(default, rename = "@context")]
    pub context: OwnedValue,
    pub id: String,
    pub r#type: String,
    pub name: String,
    pub icon: MediaAttachment,
    #[serde(default = "Timestamp::now_utc")]
    pub updated: Timestamp,
}

impl RdfNode for Emoji {
    fn id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
