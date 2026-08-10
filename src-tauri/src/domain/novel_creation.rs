//! Novel creation domain types.
//!
//! Shared option types produced by the novel-creation agent.

use serde::{Deserialize, Serialize};

use crate::domain::narrative_elements::{Culture, WorldRule};

/// 世界观选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBuildingOption {
    pub id: String,
    pub concept: String,
    pub rules: Vec<WorldRule>,
    pub history: String,
    pub cultures: Vec<Culture>,
}

/// 角色谱选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterProfileOption {
    pub id: String,
    pub name: String,
    pub personality: String,
    pub background: String,
    pub goals: String,
    pub voice_style: String,
    /// 情感内核：角色的主导情感倾向（缺省空串，向后兼容旧响应）
    #[serde(default)]
    pub emotional_core: String,
    /// 情感触发：引爆情绪的场景/行为
    #[serde(default)]
    pub emotional_trigger: String,
    /// 情感创伤：塑造情感模式的过往伤口
    #[serde(default)]
    pub emotional_wound: String,
    /// 情感需求：角色深层渴望的情感满足
    #[serde(default)]
    pub emotional_need: String,
}

/// 文字风格选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingStyleOption {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tone: String,
    pub pacing: String,
    pub vocabulary_level: String,
    pub sentence_structure: String,
    pub sample_text: String,
}
