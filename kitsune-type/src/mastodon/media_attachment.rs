use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaType {
    Unknown,
    Image,
    Gifv,
    Video,
    Audio,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MediaAttachment {
    pub id: Uuid,
    pub r#type: MediaType,
    pub url: String,
    pub preview_url: String,
    pub remote_url: String,
    pub description: String,
    pub blurhash: Option<String>,
}
