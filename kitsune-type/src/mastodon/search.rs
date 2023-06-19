use super::{Account, Status};
use serde::{Deserialize, Serialize};
use simd_json::OwnedValue;
use utoipa::ToSchema;

#[derive(Default, Deserialize, Serialize, ToSchema)]
pub struct SearchResult {
    pub accounts: Vec<Account>,
    #[schema(value_type = Object)]
    pub hashtags: Vec<OwnedValue>, // Placeholder
    pub statuses: Vec<Status>,
}
