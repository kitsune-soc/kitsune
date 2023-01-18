//!
//! Service configuration
//!

use human_size::Size;
use serde::Deserialize;
use std::path::PathBuf;

fn default_prometheus_port() -> u16 {
    9000
}

/// Configuration values
#[derive(Clone, Deserialize)]
pub struct Configuration {
    /// Path to the directory in which the indices are created
    pub index_dir_path: PathBuf,

    /// Levenshtein distance used when executing the fuzzy search
    ///
    /// It basically tells the search engine how lenient it should be with matching queries
    pub levenshtein_distance: u8,

    /// Size of the memory arena allocated by the writers
    ///
    /// Every time a writer fills its memory arena, the operations are flushed to disk
    pub memory_arena_size: Size,

    /// Port on which the gRPC server is listening on
    pub port: u16,

    /// Port on which the Prometheus exporter is listening on
    ///
    /// Defaults to 9000
    #[serde(default = "default_prometheus_port")]
    pub prometheus_port: u16,

    /// Run this node in read-only mode
    ///
    /// Every index can only have one writer (indexer), so set this to true on all secondary nodes
    pub read_only: bool,
}
