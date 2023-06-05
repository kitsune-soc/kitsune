use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MediaAttachmentType {
    #[serde(alias = "audio")] // Some implementations don't care about the casing here. Idk why??
    Audio,
    #[serde(alias = "document")]
    Document,
    #[serde(alias = "image")]
    Image,
    #[serde(alias = "video")]
    Video,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAttachment {
    pub r#type: MediaAttachmentType,
    pub name: Option<String>,
    pub media_type: Option<String>,
    pub blurhash: Option<String>,
    pub url: String,
}
