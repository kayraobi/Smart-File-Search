use serde::Serialize;
use std::{fmt::Debug, time::SystemTime, path::PathBuf};

pub const EMBEDDING_DIM: usize = 384;
pub const DEFAULT_SEARCH_LIMIT: usize = 5;

pub struct FileMetadata {
    pub file_name: String,
    pub path: PathBuf,
    pub created: SystemTime,
    pub len: u64,
}

impl Debug for FileMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} in \t{}", self.file_name, self.path.display())
    }
}

pub type SearchResult = (String, String, f64);

#[derive(Serialize)]
pub struct SearchOutput {
    pub search: String,
    pub results: Vec<FileResult>,
}

#[derive(Serialize)]
pub struct FileResult {
    pub file_name: String,
    pub path: String,
    pub distance: f64,
}

pub struct DbStats {
    pub db_path: String,
    pub file_count: i64,
    pub db_size_bytes: u64,
    pub embedding_dim: usize,
    pub last_updated: Option<i64>,
}
