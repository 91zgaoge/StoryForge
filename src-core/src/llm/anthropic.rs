use crate::error::{CinemaError, Result};
use crate::llm::adapter::{AdapterConfig, LlmAdapter};
use crate::llm::types::*;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct AnthropicAdapter {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool>>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
enum AnthropicContent {
    Text { text: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: String, is_error: Option<bool> },
    Image { source: ImageSource },
}

#[derive(Debug, Serialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
    model: String,
    stop_reason: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

impl AnthropicAdapter {
    pub fn new(config: AdapterConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key: config.api_key,
            base_url: config.base_url
                .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string()),
        }
    }

    fn convert_messages(&self,
        messages: Vec<LlmMessage>) -> Vec<AnthropicMessage> {
        messages.into_iter()
            .filter_map(|msg| {
                // Anthropic doesn't allow system messages in the messages array
                if msg.role == Role::System {
                    return None;
                }

                let role = match msg.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::System => return None,
                };

                let content: Vec<AnthropicContent> = msg.content.into_iter()
                    .map(|block| match block {
                        ContentBlock::Text { text } => {
                            AnthropicContent::Text { text }
                        }
                        ContentBlock::ToolUse { id, name, input } => {
                            AnthropicContent::ToolUse { id, name, input }
                        }
                        ContentBlock::ToolResult { tool_use_id, content, is_error } => {
                            AnthropicContent::ToolResult {
                                tool_use_id,
                                content,
                                is_error: Some(is_error),
                            }
                        }
                        ContentBlock::Image { source } => {
                            AnthropicContent::Image { source }
                        }
                    })
                    .collect();

                Some(AnthropicMessage {
                    role: role.to_string(),
                    content,
                })
            })
            .collect()
    }

    fn convert_tools(&self,
        tools: Vec<ToolDefinition>) -> Vec<AnthropicTool> {
        tools.into_iter()
            .map(|tool| AnthropicTool {
                name: tool.name,
                description: tool.description,
                input_schema: tool.parameters,
            })
            .collect()
    }

    fn convert_response(&self,
        response: AnthropicResponse) -> LlmResponse {
        let content: Vec<ContentBlock> = response.content.into_iter()
            .map(|c| match c {
                AnthropicContent::Text { text } => {
                    ContentBlock::Text { text }
                }
                AnthropicContent::ToolUse { id, name, input } => {
                    ContentBlock::ToolUse { id, name, input }
                }
                AnthropicContent::ToolResult { tool_use_id, content, is_error } => {
                    ContentBlock::ToolResult {
                        tool_use_id,
                        content,
                        is_error: is_error.unwrap_or(false),
                    }
                }
                AnthropicContent::Image { source } => {
                    ContentBlock::Image { source }
                }
            })
            .collect();

        LlmResponse {
            content,
            usage: TokenUsage {
                input_tokens: response.usage.input_tokens,
                output_tokens: response.usage.output_tokens,
            },
            model: response.model,
        }
    }
}

#[async_trait]
impl LlmAdapter for AnthropicAdapter {
    async fn chat(
        &self,
        messages: Vec<LlmMessage>,
        options: LlmOptions,
    ) -> Result<LlmResponse> {
        let anthropic_messages = self.convert_messages(messages.clone());

        // Extract system message if present
        let system = options.system.or_else(|| {
            messages.iter()
                .find(|m| m.role == Role::System)
                .map(|m| m.text_content())
        });

        let request = AnthropicRequest {
            model: options.model,
            messages: anthropic_messages,
            system,
            max_tokens: options.max_tokens.unwrap_or(4096),
            temperature: options.temperature,
            top_p: options.top_p,
            tools: options.tools.map(|t| self.convert_tools(t)),
        };

        let response = self.client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| CinemaError::LlmApi(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CinemaError::LlmApi(format!("API error: {}", error_text)));
        }

        let anthropic_response: AnthropicResponse = response.json().await
            .map_err(|e| CinemaError::LlmApi(format!("Failed to parse response: {}", e)))?;

        Ok(self.convert_response(anthropic_response))
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        // Anthropic doesn't have a models endpoint, return known models
        Ok(vec![
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ])
    }
}
