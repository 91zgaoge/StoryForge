#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Agent 模型映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMapping {
    pub agent_id: String,
    pub agent_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multimodal_model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 写作策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingStrategy {
    pub run_mode: String,
    #[serde(default = "default_conflict_level")]
    pub conflict_level: i32,
    pub pace: String,
    pub ai_freedom: String,
}

fn default_conflict_level() -> i32 {
    50
}

impl Default for WritingStrategy {
    fn default() -> Self {
        Self {
            run_mode: "fast".to_string(),
            conflict_level: 50,
            pace: "balanced".to_string(),
            ai_freedom: "medium".to_string(),
        }
    }
}

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
    #[serde(default)]
    pub agent_mappings: HashMap<String, AgentMapping>,
    /// 拆书分析 LLM 并发数（默认 3，本地模型可调大）
    #[serde(default = "default_concurrency")]
    pub book_deconstruction_concurrency: usize,
    /// AgentOrchestrator 质检改写阈值（默认 0.75）
    #[serde(default = "default_rewrite_threshold")]
    pub rewrite_threshold: f32,
    /// AgentOrchestrator 最大反馈循环次数（默认 2）
    #[serde(default = "default_max_feedback_loops")]
    pub max_feedback_loops: u32,
    #[serde(default)]
    pub writing_strategy: WritingStrategy,
    /// OAuth 客户端配置 (v4.5.0)
    #[serde(default)]
    pub auth_clients: Option<HashMap<String, crate::auth::OAuthClientConfig>>,
}

fn default_rewrite_threshold() -> f32 {
    0.75
}

fn default_max_feedback_loops() -> u32 {
    2
}

fn default_concurrency() -> usize {
    3
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

        // 1. 语言模型 - Qwen3.5-27B-Uncensored-Q4_K_M
        let qwen35 = LlmProfile {
            id: "Qwen3.5-27B-Uncensored-Q4_K_M".to_string(),
            name: "Qwen 3.5 语言模型".to_string(),
            description: Some("本地语言模型，用于文本生成和对话".to_string()),
            provider: LlmProvider::Custom,
            model: "Qwen3.5-27B-Uncensored-Q4_K_M".to_string(),
            api_key: "".to_string(),
            api_base: Some("http://10.62.239.13:17098/v1".to_string()),
            max_tokens: 8192,
            temperature: 0.8,
            timeout_seconds: 120,
            is_default: true,
            capabilities: vec![
                ModelCapability::Chat,
                ModelCapability::Completion,
                ModelCapability::LongContext,
            ],
        };
        llm_profiles.insert(qwen35.id.clone(), qwen35);

        // 2. 多模态模型 - Gemma-4-31B-it-Q6_K
        let gemma4 = LlmProfile {
            id: "Gemma-4-31B-it-Q6_K".to_string(),
            name: "Gemma 4 多模态".to_string(),
            description: Some("本地多模态模型，支持图文理解".to_string()),
            provider: LlmProvider::Custom,
            model: "Gemma-4-31B-it-Q6_K".to_string(),
            api_key: "".to_string(),
            api_base: Some("http://10.62.239.13:17099/v1".to_string()),
            max_tokens: 8192,
            temperature: 0.7,
            timeout_seconds: 120,
            is_default: false,
            capabilities: vec![
                ModelCapability::Chat,
                ModelCapability::Vision,
                ModelCapability::LongContext,
            ],
        };
        llm_profiles.insert(gemma4.id.clone(), gemma4);

        // 3. Embedding 嵌入模型 - bge-m3
        let bge_m3 = EmbeddingProfile {
            id: "bge-m3".to_string(),
            name: "BGE-M3 Embedding".to_string(),
            description: Some("文本嵌入模型，用于语义搜索和向量化".to_string()),
            provider: EmbeddingProvider::Custom,
            model: "bge-m3".to_string(),
            api_key: "76e0e2bc84c45374999a1d5e66962544c09cc00ae42ad25cd6a2a07a9d7fe330".to_string(),
            api_base: Some("http://10.62.239.13:8089".to_string()),
            dimensions: 1024,
            max_input_tokens: 8192,
            is_default: true,
        };
        embedding_profiles.insert(bge_m3.id.clone(), bge_m3);

        let mut agent_mappings = HashMap::new();
        agent_mappings.insert("writer".to_string(), AgentMapping {
            agent_id: "writer".to_string(),
            agent_name: "写作助手".to_string(),
            chat_model_id: Some("Qwen3.5-27B-Uncensored-Q4_K_M".to_string()),
            embedding_model_id: None,
            multimodal_model_id: None,
            description: Some("负责章节生成、改写".to_string()),
        });
        agent_mappings.insert("inspector".to_string(), AgentMapping {
            agent_id: "inspector".to_string(),
            agent_name: "质检员".to_string(),
            chat_model_id: Some("Qwen3.5-27B-Uncensored-Q4_K_M".to_string()),
            embedding_model_id: None,
            multimodal_model_id: None,
            description: Some("负责内容质量检查".to_string()),
        });
        agent_mappings.insert("outline_planner".to_string(), AgentMapping {
            agent_id: "outline_planner".to_string(),
            agent_name: "大纲规划师".to_string(),
            chat_model_id: Some("Qwen3.5-27B-Uncensored-Q4_K_M".to_string()),
            embedding_model_id: None,
            multimodal_model_id: None,
            description: Some("负责故事大纲设计".to_string()),
        });
        agent_mappings.insert("style_mimic".to_string(), AgentMapping {
            agent_id: "style_mimic".to_string(),
            agent_name: "风格模仿师".to_string(),
            chat_model_id: Some("Qwen3.5-27B-Uncensored-Q4_K_M".to_string()),
            embedding_model_id: None,
            multimodal_model_id: None,
            description: Some("负责文风分析与模仿".to_string()),
        });
        agent_mappings.insert("plot_analyzer".to_string(), AgentMapping {
            agent_id: "plot_analyzer".to_string(),
            agent_name: "情节分析师".to_string(),
            chat_model_id: Some("Qwen3.5-27B-Uncensored-Q4_K_M".to_string()),
            embedding_model_id: None,
            multimodal_model_id: None,
            description: Some("负责情节复杂度分析".to_string()),
        });

        Self {
            llm: LlmConfig {
                provider: "custom".to_string(),
                api_key: "".to_string(),
                model: "Qwen3.5-27B-Uncensored-Q4_K_M".to_string(),
                api_base: Some("http://10.62.239.13:17098/v1".to_string()),
                max_tokens: 8192,
                temperature: 0.8,
            },
            llm_profiles,
            embedding_profiles,
            active_llm_profile: Some("Qwen3.5-27B-Uncensored-Q4_K_M".to_string()),
            active_embedding_profile: Some("bge-m3".to_string()),
            agent_mappings,
            book_deconstruction_concurrency: 3,
            rewrite_threshold: 0.75,
            max_feedback_loops: 2,
            writing_strategy: WritingStrategy::default(),
        }
    }
}

