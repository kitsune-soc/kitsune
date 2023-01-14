use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Deserialize)]
pub struct Configuration {
    pub index_dir_path: PathBuf,
    pub memory_arena_size: usize,
    pub port: u16,
}
