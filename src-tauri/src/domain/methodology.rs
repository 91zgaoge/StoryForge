//! Methodology domain types.
//!
//! Pure data definitions shared across the creative engine and commands.
//! Methodology engines and behavior remain in
//! `crate::creative_engine::methodology`.

use serde::{Deserialize, Serialize};

/// 方法论类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MethodologyType {
    Snowflake,
    SceneStructure,
    HeroJourney,
    CharacterDepth,
    HighDensityWorldBuilding,
}

impl MethodologyType {
    pub fn name(&self) -> &'static str {
        match self {
            MethodologyType::Snowflake => "雪花写作法",
            MethodologyType::SceneStructure => "场景结构规范",
            MethodologyType::HeroJourney => "英雄之旅",
            MethodologyType::CharacterDepth => "人物深度模型",
            MethodologyType::HighDensityWorldBuilding => "高密度世界构建法",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MethodologyType::Snowflake => "从一句话逐步扩展为完整小说的十步创作法",
            MethodologyType::SceneStructure => "目标-冲突-灾难-反应-困境-决定六节拍场景结构",
            MethodologyType::HeroJourney => "约瑟夫·坎普贝尔的12阶段英雄之旅结构",
            MethodologyType::CharacterDepth => "目标-动机-冲突-秘密-弧光-顿悟六维人物模型",
            MethodologyType::HighDensityWorldBuilding => {
                "用极少元素通过状态驱动、桥节点连接、事件回流构建活的世界"
            }
        }
    }
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

/// 将别名规范为 Strategy / Genesis / WriteTimeBundle 使用的 canonical id。
/// `world_building` / `hdwb` → `high_density_world_building`。
pub fn normalize_methodology_id(id: &str) -> &str {
    match id.trim() {
        "world_building" | "hdwb" | "high_density_world_building" => "high_density_world_building",
        other => other,
    }
}

/// 各方法论的最大步数（章节完成自动推进到顶后停留）。
///
/// 与 `resources/prompts/methodology/` 下已注册的阶段文件数保持一致：
/// - snowflake：methodology_snowflake_step1..step10 → 10
/// - high_density_world_building：seed/expansion/convergence/iteration → 4
/// - 其余单文件方法论无步骤概念 → 1（永不推进）
pub fn methodology_max_steps(id: &str) -> i32 {
    match normalize_methodology_id(id) {
        "snowflake" => 10,
        "high_density_world_building" => 4,
        _ => 1,
    }
}

/// 章节完成后的下一步号：+1，到 `methodology_max_steps` 停留。
pub fn next_methodology_step(id: &str, current: i32) -> i32 {
    let max = methodology_max_steps(id);
    (current.max(1) + 1).min(max)
}

#[cfg(test)]
mod methodology_id_tests {
    use super::*;

    #[test]
    fn normalize_aliases_hdwb() {
        assert_eq!(
            normalize_methodology_id("world_building"),
            "high_density_world_building"
        );
        assert_eq!(
            normalize_methodology_id("hdwb"),
            "high_density_world_building"
        );
        assert_eq!(
            normalize_methodology_id("high_density_world_building"),
            "high_density_world_building"
        );
        assert_eq!(normalize_methodology_id("snowflake"), "snowflake");
    }

    #[test]
    fn methodology_step_advances_and_caps() {
        assert_eq!(next_methodology_step("snowflake", 1), 2);
        assert_eq!(next_methodology_step("snowflake", 9), 10);
        // 到顶停留
        assert_eq!(next_methodology_step("snowflake", 10), 10);
        // hdwb 别名归一化后 cap=4
        assert_eq!(next_methodology_step("hdwb", 3), 4);
        assert_eq!(next_methodology_step("high_density_world_building", 4), 4);
        // 单文件方法论无步骤概念，永不推进
        assert_eq!(next_methodology_step("hero_journey", 1), 1);
        // NULL/非法当前值按 1 处理
        assert_eq!(next_methodology_step("snowflake", 0), 2);
    }
}
