use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 全局应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm: LlmConfig,
    #[serde(default)]
    pub llm_profiles: HashMap<String, LlmProfile>,
    #[serde(default)]
    pub embedding_profiles: HashMap<String, EmbeddingProfile>,
    #[serde(default)]
    pub active_llm_profile: Option<String>,
    #[serde(default)]
    pub active_embedding_profile: Option<String>,
}

/// 语言模型配置（向后兼容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub api_base: Option<String>,
    pub max_tokens: i32,
    pub temperature: f32,
}

/// LLM 模型配置档案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProfile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub provider: LlmProvider,
    pub model: String,
    pub api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,
    pub max_tokens: i32,
    pub temperature: f32,
    pub timeout_seconds: u64,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub capabilities: Vec<ModelCapability>,
}

/// 支持的LLM提供商
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Azure,
    Ollama,
    DeepSeek,
    Qwen,
    Custom,
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmProvider::OpenAI => write!(f, "openai"),
            LlmProvider::Anthropic => write!(f, "anthropic"),
            LlmProvider::Azure => write!(f, "azure"),
            LlmProvider::Ollama => write!(f, "ollama"),
            LlmProvider::DeepSeek => write!(f, "deepseek"),
            LlmProvider::Qwen => write!(f, "qwen"),
            LlmProvider::Custom => write!(f, "custom"),
        }
    }
}

/// 模型能力
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    Chat,
    Completion,
    FunctionCalling,
    JsonMode,
    Vision,
    LongContext,
}

/// 嵌入模型配置档案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingProfile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub provider: EmbeddingProvider,
    pub model: String,
    pub api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,
    pub dimensions: usize,
    pub max_input_tokens: usize,
    #[serde(default)]
    pub is_default: bool,
}

/// 支持的嵌入模型提供商
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingProvider {
    OpenAI,
    Azure,
    Ollama,
    Local,  // 本地TF-IDF
    Custom,
}

impl std::fmt::Display for EmbeddingProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddingProvider::OpenAI => write!(f, "openai"),
            EmbeddingProvider::Azure => write!(f, "azure"),
            EmbeddingProvider::Ollama => write!(f, "ollama"),
            EmbeddingProvider::Local => write!(f, "local"),
            EmbeddingProvider::Custom => write!(f, "custom"),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut llm_profiles = HashMap::new();
        let mut embedding_profiles = HashMap::new();

        // 默认LLM配置
        let default_llm = LlmProfile {
            id: "default-openai".to_string(),
            name: "OpenAI GPT-3.5".to_string(),
            description: Some("默认OpenAI配置".to_string()),
            provider: LlmProvider::OpenAI,
            model: "gpt-3.5-turbo".to_string(),
            api_key: "".to_string(),
            api_base: None,
            max_tokens: 2000,
            temperature: 0.7,
            timeout_seconds: 120,
            is_default: true,
            capabilities: vec![
                ModelCapability::Chat,
                ModelCapability::FunctionCalling,
                ModelCapability::JsonMode,
            ],
        };
        llm_profiles.insert(default_llm.id.clone(), default_llm);

        // 默认嵌入配置（本地TF-IDF）
        let default_embedding = EmbeddingProfile {
            id: "default-local".to_string(),
            name: "本地TF-IDF".to_string(),
            description: Some("本地向量计算，无需API密钥".to_string()),
            provider: EmbeddingProvider::Local,
            model: "tfidf-local".to_string(),
            api_key: "".to_string(),
            api_base: None,
            dimensions: 0, // 动态
            max_input_tokens: 8192,
            is_default: true,
        };
        embedding_profiles.insert(default_embedding.id.clone(), default_embedding);

        Self {
            llm: LlmConfig {
                provider: "openai".to_string(),
                api_key: "".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                api_base: None,
                max_tokens: 2000,
                temperature: 0.7,
            },
            llm_profiles,
            embedding_profiles,
            active_llm_profile: Some("default-openai".to_string()),
            active_embedding_profile: Some("default-local".to_string()),
        }
    }
}

