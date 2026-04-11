use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub max_tokens: Option<i32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: i32,
    pub cost: f64,
}

#[async_trait::async_trait]
pub trait LlmAdapter: Send + Sync {
    async fn generate(&self,
        request: GenerateRequest,
    ) -> Result<GenerateResponse, Box<dyn std::error::Error>>;
    
    fn model_name(&self) -> String;
}
