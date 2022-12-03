use self::{
    helper::StringOrObject,
    object::{Actor, Note},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const PUBLIC_IDENTIFIER: &str = "https://www.w3.org/ns/activitystreams#Public";

pub mod helper;
pub mod object;

pub use self::helper::Privacy;

pub fn ap_context() -> Value {
    json!("https://www.w3.org/ns/activitystreams")
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
            StringOrObject::Object(ref obj) => obj.id(),
            StringOrObject::String(ref obj) => obj,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Object {
    Note(Note),
    Person(Actor),
}

impl Object {
    pub fn id(&self) -> &str {
        match self {
            Self::Note(note) => &note.rest.id,
            Self::Person(person) => &person.rest.id,
        }
    }

    pub fn cc(&self) -> &[String] {
        match self {
            Self::Note(ref note) => note.rest.cc.as_slice(),
            Self::Person(ref person) => person.rest.cc.as_slice(),
        }
    }

    pub fn to(&self) -> &[String] {
        match self {
            Self::Note(ref note) => note.rest.to.as_slice(),
            Self::Person(ref person) => person.rest.to.as_slice(),
        }
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::Note(Note::default())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseObject {
    #[serde(rename = "@context")]
    pub context: Value,
    pub id: String,
    pub attributed_to: Option<Box<StringOrObject<Actor>>>,
    pub published_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
}

impl BaseObject {
    pub fn attributed_to(&self) -> Option<&str> {
        self.attributed_to.as_deref().map(|prop| {
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
            id: String::new(),
            attributed_to: None,
            published_at: Utc::now(),
            to: Vec::new(),
            cc: Vec::new(),
        }
    }
}
