use super::{GenerateRequest, GenerateResponse, LlmAdapter};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAiAdapter {
    api_key: String,
    model: String,
    api_base: String,
    default_max_tokens: i32,
    default_temperature: f32,
}

#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: i32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    model: String,
    usage: Usage,
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    total_tokens: i32,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

impl OpenAiAdapter {
    pub fn new(
        api_key: String,
        model: String,
        api_base: Option<String>,
        max_tokens: i32,
        temperature: f32,
    ) -> Self {
        Self {
            api_key,
            model,
            api_base: api_base.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            default_max_tokens: max_tokens,
            default_temperature: temperature,
        }
    }

    fn calculate_cost(&self, model: &str, tokens: i32) -> f64 {
        // Pricing per 1K tokens (as of 2024)
        let rate = match model {
            "gpt-4" => 0.03,
            "gpt-4-turbo" => 0.01,
            "gpt-3.5-turbo" => 0.002,
            _ => 0.002,
        };
        (tokens as f64 / 1000.0) * rate
    }
}

#[async_trait::async_trait]
impl LlmAdapter for OpenAiAdapter {
    async fn generate(
        &self,
        request: GenerateRequest,
    ) -> Result<GenerateResponse, Box<dyn std::error::Error>> {
        let client = Client::new();
        
        let openai_req = OpenAiRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a professional creative writing assistant.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: request.prompt,
                },
            ],
            max_tokens: request.max_tokens.unwrap_or(self.default_max_tokens),
            temperature: request.temperature.unwrap_or(self.default_temperature),
        };

        let response = client
            .post(format!("{}/chat/completions", self.api_base))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_req)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("OpenAI API error: {}", error_text).into());
        }

        let openai_resp: OpenAiResponse = response.json().await?;
        let content = openai_resp.choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();
        
        let cost = self.calculate_cost(&openai_resp.model, openai_resp.usage.total_tokens);

        Ok(GenerateResponse {
            content,
            model: openai_resp.model,
            tokens_used: openai_resp.usage.total_tokens,
            cost,
        })
    }

    fn model_name(&self) -> String {
        self.model.clone()
    }
}
