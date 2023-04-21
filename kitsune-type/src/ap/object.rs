use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MediaAttachmentType {
    Audio,
    Document,
    Image,
    Video,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAttachment {
    pub r#type: MediaAttachmentType,
    pub name: Option<String>,
    pub media_type: String,
    pub blurhash: Option<String>,
    pub url: String,
}
