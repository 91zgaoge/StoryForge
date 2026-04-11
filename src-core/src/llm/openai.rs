use crate::error::{CinemaError, Result};
use crate::llm::adapter::{AdapterConfig, LlmAdapter};
use crate::llm::types::*;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAIAdapter {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OpenAITool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Serialize)]
struct OpenAIFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
    model: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    role: String,
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<OpenAIToolCall>,
}

#[derive(Debug, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAIFunctionCall,
}

#[derive(Debug, Deserialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

impl OpenAIAdapter {
    pub fn new(config: AdapterConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key: config.api_key,
            base_url: config.base_url
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
        }
    }

    fn convert_messages(&self,
        messages: Vec<LlmMessage>) -> Vec<OpenAIMessage> {
        messages.into_iter()
            .map(|msg| OpenAIMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: msg.text_content(),
            })
            .collect()
    }

    fn convert_tools(&self,
        tools: Vec<ToolDefinition>) -> Vec<OpenAITool> {
        tools.into_iter()
            .map(|tool| OpenAITool {
                tool_type: "function".to_string(),
                function: OpenAIFunction {
                    name: tool.name,
                    description: tool.description,
                    parameters: tool.parameters,
                },
            })
            .collect()
    }

    fn convert_response(&self,
        response: OpenAIResponse) -> LlmResponse {
        let mut content_blocks = vec![];

        if let Some(choice) = response.choices.into_iter().next() {
            // Add text content
            if let Some(text) = choice.message.content {
                if !text.is_empty() {
                    content_blocks.push(ContentBlock::Text { text });
                }
            }

            // Add tool calls
            for tool_call in choice.message.tool_calls {
                if let Ok(input) = serde_json::from_str(&tool_call.function.arguments) {
                    content_blocks.push(ContentBlock::ToolUse {
                        id: tool_call.id,
                        name: tool_call.function.name,
                        input,
                    });
                }
            }
        }

        LlmResponse {
            content: content_blocks,
            usage: TokenUsage {
                input_tokens: response.usage.prompt_tokens,
                output_tokens: response.usage.completion_tokens,
            },
            model: response.model,
        }
    }
}

#[async_trait]
impl LlmAdapter for OpenAIAdapter {
    async fn chat(
        &self,
        messages: Vec<LlmMessage>,
        options: LlmOptions,
    ) -> Result<LlmResponse> {
        let openai_messages = self.convert_messages(messages);

        // Add system prompt if provided
        let mut final_messages = openai_messages;
        if let Some(system) = options.system {
            final_messages.insert(0, OpenAIMessage {
                role: "system".to_string(),
                content: system,
            });
        }

        let request = OpenAIRequest {
            model: options.model,
            messages: final_messages,
            temperature: options.temperature,
            max_tokens: options.max_tokens,
            top_p: options.top_p,
            tools: options.tools.map(|t| self.convert_tools(t)),
        };

        let response = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let openai_response: OpenAIResponse = response.json().await
            .map_err(|e| CinemaError::LlmApi(format!("Failed to parse response: {}", e)))?;

        Ok(self.convert_response(openai_response))
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let response = self.client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| CinemaError::LlmApi(e.to_string()))?;

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelInfo>,
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            id: String,
        }

        let models: ModelsResponse = response.json().await
            .map_err(|e| CinemaError::LlmApi(e.to_string()))?;

        Ok(models.data.into_iter().map(|m| m.id).collect())
    }
}
