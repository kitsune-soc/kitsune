use crate::jsonld;
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
    #[serde(deserialize_with = "jsonld::serde::FirstOk::deserialize")]
    pub r#type: MediaAttachmentType,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::Optional::<jsonld::serde::First<_>>::deserialize")]
    pub name: Option<String>,
    pub media_type: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::Optional::<jsonld::serde::First<_>>::deserialize")]
    pub blurhash: Option<String>,
    #[serde(deserialize_with = "jsonld::serde::First::deserialize")]
    pub url: String,
}
