use super::object::MediaAttachment;
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ActorType {
    Group,
    Person,
    Service,
}

impl ActorType {
    pub fn is_bot(&self) -> bool {
        matches!(self, Self::Service)
    }

    pub fn is_group(&self) -> bool {
        matches!(self, Self::Group)
    }
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
    pub endpoints: Option<Endpoints>,
    pub featured: Option<String>,
    pub inbox: String,
    pub outbox: Option<String>,
    pub followers: Option<String>,
    pub following: Option<String>,
    #[serde(default = "Timestamp::now_utc")]
    pub published: Timestamp,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
    pub shared_inbox: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    pub id: String,
    pub owner: String,
    pub public_key_pem: String,
}