impl AppConfig {
    pub fn load(config_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = config_dir.join("config.json");
        let mut config = if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            let mut config: AppConfig = serde_json::from_str(&content)?;

            // 迁移旧配置到新格式
            if config.llm_profiles.is_empty() {
                config.migrate_legacy_config();
            }

            config
        } else {
            AppConfig::default()
        };

        // 自动补充真实本地模型（如果缺失）
        let mut needs_save = false;

        // 补充 Qwen3.5 语言模型
        if !config.llm_profiles.contains_key("Qwen3.5-27B-Uncensored-Q4_K_M") {
            let qwen35 = LlmProfile {
                id: "Qwen3.5-27B-Uncensored-Q4_K_M".to_string(),
                name: "Qwen 3.5 语言模型".to_string(),
                description: Some("本地语言模型，用于文本生成和对话".to_string()),
                provider: LlmProvider::Custom,
                model: "Qwen3.5-27B-Uncensored-Q4_K_M".to_string(),
                api_key: "".to_string(),
                api_base: Some("http://10.62.239.13:17098/v1".to_string()),
                max_tokens: 8192,
                temperature: 0.8,
                timeout_seconds: 120,
                is_default: config.llm_profiles.values().all(|p| !p.is_default),
                capabilities: vec![
                    ModelCapability::Chat,
                    ModelCapability::Completion,
                    ModelCapability::LongContext,
                ],
            };
            config.llm_profiles.insert(qwen35.id.clone(), qwen35);
            if config.active_llm_profile.is_none() {
                config.active_llm_profile = Some("Qwen3.5-27B-Uncensored-Q4_K_M".to_string());
            }
            needs_save = true;
        }

        // 补充 Gemma-4 多模态模型
        if !config.llm_profiles.contains_key("Gemma-4-31B-it-Q6_K") {
            let gemma4 = LlmProfile {
                id: "Gemma-4-31B-it-Q6_K".to_string(),
                name: "Gemma 4 多模态".to_string(),
                description: Some("本地多模态模型，支持图文理解".to_string()),
                provider: LlmProvider::Custom,
                model: "Gemma-4-31B-it-Q6_K".to_string(),
                api_key: "".to_string(),
                api_base: Some("http://10.62.239.13:17099/v1".to_string()),
                max_tokens: 8192,
                temperature: 0.7,
                timeout_seconds: 120,
                is_default: false,
                capabilities: vec![
                    ModelCapability::Chat,
                    ModelCapability::Vision,
                    ModelCapability::LongContext,
                ],
            };
            config.llm_profiles.insert(gemma4.id.clone(), gemma4);
            needs_save = true;
        }

        // 补充 bge-m3 嵌入模型
        if !config.embedding_profiles.contains_key("bge-m3") {
            let bge_m3 = EmbeddingProfile {
                id: "bge-m3".to_string(),
                name: "BGE-M3 Embedding".to_string(),
                description: Some("文本嵌入模型，用于语义搜索和向量化".to_string()),
                provider: EmbeddingProvider::Custom,
                model: "bge-m3".to_string(),
                api_key: "76e0e2bc84c45374999a1d5e66962544c09cc00ae42ad25cd6a2a07a9d7fe330".to_string(),
                api_base: Some("http://10.62.239.13:8089".to_string()),
                dimensions: 1024,
                max_input_tokens: 8192,
                is_default: config.embedding_profiles.values().all(|p| !p.is_default),
            };
            config.embedding_profiles.insert(bge_m3.id.clone(), bge_m3);
            if config.active_embedding_profile.is_none() {
                config.active_embedding_profile = Some("bge-m3".to_string());
            }
            needs_save = true;
        }

        if needs_save {
            config.save(config_dir)?;
        }

        Ok(config)
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

#[cfg(test)]
#[path = "settings_tests.rs"]
mod settings_tests;
