pub mod context;
pub mod db;
pub mod embedding;
pub mod models;
pub mod scanner;

pub use context::AppContext;
pub use db::Database;
pub use embedding::EmbeddingModel;
pub use models::{FileMetadata, SearchResult};
pub use scanner::{find_important_files, is_hidden};
