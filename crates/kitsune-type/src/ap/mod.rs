use self::{actor::Actor, object::MediaAttachment};
use crate::jsonld::{self, RdfNode};
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use simd_json::{json, OwnedValue};

pub const PUBLIC_IDENTIFIER: &str = "https://www.w3.org/ns/activitystreams#Public";

pub mod actor;
pub mod collection;
pub mod emoji;
pub mod helper;
pub mod object;

pub use self::helper::Privacy;

#[must_use]
pub fn ap_context() -> OwnedValue {
    json!([
        "https://www.w3.org/ns/activitystreams",
        "https://w3id.org/security/v1",
        {
            "Hashtag": "as:Hashtag",
            "sensitive": "as:sensitive",
            "schema": "http://schema.org/",
            "toot": "http://joinmastodon.org/ns#",
            "Emoji": "toot:Emoji",
            "PropertyValue": "schema:PropertyValue",
            "manuallyApprovesFollowers": "as:manuallyApprovesFollowers",
            "value": "schema:value",
            "quoteUrl": "as:quoteUrl",
        },
    ])
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ActivityType {
    Accept,
    Announce,
    Create,
    Block,
    Delete,
    Follow,
    Like,
    Reject,
    Undo,
    Update,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ObjectField {
    Activity(Box<Activity>),
    Actor(Actor),
    Object(Object),
    Url(String),
    // We really just need the ID from a tombstone object.
    // These are used by, for example, Mastodon in the object field of `Delete` activities.
    // So we just hack this in as the last possible case.
    Tombstone { id: String },
}

impl ObjectField {
    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Activity(ref activity) => &activity.id,
            Self::Actor(ref actor) => &actor.id,
            Self::Object(ref object) => &object.id,
            Self::Url(ref url) => url,
            Self::Tombstone { ref id } => id,
        }
    }

    #[must_use]
    pub fn into_activity(self) -> Option<Box<Activity>> {
        if let Self::Activity(activity) = self {
            Some(activity)
        } else {
            None
        }
    }

    #[must_use]
    pub fn into_actor(self) -> Option<Actor> {
        if let Self::Actor(actor) = self {
            Some(actor)
        } else {
            None
        }
    }

    #[must_use]
    pub fn into_object(self) -> Option<Object> {
        if let Self::Object(object) = self {
            Some(object)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    #[serde(default, rename = "@context")]
    pub context: OwnedValue,
    pub id: String,
    #[serde(deserialize_with = "jsonld::serde::FirstOk::deserialize")]
    pub r#type: ActivityType,
    #[serde(deserialize_with = "jsonld::serde::FirstId::deserialize")]
    pub actor: String,
    pub object: ObjectField,
    #[serde(default = "Timestamp::now_utc")]
    pub published: Timestamp,
}

impl Activity {
    #[must_use]
    pub fn object(&self) -> &str {
        match self.object {
            ObjectField::Activity(ref activity) => activity.id.as_str(),
            ObjectField::Actor(ref actor) => actor.id.as_str(),
            ObjectField::Object(ref object) => object.id.as_str(),
            ObjectField::Url(ref url) => url,
            ObjectField::Tombstone { ref id } => id,
        }
    }
}

impl RdfNode for Activity {
    fn id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ObjectType {
    Article,
    Image,
    Note,
    Page,
    Video,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    #[serde(default, rename = "@context")]
    pub context: OwnedValue,
    pub id: String,
    #[serde(deserialize_with = "jsonld::serde::FirstOk::deserialize")]
    pub r#type: ObjectType,
    #[serde(deserialize_with = "jsonld::serde::FirstId::deserialize")]
    pub attributed_to: String,
    #[serde(default)]
    #[serde(
        deserialize_with = "jsonld::serde::Optional::<jsonld::serde::FirstId<_>>::deserialize"
    )]
    pub in_reply_to: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::Optional::<jsonld::serde::First<_>>::deserialize")]
    pub name: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::Optional::<jsonld::serde::First<_>>::deserialize")]
    pub summary: Option<String>,
    #[serde(deserialize_with = "jsonld::serde::First::deserialize")]
    pub content: String,
    pub media_type: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::Set::deserialize")]
    pub attachment: Vec<MediaAttachment>,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::Set::deserialize")]
    pub tag: Vec<Tag>,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::First::deserialize")]
    pub sensitive: bool,
    pub published: Timestamp,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::IdSet::deserialize")]
    pub to: Vec<String>,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::IdSet::deserialize")]
    pub cc: Vec<String>,
}

impl RdfNode for Object {
    fn id(&self) -> Option<&str> {
        Some(&self.id)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum TagType {
    Emoji,
    Hashtag,
    Mention,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tag {
    pub id: Option<String>,
    #[serde(deserialize_with = "jsonld::serde::FirstOk::deserialize")]
    pub r#type: TagType,
    #[serde(deserialize_with = "jsonld::serde::First::deserialize")]
    pub name: String,
    pub href: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "jsonld::serde::Optional::<jsonld::serde::First<_>>::deserialize")]
    pub icon: Option<MediaAttachment>,
}
