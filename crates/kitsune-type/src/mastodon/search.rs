use super::{Account, Status};
use serde::{Deserialize, Serialize};
use simd_json::OwnedValue;

#[derive(Default, Deserialize, Serialize)]
pub struct SearchResult {
    pub accounts: Vec<Account>,
    pub hashtags: Vec<OwnedValue>, // Placeholder
    pub statuses: Vec<Status>,
}
