use super::{Account, Status};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct SearchResult {
    pub accounts: Vec<Account>,
    pub statuses: Vec<Status>,
}
