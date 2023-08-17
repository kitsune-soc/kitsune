use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use utoipa::ToSchema;

#[derive(Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PreviewType {
    Link,
    Photo,
    Video,
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct PreviewCard {
    pub url: String,
    pub title: SmolStr,
    pub description: SmolStr,
    pub r#type: PreviewType,
    pub author_name: SmolStr,
    pub author_url: SmolStr,
    pub provider_name: SmolStr,
    pub provider_url: SmolStr,
    pub html: String,
    pub width: i32,
    pub height: i32,
    pub image: Option<SmolStr>,
    pub embed_url: SmolStr,
}
