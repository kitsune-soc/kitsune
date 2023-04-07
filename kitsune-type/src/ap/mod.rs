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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum ActivityType {
    Accept,
    Announce,
    #[default]
    Create,
    Block,
    Delete,
    Follow,
    Like,
    Reject,
    Undo,
    Update,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub r#type: ActivityType,
    pub object: StringOrObject<Object>,
    #[serde(flatten)]
    pub rest: BaseObject,
}

impl Activity {
    pub fn object(&self) -> &str {
        match self.object {
            StringOrObject::Object(ref obj) => &obj.rest.id,
            StringOrObject::String(ref obj) => obj,
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
pub struct Object {
    pub r#type: ObjectType,
    pub summary: Option<String>,
    pub content: String,
    pub attachment: Vec<MediaAttachment>,
    pub tag: Vec<Tag>,
    pub url: Option<String>,
    #[serde(flatten)]
    pub rest: BaseObject,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseObject {
    #[serde(rename = "@context")]
    pub context: Value,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributed_to: Option<StringOrObject<Box<Actor>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,
    #[serde(default)]
    pub sensitive: bool,
    pub published: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
}

impl BaseObject {
    pub fn attributed_to(&self) -> Option<&str> {
        self.attributed_to.as_ref().map(|prop| {
            match prop {
                StringOrObject::Object(actor) => &actor.rest.id,
                StringOrObject::String(id) => id,
            }
            .as_str()
        })
    }
}

impl Default for BaseObject {
    fn default() -> Self {
        Self {
            context: ap_context(),
            id: String::default(),
            attributed_to: Option::default(),
            in_reply_to: Option::default(),
            sensitive: bool::default(),
            published: Utc::now(),
            to: Vec::default(),
            cc: Vec::default(),
        }
    }
}
