use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

#[derive(Clone, Deserialize, Serialize)]
pub struct Configuration {
    pub database_url: String,
    pub domain: String,
    pub job_workers: NonZeroUsize,
    pub port: u16,
}
