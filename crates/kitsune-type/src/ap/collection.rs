use crate::jsonld;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use sonic_rs::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CollectionType {
    OrderedCollection,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    #[serde(default, rename = "@context")]
    pub context: Value,

    pub id: String,

    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub r#type: CollectionType,

    pub total_items: u64,

    pub first: Option<String>,

    pub last: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PageType {
    OrderedCollectionPage,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionPage<T> {
    #[serde(default, rename = "@context")]
    pub context: Value,

    pub id: String,

    #[serde_as(as = "jsonld::serde::FirstOk")]
    pub r#type: PageType,

    pub next: String,

    pub prev: String,

    pub part_of: String,

    pub ordered_items: Vec<T>,
}
