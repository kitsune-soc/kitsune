use super::object::MediaAttachment;
use crate::jsonld::RdfNode;
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sonic_rs::Value;

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Emoji {
    #[serde(default, rename = "@context")]
    pub context: Value,
    pub id: String,
    #[serde_with(as = "jsonld::serde::FirstOk")]
    pub r#type: String,
    #[serde_with(as = "jsonld::serde::First")]
    pub name: String,
    #[serde_with(as = "jsonld::serde::First")]
    pub icon: MediaAttachment,
    #[serde(default = "Timestamp::now_utc")]
    pub updated: Timestamp,
}

impl RdfNode for Emoji {
    fn id(&self) -> Option<&str> {
        Some(&self.id)
    }
}
