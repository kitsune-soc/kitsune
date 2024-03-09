use super::object::MediaAttachment;
use crate::jsonld::{self, RdfNode};
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use simd_json::OwnedValue;

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    #[serde(default, rename = "@context")]
    pub context: OwnedValue,
    pub id: String,
    #[serde(deserialize_with = "jsonld::serde::FirstOk::deserialize")]
    pub r#type: ActorType,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::Optional::<jsonld::serde::First<_>>::deserialize")]
    pub name: Option<String>,
    #[serde(deserialize_with = "jsonld::serde::First::deserialize")]
    pub preferred_username: String,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::Optional::<jsonld::serde::First<_>>::deserialize")]
    pub subject: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::Optional::<jsonld::serde::First<_>>::deserialize")]
    pub icon: Option<MediaAttachment>,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::Optional::<jsonld::serde::First<_>>::deserialize")]
    pub image: Option<MediaAttachment>,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::First::deserialize")]
    pub manually_approves_followers: bool,
    #[serde(deserialize_with = "jsonld::serde::First::deserialize")]
    pub public_key: PublicKey,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::Optional::<jsonld::serde::First<_>>::deserialize")]
    pub endpoints: Option<Endpoints>,
    #[serde(default)]
    #[serde(
        deserialize_with = "jsonld::serde::Optional::<jsonld::serde::FirstId<_>>::deserialize"
    )]
    pub featured: Option<String>,
    #[serde(deserialize_with = "jsonld::serde::FirstId::deserialize")]
    pub inbox: String,
    #[serde(default)]
    #[serde(
        deserialize_with = "jsonld::serde::Optional::<jsonld::serde::FirstId<_>>::deserialize"
    )]
    pub outbox: Option<String>,
    #[serde(default)]
    #[serde(
        deserialize_with = "jsonld::serde::Optional::<jsonld::serde::FirstId<_>>::deserialize"
    )]
    pub followers: Option<String>,
    #[serde(default)]
    #[serde(
        deserialize_with = "jsonld::serde::Optional::<jsonld::serde::FirstId<_>>::deserialize"
    )]
    pub following: Option<String>,
    #[serde(default = "Timestamp::now_utc")]
    pub published: Timestamp,
}

impl RdfNode for Actor {
    fn id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
    #[serde(default)]
    #[serde(
        deserialize_with = "jsonld::serde::Optional::<jsonld::serde::FirstId<_>>::deserialize"
    )]
    pub shared_inbox: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    pub id: String,
    #[serde(deserialize_with = "jsonld::serde::FirstId::deserialize")]
    pub owner: String,
    #[serde(deserialize_with = "jsonld::serde::First::deserialize")]
    pub public_key_pem: String,
}
