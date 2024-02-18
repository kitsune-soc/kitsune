use crate::jsonld;
use serde::{Deserialize, Serialize};
use simd_json::OwnedValue;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CollectionType {
    OrderedCollection,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    #[serde(default, rename = "@context")]
    pub context: OwnedValue,
    pub id: String,
    #[serde(deserialize_with = "jsonld::serde::FirstOk::deserialize")]
    pub r#type: CollectionType,
    pub total_items: u64,
    pub first: Option<String>,
    pub last: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PageType {
    OrderedCollectionPage,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionPage<T> {
    #[serde(default, rename = "@context")]
    pub context: OwnedValue,
    pub id: String,
    #[serde(deserialize_with = "jsonld::serde::FirstOk::deserialize")]
    pub r#type: PageType,
    pub next: String,
    pub prev: String,
    pub part_of: String,
    pub ordered_items: Vec<T>,
}
