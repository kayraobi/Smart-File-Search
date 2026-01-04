use crate::models::{SearchOutput, FileResult, DEFAULT_SEARCH_LIMIT};
use crate::db::Database;
use crate::embedding::EmbeddingModel;
use std::env;

pub struct AppContext {
    db: Database,
    model: EmbeddingModel,
}

impl AppContext {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = env::temp_dir().join("file_index.db");
        let db = Database::new(db_path.to_str().unwrap())?;
        let model = EmbeddingModel::new()?;
        Ok(Self { db, model })
    }

    pub fn db(&self) -> &Database {
        &self.db
    }

    pub fn model_mut(&mut self) -> &mut EmbeddingModel {
        &mut self.model
    }

    pub fn search(&mut self, query: &str) -> Result<SearchOutput, Box<dyn std::error::Error>> {
        let query_embedding = self.model.embed_one(query)?;
        let results = self.db.search_similar(&query_embedding, DEFAULT_SEARCH_LIMIT)?;

        let file_results = results.iter().rev().map(|(name, path, distance)| FileResult {
            file_name: name.clone(),
            path: path.clone(),
            distance: *distance,
        }).collect();

        Ok(SearchOutput {
            search: query.to_string(),
            results: file_results,
        })
    }
}
