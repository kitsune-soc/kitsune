use crate::jsonld;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MediaAttachmentType {
    Audio,
    Document,
    Image,
    Link,
    Video,

    #[serde(other)]
    Other,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAttachment {
    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub r#type: MediaAttachmentType,
    #[serde_as(as = "Option<jsonld::serde::First>")]
    pub name: Option<String>,
    pub media_type: Option<String>,
    #[serde_as(as = "Option<jsonld::serde::First>")]
    pub blurhash: Option<String>,

    #[serde(alias = "href")]
    #[serde_as(as = "jsonld::serde::First")]
    pub url: String,
}
