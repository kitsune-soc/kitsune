use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Stats {
    pub user_count: u64,
    pub status_count: u64,
    pub domain_count: u64,
}

#[derive(Deserialize, Serialize)]
pub struct Urls {
    pub streaming_api: String,
}

#[derive(Deserialize, Serialize)]
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
