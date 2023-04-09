use self::{
    helper::StringOrObject,
    object::{Actor, MediaAttachment},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const PUBLIC_IDENTIFIER: &str = "https://www.w3.org/ns/activitystreams#Public";

pub mod collection;
pub mod helper;
pub mod object;

pub use self::helper::Privacy;

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
}

impl ObjectField {
    pub fn id(&self) -> &str {
        match self {
            Self::Activity(ref activity) => &activity.id,
            Self::Actor(ref actor) => &actor.id,
            Self::Object(ref object) => &object.id,
            Self::Url(ref url) => url,
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
    #[serde(rename = "@context")]
    pub context: Value,
    pub id: String,
    pub r#type: ActivityType,
    pub actor: StringOrObject<Actor>,
    pub object: ObjectField,
    #[serde(default)]
    pub published: DateTime<Utc>,
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
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ObjectType {
    Article,
    Image,
    Note,
    Video,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    #[serde(default, rename = "@context")]
    pub context: Value,
    pub id: String,
    pub r#type: ObjectType,
    pub attributed_to: StringOrObject<Actor>,
    pub in_reply_to: Option<String>,
    pub summary: Option<String>,
    pub content: String,
    pub attachment: Vec<MediaAttachment>,
    pub tag: Vec<Tag>,
    #[serde(default)]
    pub sensitive: bool,
    pub published: DateTime<Utc>,
    #[serde(default)]
    pub to: Vec<String>,
    #[serde(default)]
    pub cc: Vec<String>,
}

impl Object {
    pub fn attributed_to(&self) -> &str {
        match self.attributed_to {
            StringOrObject::Object(ref actor) => &actor.id,
            StringOrObject::String(ref id) => id,
        }
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
    pub r#type: TagType,
    pub name: String,
    pub href: Option<String>,
    pub icon: Option<MediaAttachment>,
}
