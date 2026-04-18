//! 创作方法论引擎
//!
//! 将经典创作方法论编码为系统级提示词规范，
//! 在幕后配置，幕前无感知自动应用。
//!
//! 支持的方法论：
//! - 雪花写作法 (Snowflake)
//! - 场景结构规范 (Scene Structure: Goal-Conflict-Disaster-Reaction-Dilemma-Decision)
//! - 英雄之旅 (Hero's Journey)
//! - 人物深度模型 (Character Depth)

pub mod snowflake;
pub mod scene_structure;
pub mod hero_journey;
pub mod character_depth;

pub use snowflake::{SnowflakeMethodology, SnowflakeStep};
pub use scene_structure::SceneStructureMethodology;
pub use hero_journey::{HeroJourneyMethodology, HeroJourneyStage};
pub use character_depth::CharacterDepthModel;

use serde::{Deserialize, Serialize};

/// 方法论类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MethodologyType {
    Snowflake,
    SceneStructure,
    HeroJourney,
    CharacterDepth,
}

impl MethodologyType {
    pub fn name(&self) -> &'static str {
        match self {
            MethodologyType::Snowflake => "雪花写作法",
            MethodologyType::SceneStructure => "场景结构规范",
            MethodologyType::HeroJourney => "英雄之旅",
            MethodologyType::CharacterDepth => "人物深度模型",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MethodologyType::Snowflake => "从一句话逐步扩展为完整小说的十步创作法",
            MethodologyType::SceneStructure => "目标-冲突-灾难-反应-困境-决定六节拍场景结构",
            MethodologyType::HeroJourney => "约瑟夫·坎普贝尔的12阶段英雄之旅结构",
            MethodologyType::CharacterDepth => "目标-动机-冲突-秘密-弧光-顿悟六维人物模型",
        }
    }
}

/// 方法论 trait - 所有方法论必须实现
pub trait Methodology: Send + Sync {
    /// 方法论名称
    fn name(&self) -> &'static str;
    /// 方法论描述
    fn description(&self) -> &'static str;
    /// 获取该方法论的 system prompt 片段
    fn system_prompt_extension(&self) -> String;
    /// 获取输出格式要求（JSON Schema 或文本描述）
    fn output_schema(&self) -> Option<String>;
    /// 获取该方法论的当前阶段/步骤（如有）
    fn current_step(&self) -> Option<String>;
}

/// 方法论配置（存储于数据库或配置中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodologyConfig {
    pub methodology_type: MethodologyType,
    pub is_active: bool,
    pub current_step: Option<String>,
    pub custom_params: serde_json::Value,
}

impl Default for MethodologyConfig {
    fn default() -> Self {
        Self {
            methodology_type: MethodologyType::SceneStructure,
            is_active: true,
            current_step: None,
            custom_params: serde_json::json!({}),
        }
    }
}

/// 方法论引擎 - 根据配置选择并应用方法论
pub struct MethodologyEngine;

impl MethodologyEngine {
    /// 根据配置生成 system prompt 扩展
    pub fn build_prompt_extension(config: &MethodologyConfig) -> String {
        if !config.is_active {
            return String::new();
        }

        let methodology: Box<dyn Methodology> = match config.methodology_type {
            MethodologyType::Snowflake => {
                let step = config.current_step.as_ref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(SnowflakeStep::OneSentence);
                Box::new(SnowflakeMethodology::new(step))
            }
            MethodologyType::SceneStructure => {
                Box::new(SceneStructureMethodology::default())
            }
            MethodologyType::HeroJourney => {
                let stage = config.current_step.as_ref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(HeroJourneyStage::OrdinaryWorld);
                Box::new(HeroJourneyMethodology::new(stage))
            }
            MethodologyType::CharacterDepth => {
                Box::new(CharacterDepthModel::default())
            }
        };

        methodology.system_prompt_extension()
    }

    /// 获取所有可用方法论列表
    pub fn list_available() -> Vec<MethodologyType> {
        vec![
            MethodologyType::Snowflake,
            MethodologyType::SceneStructure,
            MethodologyType::HeroJourney,
            MethodologyType::CharacterDepth,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_methodology_type_name() {
        assert_eq!(MethodologyType::Snowflake.name(), "雪花写作法");
        assert_eq!(MethodologyType::SceneStructure.name(), "场景结构规范");
        assert_eq!(MethodologyType::HeroJourney.name(), "英雄之旅");
        assert_eq!(MethodologyType::CharacterDepth.name(), "人物深度模型");
    }

    #[test]
    fn test_list_available() {
        let list = MethodologyEngine::list_available();
        assert_eq!(list.len(), 4);
    }

    #[test]
    fn test_build_prompt_extension_scene_structure() {
        let config = MethodologyConfig {
            methodology_type: MethodologyType::SceneStructure,
            is_active: true,
            current_step: None,
            custom_params: serde_json::json!({}),
        };
        let ext = MethodologyEngine::build_prompt_extension(&config);
        assert!(ext.contains("目标场景"));
        assert!(ext.contains("反应场景"));
    }

    #[test]
    fn test_build_prompt_extension_inactive() {
        let config = MethodologyConfig {
            methodology_type: MethodologyType::Snowflake,
            is_active: false,
            current_step: None,
            custom_params: serde_json::json!({}),
        };
        let ext = MethodologyEngine::build_prompt_extension(&config);
        assert!(ext.is_empty());
    }
}
