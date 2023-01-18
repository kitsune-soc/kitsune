use super::{Account, Status};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Deserialize, Serialize)]
pub struct SearchResult {
    pub accounts: Vec<Account>,
    pub hashtags: Vec<Value>, // Placeholder
    pub statuses: Vec<Status>,
}
