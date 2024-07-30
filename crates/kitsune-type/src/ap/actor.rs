use super::object::MediaAttachment;
use crate::jsonld::{self, RdfNode};
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sonic_rs::Value;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ActorType {
    Group,
    Person,
    Service,
}

impl ActorType {
    #[must_use]
    pub fn is_bot(&self) -> bool {
        matches!(self, Self::Service)
    }

    #[must_use]
    pub fn is_group(&self) -> bool {
        matches!(self, Self::Group)
    }
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    #[serde(default, rename = "@context")]
    pub context: Value,
    pub id: String,
    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub r#type: ActorType,
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First>")]
    pub name: Option<String>,
    #[serde_as(as = "jsonld::serde::First")]
    pub preferred_username: String,
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First>")]
    pub subject: Option<String>,
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First>")]
    pub icon: Option<MediaAttachment>,
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First>")]
    pub image: Option<MediaAttachment>,
    #[serde(default)]
    #[serde_as(as = "jsonld::serde::First")]
    pub manually_approves_followers: bool,
    #[serde_as(as = "jsonld::serde::First")]
    pub public_key: PublicKey,
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First>")]
    pub endpoints: Option<Endpoints>,
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First<jsonld::serde::Id>>")]
    pub featured: Option<String>,
    #[serde_as(as = "jsonld::serde::First")]
    pub inbox: String,
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First<jsonld::serde::Id>>")]
    pub outbox: Option<String>,
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First<jsonld::serde::Id>>")]
    pub followers: Option<String>,
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First<jsonld::serde::Id>>")]
    pub following: Option<String>,
    #[serde(default = "Timestamp::now_utc")]
    pub published: Timestamp,
}

impl RdfNode for Actor {
    fn id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First<jsonld::serde::Id>>")]
    pub shared_inbox: Option<String>,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    pub id: String,
    #[serde_as(as = "jsonld::serde::First<jsonld::serde::Id>")]
    pub owner: String,
    #[serde_as(as = "jsonld::serde::First")]
    pub public_key_pem: String,
}
