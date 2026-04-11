use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use super::embedding::*;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, texts: Vec<String>) -> Result<Vec<Embedding>, EmbeddingError>;
    fn dimensions(&self) -> usize;
    fn max_batch_size(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct EmbeddingError {
    pub message: String,
    pub code: String,
}

impl std::fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for EmbeddingError {}

/// OpenAI embedding provider
pub struct OpenAIEmbeddingProvider {
    api_key: String,
    model: String,
    dimensions: usize,
    client: reqwest::Client,
}

impl OpenAIEmbeddingProvider {
    pub fn new(api_key: String, model: String, dimensions: usize) -> Self {
        Self {
            api_key,
            model,
            dimensions,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    async fn embed(&self, texts: Vec<String>) -> Result<Vec<Embedding>, EmbeddingError> {
        let request = OpenAIEmbeddingRequest {
            model: self.model.clone(),
            input: texts.clone(),
        };

        let response = self.client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| EmbeddingError {
                message: e.to_string(),
                code: "REQUEST_FAILED".to_string(),
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError {
                message: error_text,
                code: "API_ERROR".to_string(),
            });
        }

        let result: OpenAIEmbeddingResponse = response.json().await
            .map_err(|e| EmbeddingError {
                message: e.to_string(),
                code: "PARSE_ERROR".to_string(),
            })?;

        Ok(result.data.into_iter().enumerate().map(|(i, d)| Embedding {
            id: format!("emb_{}", i),
            vector: d.embedding,
            dimensions: self.dimensions,
            model: self.model.clone(),
        }).collect())
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn max_batch_size(&self) -> usize {
        100
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Clone, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

/// Local embedding provider (placeholder for local models)
pub struct LocalEmbeddingProvider {
    dimensions: usize,
}

impl LocalEmbeddingProvider {
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }
}

#[async_trait]
impl EmbeddingProvider for LocalEmbeddingProvider {
    async fn embed(&self, texts: Vec<String>) -> Result<Vec<Embedding>, EmbeddingError> {
        // Placeholder: return random embeddings
        // In real implementation, this would use a local model like bert-rs
        Ok(texts.into_iter().enumerate().map(|(i, _)| Embedding {
            id: format!("emb_{}", i),
            vector: vec![0.0; self.dimensions],
            dimensions: self.dimensions,
            model: "local".to_string(),
        }).collect())
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn max_batch_size(&self) -> usize {
        32
    }
}