impl AppConfig {
    pub fn load(config_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = config_dir.join("config.json");
        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            let mut config: AppConfig = serde_json::from_str(&content)?;

            // 迁移旧配置到新格式
            if config.llm_profiles.is_empty() {
                config.migrate_legacy_config();
            }

            Ok(config)
        } else {
            Ok(AppConfig::default())
        }
    }

    pub fn save(&self, config_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(config_dir)?;
        let config_path = config_dir.join("config.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    /// 获取当前活跃的LLM配置
    pub fn get_active_llm_profile(&self) -> Option<&LlmProfile> {
        self.active_llm_profile
            .as_ref()
            .and_then(|id| self.llm_profiles.get(id))
            .or_else(|| self.llm_profiles.values().find(|p| p.is_default))
    }

    /// 获取当前活跃的嵌入模型配置
    pub fn get_active_embedding_profile(&self) -> Option<&EmbeddingProfile> {
        self.active_embedding_profile
            .as_ref()
            .and_then(|id| self.embedding_profiles.get(id))
            .or_else(|| self.embedding_profiles.values().find(|p| p.is_default))
    }

    /// 设置活跃的LLM配置
    pub fn set_active_llm_profile(&mut self, profile_id: &str) -> Result<(), String> {
        if self.llm_profiles.contains_key(profile_id) {
            self.active_llm_profile = Some(profile_id.to_string());
            Ok(())
        } else {
            Err(format!("Profile '{}' not found", profile_id))
        }
    }

    /// 设置活跃的嵌入模型配置
    pub fn set_active_embedding_profile(&mut self, profile_id: &str) -> Result<(), String> {
        if self.embedding_profiles.contains_key(profile_id) {
            self.active_embedding_profile = Some(profile_id.to_string());
            Ok(())
        } else {
            Err(format!("Profile '{}' not found", profile_id))
        }
    }

    /// 添加LLM配置
    pub fn add_llm_profile(&mut self, mut profile: LlmProfile) -> Result<(), String> {
        if profile.id.is_empty() {
            profile.id = format!("llm-{}", uuid::Uuid::new_v4());
        }

        // 如果设为默认，取消其他默认
        if profile.is_default {
            for p in self.llm_profiles.values_mut() {
                p.is_default = false;
            }
        }

        self.llm_profiles.insert(profile.id.clone(), profile);
        Ok(())
    }

    /// 添加嵌入模型配置
    pub fn add_embedding_profile(&mut self, mut profile: EmbeddingProfile) -> Result<(), String> {
        if profile.id.is_empty() {
            profile.id = format!("emb-{}", uuid::Uuid::new_v4());
        }

        // 如果设为默认，取消其他默认
        if profile.is_default {
            for p in self.embedding_profiles.values_mut() {
                p.is_default = false;
            }
        }

        self.embedding_profiles.insert(profile.id.clone(), profile);
        Ok(())
    }

    /// 删除LLM配置
    pub fn remove_llm_profile(&mut self, profile_id: &str) -> Result<(), String> {
        if let Some(profile) = self.llm_profiles.get(profile_id) {
            if profile.is_default && self.llm_profiles.len() > 1 {
                return Err("Cannot delete the default profile".to_string());
            }
            self.llm_profiles.remove(profile_id);

            // 如果删除的是当前活跃配置，重置
            if self.active_llm_profile.as_deref() == Some(profile_id) {
                self.active_llm_profile = self.llm_profiles.values()
                    .find(|p| p.is_default)
                    .map(|p| p.id.clone());
            }
            Ok(())
        } else {
            Err(format!("Profile '{}' not found", profile_id))
        }
    }

    /// 删除嵌入模型配置
    pub fn remove_embedding_profile(&mut self, profile_id: &str) -> Result<(), String> {
        if let Some(profile) = self.embedding_profiles.get(profile_id) {
            if profile.is_default && self.embedding_profiles.len() > 1 {
                return Err("Cannot delete the default profile".to_string());
            }
            self.embedding_profiles.remove(profile_id);

            if self.active_embedding_profile.as_deref() == Some(profile_id) {
                self.active_embedding_profile = self.embedding_profiles.values()
                    .find(|p| p.is_default)
                    .map(|p| p.id.clone());
            }
            Ok(())
        } else {
            Err(format!("Profile '{}' not found", profile_id))
        }
    }

    /// 迁移旧版配置
    fn migrate_legacy_config(&mut self) {
        let legacy_profile = LlmProfile {
            id: "legacy".to_string(),
            name: "Legacy Config".to_string(),
            description: Some("从旧版本迁移的配置".to_string()),
            provider: match self.llm.provider.as_str() {
                "anthropic" => LlmProvider::Anthropic,
                "ollama" => LlmProvider::Ollama,
                _ => LlmProvider::OpenAI,
            },
            model: self.llm.model.clone(),
            api_key: self.llm.api_key.clone(),
            api_base: self.llm.api_base.clone(),
            max_tokens: self.llm.max_tokens,
            temperature: self.llm.temperature,
            timeout_seconds: 120,
            is_default: true,
            capabilities: vec![ModelCapability::Chat, ModelCapability::Completion],
        };

        self.llm_profiles.insert(legacy_profile.id.clone(), legacy_profile);
        self.active_llm_profile = Some("legacy".to_string());
    }
}
