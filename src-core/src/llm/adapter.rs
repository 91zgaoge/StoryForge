use crate::error::Result;
use crate::llm::types::{LlmMessage, LlmResponse, LlmOptions};
use async_trait::async_trait;
use std::sync::Arc;

/// LLM provider trait
#[async_trait]
pub trait LlmAdapter: Send + Sync {
    /// Send a chat completion request
    async fn chat(
        &self,
        messages: Vec<LlmMessage>,
        options: LlmOptions,
    ) -> Result<LlmResponse>;

    /// Stream chat completion (optional)
    async fn chat_stream(
        &self,
        _messages: Vec<LlmMessage>,
        _options: LlmOptions,
    ) -> Result<Box<dyn futures::Stream<Item = Result<String>> + Send>> {
        unimplemented!("Streaming not supported for this provider")
    }

    /// Get available models
    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

/// Supported LLM providers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Ollama,
}

/// Configuration for creating an adapter
#[derive(Debug, Clone)]
pub struct AdapterConfig {
    pub provider: Provider,
    pub api_key: String,
    pub base_url: Option<String>,
    pub timeout_secs: u64,
}

/// Create an adapter for the specified provider
pub fn create_adapter(config: AdapterConfig) -> Arc<dyn LlmAdapter> {
    match config.provider {
        Provider::OpenAI => {
            Arc::new(crate::llm::openai::OpenAIAdapter::new(config))
        }
        Provider::Anthropic => {
            Arc::new(crate::llm::anthropic::AnthropicAdapter::new(config))
        }
        Provider::Ollama => {
            Arc::new(crate::llm::ollama::OllamaAdapter::new(config))
        }
    }
}

/// Adapter factory for multiple providers
pub struct AdapterRegistry {
    adapters: std::collections::HashMap<Provider, Arc<dyn LlmAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self,
        provider: Provider,
        adapter: Arc<dyn LlmAdapter>) {
        self.adapters.insert(provider, adapter);
    }

    pub fn get(&self,
        provider: Provider) -> Option<Arc<dyn LlmAdapter>> {
        self.adapters.get(&provider).cloned()
    }

    /// Route based on complexity (from hermes-agent pattern)
    pub fn route_by_complexity(
        &self,
        complexity: crate::ComplexityTier,
    ) -> Option<Arc<dyn LlmAdapter>> {
        match complexity {
            crate::ComplexityTier::Low => {
                // Use Ollama local model if available
                self.adapters.get(&Provider::Ollama)
                    .or_else(|| self.adapters.get(&Provider::OpenAI))
                    .cloned()
            }
            crate::ComplexityTier::Medium | crate::ComplexityTier::High => {
                self.adapters.get(&Provider::OpenAI)
                    .or_else(|| self.adapters.get(&Provider::Anthropic))
                    .cloned()
            }
            crate::ComplexityTier::Critical => {
                // Always use best available
                self.adapters.get(&Provider::Anthropic)
                    .or_else(|| self.adapters.get(&Provider::OpenAI))
                    .cloned()
            }
        }
    }
}
