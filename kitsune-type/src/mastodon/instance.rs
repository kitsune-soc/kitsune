use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Stats {
    pub user_count: u64,
    pub status_count: u64,
    pub domain_count: u64,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Urls {
    pub streaming_api: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Instance {
    pub uri: String,
    pub title: String,
    pub short_description: String,
    pub description: String,
    pub max_toot_chars: usize,
    pub email: String,
    pub version: String,
    pub urls: Urls,
    pub stats: Stats,
}
