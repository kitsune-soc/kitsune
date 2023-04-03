use super::{Account, Status};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Default, Deserialize, Serialize, ToSchema)]
pub struct SearchResult {
    pub accounts: Vec<Account>,
    #[schema(value_type = Object)]
    pub hashtags: Vec<Value>, // Placeholder
    pub statuses: Vec<Status>,
}
