use crate::error::{CinemaError, Result};
use async_trait::async_trait;

/// Embedding model trait
#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    /// Get embedding dimension
    fn dimension(&self) -> usize;

    /// Generate embedding for text
    async fn embed(&self,
        text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts (batch)
    async fn embed_batch(
        &self,
        texts: Vec<String>,
    ) -> Result<Vec<Vec<f32>>> {
        let mut results = vec![];
        for text in texts {
            results.push(self.embed(&text).await?);
        }
        Ok(results)
    }
}

/// OpenAI embedding model
pub struct OpenAIEmbedder {
    client: reqwest::Client,
    api_key: String,
    model: String,
    dimension: usize,
}

impl OpenAIEmbedder {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "text-embedding-3-small".to_string(),
            dimension: 1536,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>, dimension: usize) -> Self {
        self.model = model.into();
        self.dimension = dimension;
        self
    }
}

#[async_trait]
impl EmbeddingModel for OpenAIEmbedder {
    fn dimension(&self) -> usize {
        self.dimension
    }

    async fn embed(&self,
        text: &str) -> Result<Vec<f32>> {
        #[derive(serde::Serialize)]
        struct Request {
            model: String,
            input: String,
        }

        #[derive(serde::Deserialize)]
        struct Response {
            data: Vec<EmbeddingData>,
        }

        #[derive(serde::Deserialize)]
        struct EmbeddingData {
            embedding: Vec<f32>,
        }

        let request = Request {
            model: self.model.clone(),
            input: text.to_string(),
        };

        let response: Response = self.client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| CinemaError::LlmApi(format!("Embedding request failed: {}", e)))?
            .json()
            .await
            .map_err(|e| CinemaError::LlmApi(format!("Failed to parse embedding: {}", e)))?;

        response.data.into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| CinemaError::LlmApi("No embedding returned".to_string()))
    }
}

/// Ollama embedding model (local)
pub struct OllamaEmbedder {
    client: reqwest::Client,
    base_url: String,
    model: String,
    dimension: usize,
}

impl OllamaEmbedder {
    pub fn new(model: impl Into<String>, dimension: usize) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "http://localhost:11434".to_string(),
            model: model.into(),
            dimension,
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
}

#[async_trait]
impl EmbeddingModel for OllamaEmbedder {
    fn dimension(&self) -> usize {
        self.dimension
    }

    async fn embed(&self,
        text: &str) -> Result<Vec<f32>> {
        #[derive(serde::Serialize)]
        struct Request {
            model: String,
            prompt: String,
        }

        #[derive(serde::Deserialize)]
        struct Response {
            embedding: Vec<f32>,
        }

        let request = Request {
            model: self.model.clone(),
            prompt: text.to_string(),
        };

        let response: Response = self.client
            .post(format!("{}/api/embeddings", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| CinemaError::LlmApi(format!("Ollama embedding failed: {}", e)))?
            .json()
            .await
            .map_err(|e| CinemaError::LlmApi(format!("Failed to parse embedding: {}", e)))?;

        Ok(response.embedding)
    }
}

/// Mock embedder for testing
pub struct MockEmbedder {
    dimension: usize,
}

impl MockEmbedder {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingModel for MockEmbedder {
    fn dimension(&self) -> usize {
        self.dimension
    }

    async fn embed(&self,
        _text: &str) -> Result<Vec<f32>> {
        // Generate deterministic mock embedding based on text hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        _text.hash(&mut hasher);
        let hash = hasher.finish();

        let mut embedding = vec![];
        for i in 0..self.dimension {
            let value = ((hash.wrapping_add(i as u64) % 1000) as f32) / 1000.0;
            embedding.push(value);
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut embedding {
                *v /= norm;
            }
        }

        Ok(embedding)
    }
}
