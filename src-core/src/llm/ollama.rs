use crate::error::{CinemaError, Result};
use crate::llm::adapter::{AdapterConfig, LlmAdapter};
use crate::llm::types::*;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OllamaAdapter {
    client: Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaResponseMessage,
    #[serde(default)]
    prompt_eval_count: u32,
    #[serde(default)]
    eval_count: u32,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

impl OllamaAdapter {
    pub fn new(config: AdapterConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // Longer timeout for local models
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: config.base_url
                .unwrap_or_else(|| "http://localhost:11434".to_string()),
        }
    }

    fn convert_messages(&self,
        messages: Vec<LlmMessage>) -> Vec<OllamaMessage> {
        let mut ollama_msgs: Vec<OllamaMessage> = messages.into_iter()
            .map(|msg| OllamaMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: msg.text_content(),
            })
            .collect();

        // Merge consecutive system messages (Ollama quirk)
        let mut merged = vec![];
        for msg in ollama_msgs {
            if let Some(last) = merged.last_mut() {
                if last.role == "system" && msg.role == "system" {
                    last.content.push('\n');
                    last.content.push_str(&msg.content);
                    continue;
                }
            }
            merged.push(msg);
        }

        merged
    }

    fn convert_response(&self,
        response: OllamaResponse) -> LlmResponse {
        LlmResponse {
            content: vec![ContentBlock::Text {
                text: response.message.content,
            }],
            usage: TokenUsage {
                input_tokens: response.prompt_eval_count,
                output_tokens: response.eval_count,
            },
            model: "ollama".to_string(),
        }
    }
}

#[async_trait]
impl LlmAdapter for OllamaAdapter {
    async fn chat(
        &self,
        messages: Vec<LlmMessage>,
        options: LlmOptions,
    ) -> Result<LlmResponse> {
        let ollama_messages = self.convert_messages(messages);

        let request = OllamaRequest {
            model: options.model,
            messages: ollama_messages,
            stream: false,
            options: Some(OllamaOptions {
                temperature: options.temperature,
                num_predict: options.max_tokens,
                top_p: options.top_p,
            }),
        };

        let response = self.client
            .post(format!("{}/api/chat", self.base_url))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    CinemaError::LlmApi(
                        "Cannot connect to Ollama. Is it running? (ollama serve)".to_string()
                    )
                } else {
                    CinemaError::LlmApi(format!("Request failed: {}", e))
                }
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CinemaError::LlmApi(format!("Ollama error: {}", error_text)));
        }

        let ollama_response: OllamaResponse = response.json().await
            .map_err(|e| CinemaError::LlmApi(format!("Failed to parse response: {}", e)))?;

        Ok(self.convert_response(ollama_response))
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let response = self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| CinemaError::LlmApi(e.to_string()))?;

        let models: OllamaModelsResponse = response.json().await
            .map_err(|e| CinemaError::LlmApi(e.to_string()))?;

        Ok(models.models.into_iter().map(|m| m.name).collect())
    }
}
