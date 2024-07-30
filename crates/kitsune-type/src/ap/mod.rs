use self::{actor::Actor, object::MediaAttachment};
use crate::jsonld::{self, RdfNode};
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnNull, OneOrMany};
use sonic_rs::{json, Value};
use strum::AsRefStr;

pub const PUBLIC_IDENTIFIER: &str = "https://www.w3.org/ns/activitystreams#Public";

pub mod actor;
pub mod collection;
pub mod emoji;
pub mod helper;
pub mod object;

pub use self::helper::Privacy;

#[must_use]
pub fn ap_context() -> Value {
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

#[derive(AsRefStr, Clone, Debug, Deserialize, Serialize)]
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

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    #[serde(default, rename = "@context")]
    pub context: Value,
    pub id: String,
    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub r#type: ActivityType,
    #[serde_as(as = "jsonld::serde::First<jsonld::serde::Id>")]
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

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    #[serde(default, rename = "@context")]
    pub context: Value,
    pub id: String,
    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub r#type: ObjectType,
    #[serde_as(as = "jsonld::serde::First<jsonld::serde::Id>")]
    pub attributed_to: String,
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First<jsonld::serde::Id>>")]
    pub in_reply_to: Option<String>,
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First>")]
    pub name: Option<String>,
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First>")]
    pub summary: Option<String>,
    #[serde_as(as = "jsonld::serde::First")]
    pub content: String,
    pub media_type: Option<String>,
    #[serde(default)]
    #[serde_as(as = "OneOrMany<_>")]
    pub attachment: Vec<MediaAttachment>,
    #[serde(default)]
    #[serde_as(as = "OneOrMany<_>")]
    pub tag: Vec<Tag>,
    #[serde(default)]
    #[serde_as(as = "DefaultOnNull<jsonld::serde::First>")]
    pub sensitive: bool,
    pub published: Timestamp,
    #[serde(default)]
    #[serde_as(as = "OneOrMany<jsonld::serde::Id>")]
    pub to: Vec<String>,
    #[serde(default)]
    #[serde_as(as = "OneOrMany<jsonld::serde::Id>")]
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

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tag {
    pub id: Option<String>,
    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub r#type: TagType,
    #[serde_as(as = "jsonld::serde::First")]
    pub name: String,
    pub href: Option<String>,
    #[serde(default)]
    #[serde_as(as = "Option<jsonld::serde::First>")]
    pub icon: Option<MediaAttachment>,
}
