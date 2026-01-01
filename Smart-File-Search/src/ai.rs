use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};

pub struct AiHandler {
    model: TextEmbedding,
}

impl AiHandler {
    pub fn new() -> anyhow::Result<Self> {
        let mut options = InitOptions::default();
        options.model_name = EmbeddingModel::AllMiniLML6V2;
        options.show_download_progress = true;

        let model_result = TextEmbedding::try_new(options);

        match model_result {
            Ok(loaded_model) => {
                Ok(Self { model: loaded_model })
            },
            Err(error) => {
                eprintln!("Error loading model!");
                // DÜZELTME: anyhow::Error::new() kaldirildi.
                // Zaten 'error' bir anyhow::Error oldugu icin direkt gonderiyoruz.
                Err(error)
            }
        }
    }

    pub fn get_embedding(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let result = self.model.embed(vec![text], None);

        match result {
            Ok(embeddings) => {
                Ok(embeddings[0].clone())
            },
            Err(e) => {
                // DÜZELTME: Direkt hatayi donuyoruz
                Err(e)
            },
        }
    }

    pub fn get_embeddings_batch(&self, texts: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>> {
        let result = self.model.embed(texts, None);
        match result {
            Ok(embeddings) => Ok(embeddings),
            Err(e) => {
                // DÜZELTME: Direkt hatayi donuyoruz
                Err(e)
            },
        }
    }

    pub fn similarity_score(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        let mut dot_product: f32 = 0.0;
        let mut sum_sq1: f32 = 0.0;
        let mut sum_sq2: f32 = 0.0;

        for (v1, v2) in vec1.iter().zip(vec2.iter()) {
            dot_product += v1 * v2;
            sum_sq1 += v1 * v1;
            sum_sq2 += v2 * v2;
        }

        let norm1 = sum_sq1.sqrt();
        let norm2 = sum_sq2.sqrt();

        return dot_product / (norm1 * norm2);
    }
}