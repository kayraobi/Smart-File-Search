use std::error::Error;

pub struct EmbeddingModel {
    model: fastembed::TextEmbedding,
}

impl EmbeddingModel {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let model = fastembed::TextEmbedding::try_new(Default::default())?;
        Ok(Self { model })
    }

    pub fn embed_batch(&mut self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
        self.model.embed(texts, None).map_err(|e| e.into())
    }

    pub fn embed_one(&mut self, text: &str) -> Result<Vec<f32>, Box<dyn Error>> {
        let embeddings = self.model.embed(vec![text], None)?;
        Ok(embeddings.into_iter().next().unwrap())
    }
}
