use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub api_keys: ApiKeys,
    pub models: ModelSettings,
    pub generation: GenerationSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeys {
    pub openai: Option<String>,
    pub anthropic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSettings {
    pub default_provider: String,
    pub default_model: String,
    pub cheap_model: String,
    pub enable_smart_routing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationSettings {
    pub target_word_count: u32,
    pub max_tokens: u32,
    pub temperature: f32,
    pub auto_save: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_keys: ApiKeys {
                openai: None,
                anthropic: None,
            },
            models: ModelSettings {
                default_provider: "openai".to_string(),
                default_model: "gpt-4o".to_string(),
                cheap_model: "gpt-3.5-turbo".to_string(),
                enable_smart_routing: true,
            },
            generation: GenerationSettings {
                target_word_count: 3000,
                max_tokens: 4000,
                temperature: 0.7,
                auto_save: true,
            },
        }
    }
}
