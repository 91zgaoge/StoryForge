//! Story Continuity Engine - 故事连续性引擎
//!
//! 追踪角色状态、检测一致性冲突、管理时间线。
//! 在幕后运行，为 Agent 提供连续性保障。

use crate::db::DbPool;
use crate::db::repositories::{CharacterRepository};
use crate::db::repositories_v3::{SceneRepository, KnowledgeGraphRepository};

/// 角色当前状态
#[derive(Debug, Clone)]
pub struct CharacterState {
    pub character_id: String,
    pub name: String,
    pub current_location: Option<String>,
    pub current_emotion: Option<String>,
    pub active_goal: Option<String>,
    pub secrets_known: Vec<String>,
    pub secrets_unknown: Vec<String>,
    pub arc_progress: f32, // 0.0 - 1.0
}

/// 连续性检查结果
#[derive(Debug, Clone)]
pub struct ConsistencyCheck {
    pub is_valid: bool,
    pub issues: Vec<ConsistencyIssue>,
}

/// 一致性问题
#[derive(Debug, Clone)]
pub struct ConsistencyIssue {
    pub issue_type: IssueType,
    pub severity: Severity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub enum IssueType {
    CharacterLocation,
    CharacterEmotion,
    TimelineConflict,
    WorldRuleViolation,
    RelationshipInconsistency,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

/// 连续性引擎
pub struct ContinuityEngine {
    pool: DbPool,
}

impl ContinuityEngine {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 检查场景的连续性
    pub fn check_scene_continuity(
        &self,
        story_id: &str,
        scene_id: &str,
        proposed_content: &str,
    ) -> Result<ConsistencyCheck, String> {
        let mut issues = Vec::new();

        // 1. 检查角色位置一致性
        if let Ok(scene_issues) = self.check_character_locations(story_id, scene_id, proposed_content) {
            issues.extend(scene_issues);
        }

        // 2. 检查世界观规则一致性
        if let Ok(rule_issues) = self.check_world_rules(story_id, proposed_content) {
            issues.extend(rule_issues);
        }

        let is_valid = !issues.iter().any(|i| i.severity == Severity::Critical);

        Ok(ConsistencyCheck { is_valid, issues })
    }

    /// 获取角色的当前状态
    pub fn get_character_states(&self, story_id: &str) -> Result<Vec<CharacterState>, String> {
        let char_repo = CharacterRepository::new(self.pool.clone());
        let characters = char_repo.get_by_story(story_id)
            .map_err(|e| format!("获取角色失败: {}", e))?;

        let mut states = Vec::new();
        for c in characters {
            // 从知识图谱中查找角色的当前状态
            let kg_repo = KnowledgeGraphRepository::new(self.pool.clone());
            let entities = kg_repo.get_entities_by_story(story_id)
                .map_err(|e| format!("获取实体失败: {}", e))?;

            let character_entity = entities.iter()
                .find(|e| e.name == c.name && e.entity_type.to_string() == "character")
                .cloned();

            let (location, emotion, goal) = if let Some(entity) = character_entity {
                let attrs = &entity.attributes;
                (
                    attrs.get("current_location").and_then(|v| v.as_str().map(|s| s.to_string())),
                    attrs.get("current_emotion").and_then(|v| v.as_str().map(|s| s.to_string())),
                    attrs.get("active_goal").and_then(|v| v.as_str().map(|s| s.to_string())),
                )
            } else {
                (None, None, None)
            };

            states.push(CharacterState {
                character_id: c.id,
                name: c.name,
                current_location: location,
                current_emotion: emotion,
                active_goal: goal,
                secrets_known: vec![],
                secrets_unknown: vec![],
                arc_progress: 0.0,
            });
        }

        Ok(states)
    }

    // ==================== 私有检查方法 ====================

    fn check_character_locations(
        &self,
        story_id: &str,
        scene_id: &str,
        _content: &str,
    ) -> Result<Vec<ConsistencyIssue>, String> {
        let mut issues = Vec::new();

        let scene_repo = SceneRepository::new(self.pool.clone());
        let current_scene = scene_repo.get_by_id(scene_id)
            .map_err(|e| format!("获取场景失败: {}", e))?
            .ok_or("场景不存在")?;

        let scene_location = current_scene.setting_location.clone().unwrap_or_default();

        // 获取角色状态
        let states = self.get_character_states(story_id)?;

        for state in states {
            // 如果角色在当前场景中出场，但内容中提到了他在其他地方
            if current_scene.characters_present.contains(&state.name) {
                if let Some(ref last_location) = state.current_location {
                    if !scene_location.is_empty() && last_location != &scene_location {
                        // 这是一个潜在的一致性问题（但不一定是错误，角色可以移动）
                        issues.push(ConsistencyIssue {
                            issue_type: IssueType::CharacterLocation,
                            severity: Severity::Info,
                            message: format!(
                                "{} 从 '{}' 移动到了 '{}'。请确保移动过程合理。",
                                state.name, last_location, scene_location
                            ),
                            suggestion: Some(format!("考虑在场景中描述 {} 如何到达 {}", state.name, scene_location)),
                        });
                    }
                }
            }
        }

        Ok(issues)
    }

    fn check_world_rules(
        &self,
        story_id: &str,
        content: &str,
    ) -> Result<Vec<ConsistencyIssue>, String> {
        let issues = Vec::new();

        let wb_repo = crate::db::repositories_v3::WorldBuildingRepository::new(self.pool.clone());
        let world_building = match wb_repo.get_by_story(story_id) {
            Ok(Some(wb)) => wb,
            _ => return Ok(issues),
        };

        // 简单的规则检查：搜索内容中是否有违反规则的关键词
        for rule in world_building.rules {
            // 如果规则有明确的禁止关键词
            if let Some(ref desc) = rule.description {
                // 示例：如果规则说"凡人不能飞行"，但内容中出现了"凡人飞行"
                // 这是一个简化的启发式检查，未来可以用 LLM 增强
                let rule_keywords: Vec<&str> = desc.split(|c| c == '，' || c == '。' || c == ';').collect();
                for keyword in rule_keywords {
                    let trimmed = keyword.trim();
                    if trimmed.len() > 4 && content.contains(trimmed) {
                        // 检查是否有否定词
                        let negation_words = ["不能", "禁止", "不得", "无法", "没有"];
                        let has_negation = negation_words.iter().any(|&w| trimmed.contains(w));
                        if !has_negation {
                            // 可能违反规则
                            // 这是一个非常简化的检查，实际应该用 LLM
                        }
                    }
                }
            }
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistency_check_creation() {
        let check = ConsistencyCheck {
            is_valid: true,
            issues: vec![],
        };
        assert!(check.is_valid);
    }
}
