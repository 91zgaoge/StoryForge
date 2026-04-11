//! Model configuration and types

use serde::{Deserialize, Serialize};
use crate::ComplexityTier;

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub provider: ModelProvider,
    pub tier: ComplexityTier,
    pub cost_per_1k_tokens: f64,
    pub context_length: u32,
    pub api_endpoint: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    OpenAI,
    Anthropic,
    Ollama,
    Custom,
}

impl ModelProvider {
    pub fn default_endpoint(&self) -> String {
        match self {
            ModelProvider::OpenAI => "https://api.openai.com/v1".to_string(),
            ModelProvider::Anthropic => "https://api.anthropic.com".to_string(),
            ModelProvider::Ollama => "http://localhost:11434".to_string(),
            ModelProvider::Custom => "".to_string(),
        }
    }
    
    pub fn requires_api_key(&self) -> bool {
        matches!(self, ModelProvider::OpenAI | ModelProvider::Anthropic)
    }
}

/// 路由决策结果
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub model: ModelConfig,
    pub reason: String,
    pub estimated_cost: f64,
    pub temperature: f32,
    pub max_tokens: u32,
    pub fallback: Option<ModelConfig>,
}

/// 任务类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    SceneDescription,    // 场景描写
    Dialogue,           // 对话
    ConflictResolution, // 冲突解决
    CharacterIntro,     // 角色出场
    LogicValidation,    // 逻辑验证
    StyleEvolution,     // 风格进化
    Outline,            // 大纲生成
}
