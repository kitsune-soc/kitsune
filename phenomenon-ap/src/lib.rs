use self::{helper::StringOrObject, object::Actor};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const PUBLIC_IDENTIFIER: &str = "https://www.w3.org/ns/activitystreams#Public";

pub mod helper;
pub mod object;

pub fn ap_context() -> Value {
    json!("https://www.w3.org/ns/activitystreams")
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub object: StringOrObject<Object>,
    #[serde(flatten)]
    pub rest: Object,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    #[serde(rename = "@context")]
    pub context: Value,
    pub id: String,
    pub r#type: String,
    pub attributed_to: Option<Box<StringOrObject<Actor>>>,
    pub published_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
}

impl Object {
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

impl Default for Object {
    fn default() -> Self {
        Self {
            context: ap_context(),
            id: String::new(),
            r#type: String::new(),
            attributed_to: None,
            published_at: Utc::now(),
            to: Vec::new(),
            cc: Vec::new(),
        }
    }
}
