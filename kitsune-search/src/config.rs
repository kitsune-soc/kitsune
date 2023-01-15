use human_size::Size;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Deserialize)]
pub struct Configuration {
    pub index_dir_path: PathBuf,
    pub levenshtein_distance: u8,
    pub memory_arena_size: Size,
    pub port: u16,
    pub read_only: bool,
}
