use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum MediaType {
    Unknown,
    Image,
    Gifv,
    Video,
    Audio,
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct MediaAttachment {
    pub id: Uuid,
    pub r#type: MediaType,
    pub url: String,
    pub preview_url: String,
    pub remote_url: String,
    pub description: String,
    pub blurhash: Option<String>,
}
