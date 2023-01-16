use serde::{Deserialize, Serialize};
use std::{num::NonZeroUsize, path::PathBuf};

fn default_frontend_dir() -> PathBuf {
    "kitsune-fe/dist".into()
}

fn default_upload_dir() -> PathBuf {
    "uploads".into()
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Configuration {
    pub database_url: String,
    pub domain: String,
    #[serde(default = "default_frontend_dir")]
    pub frontend_dir: PathBuf,
    pub job_workers: NonZeroUsize,
    pub port: u16,
    pub redis_url: String,
    pub search_servers: Vec<String>,
    pub search_index_server: String,
    #[serde(default = "default_upload_dir")]
    pub upload_dir: PathBuf,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            database_url: String::default(),
            domain: String::default(),
            frontend_dir: PathBuf::default(),
            job_workers: NonZeroUsize::new(1).unwrap(),
            port: u16::default(),
            redis_url: String::default(),
            search_servers: Vec::default(),
            search_index_server: String::default(),
            upload_dir: PathBuf::default(),
        }
    }
}
