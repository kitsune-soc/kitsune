use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ActorType {
    Group,
    Person,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    #[serde(default, rename = "@context")]
    pub context: Value,
    pub id: String,
    pub r#type: ActorType,
    pub name: Option<String>,
    pub preferred_username: String,
    pub subject: Option<String>,
    pub icon: Option<MediaAttachment>,
    pub image: Option<MediaAttachment>,
    #[serde(default)]
    pub manually_approves_followers: bool,
    pub public_key: PublicKey,
    pub inbox: String,
    pub outbox: String,
    pub followers: String,
    pub following: String,
    #[serde(with = "time::serde::rfc3339")]
    pub published: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    pub id: String,
    pub owner: String,
    pub public_key_pem: String,
}
