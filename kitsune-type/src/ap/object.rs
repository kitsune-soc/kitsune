use super::BaseObject;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum MediaAttachmentType {
    Audio,
    #[default]
    Document,
    Image,
    Video,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAttachment {
    pub r#type: MediaAttachmentType,
    pub name: Option<String>,
    pub media_type: String,
    pub blurhash: Option<String>,
    pub url: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    pub name: Option<String>,
    pub preferred_username: String,
    pub subject: Option<String>,
    pub icon: Option<MediaAttachment>,
    pub image: Option<MediaAttachment>,
    #[serde(flatten)]
    pub rest: BaseObject,
    #[serde(default)]
    pub manually_approves_followers: bool,
    pub public_key: PublicKey,
    pub inbox: String,
    pub outbox: String,
    pub followers: String,
    pub following: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    pub id: String,
    pub owner: String,
    pub public_key_pem: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub summary: Option<String>,
    pub content: String,
    #[serde(flatten)]
    pub rest: BaseObject,
}
