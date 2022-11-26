use super::{object::Note, Activity, Object, PUBLIC_IDENTIFIER};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StringOrObject<T> {
    String(String),
    Object(T),
}

impl<T> StringOrObject<T> {
    pub fn into_string(self) -> Option<String> {
        match self {
            Self::String(str) => Some(str),
            Self::Object(..) => None,
        }
    }

    pub fn into_object(self) -> Option<T> {
        match self {
            Self::String(..) => None,
            Self::Object(obj) => Some(obj),
        }
    }
}

impl<T> Default for StringOrObject<T> {
    fn default() -> Self {
        Self::String(String::new())
    }
}

pub trait Privacy {
    fn is_public(&self) -> bool;
    fn is_unlisted(&self) -> bool;

    fn is_private(&self) -> bool {
        !self.is_public() && !self.is_unlisted()
    }
}

impl Privacy for Activity {
    fn is_public(&self) -> bool {
        self.rest.is_public()
    }

    fn is_unlisted(&self) -> bool {
        self.rest.is_unlisted()
    }
}

impl Privacy for Note {
    fn is_public(&self) -> bool {
        self.rest.is_public()
    }

    fn is_unlisted(&self) -> bool {
        self.rest.is_unlisted()
    }
}

impl Privacy for Object {
    fn is_public(&self) -> bool {
        self.to.iter().any(|url| url == PUBLIC_IDENTIFIER)
    }

    fn is_unlisted(&self) -> bool {
        !self.is_public() && self.cc.iter().any(|url| url == PUBLIC_IDENTIFIER)
    }
}
