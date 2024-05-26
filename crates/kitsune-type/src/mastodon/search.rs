use super::{Account, Status};
use serde::{Deserialize, Serialize};
use sonic_rs::Value;

#[derive(Default, Deserialize, Serialize)]
pub struct SearchResult {
    pub accounts: Vec<Account>,
    pub hashtags: Vec<Value>, // Placeholder
    pub statuses: Vec<Status>,
}
