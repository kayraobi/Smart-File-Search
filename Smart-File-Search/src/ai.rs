use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};

pub struct AiHandler {
    model: TextEmbedding,
}

impl AiHandler {
    pub fn new() -> anyhow::Result<Self> {
        let mut options = InitOptions::default();
        options.model_name = EmbeddingModel::AllMiniLML6V2;
        options.show_download_progress = true;

        let model = TextEmbedding::try_new(options)?;
        Ok(Self { model })
    }

    pub fn get_embedding(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let result = self.model.embed(vec![text], None)?;
        Ok(result[0].clone())
    }

    pub fn get_embeddings_batch(&self, texts: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>> {
        let results = self.model.embed(texts, None)?;
        Ok(results)
    }

    pub fn similarity_score(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        let dot_product: f32 = vec1.iter().zip(vec2).map(|(x, y)| x * y).sum();
        let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot_product / (norm1 * norm2)
    }
}