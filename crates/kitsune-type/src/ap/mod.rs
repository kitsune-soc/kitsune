use self::{
    actor::{Actor, ActorType},
    helper::StringOrObject,
    object::MediaAttachment,
};
use crate::jsonld::RdfNode;
use iso8601_timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use simd_json::{json, OwnedValue};

pub const PUBLIC_IDENTIFIER: &str = "https://www.w3.org/ns/activitystreams#Public";

pub mod actor;
pub mod collection;
pub mod helper;
pub mod object;

pub use self::helper::Privacy;

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
pub struct AttributedToListEntry {
    pub r#type: ActorType,
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AttributedToField {
    Actor(Actor),
    Url(String),
    List(Vec<AttributedToListEntry>),
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
    pub fn id(&self) -> &str {
        match self {
            Self::Activity(ref activity) => &activity.id,
            Self::Actor(ref actor) => &actor.id,
            Self::Object(ref object) => &object.id,
            Self::Url(ref url) => url,
            Self::Tombstone { ref id } => id,
        }
    }

    pub fn into_activity(self) -> Option<Box<Activity>> {
        if let Self::Activity(activity) = self {
            Some(activity)
        } else {
            None
        }
    }

    pub fn into_actor(self) -> Option<Actor> {
        if let Self::Actor(actor) = self {
            Some(actor)
        } else {
            None
        }
    }

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
    pub r#type: ActivityType,
    pub actor: StringOrObject<Actor>,
    pub object: ObjectField,
    #[serde(default = "Timestamp::now_utc")]
    pub published: Timestamp,
}

impl Activity {
    pub fn actor(&self) -> &str {
        match self.actor {
            StringOrObject::Object(ref obj) => &obj.id,
            StringOrObject::String(ref url) => url,
        }
    }

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
    pub r#type: ObjectType,
    pub attributed_to: AttributedToField,
    pub in_reply_to: Option<String>,
    pub name: Option<String>,
    pub summary: Option<String>,
    pub content: String,
    pub media_type: Option<String>,
    #[serde(default)]
    pub attachment: Vec<MediaAttachment>,
    #[serde(default)]
    pub tag: Vec<Tag>,
    #[serde(default)]
    pub sensitive: bool,
    pub published: Timestamp,
    #[serde(default)]
    pub to: Vec<String>,
    #[serde(default)]
    pub cc: Vec<String>,
}

impl Object {
    pub fn attributed_to(&self) -> Option<&str> {
        match self.attributed_to {
            AttributedToField::Actor(ref actor) => Some(&actor.id),
            AttributedToField::Url(ref url) => Some(url),
            AttributedToField::List(ref list) => list.iter().map(|item| item.id.as_str()).next(),
        }
    }
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
    pub r#type: TagType,
    pub name: String,
    pub href: Option<String>,
    pub icon: Option<MediaAttachment>,
}
