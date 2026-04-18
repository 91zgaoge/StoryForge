//! 自适应生成器
//!
//! 根据用户偏好动态调整生成策略：
//! - temperature / top-p 调整
//! - prompt 权重调整
//! - 生成内容类型偏好注入

use crate::db::DbPool;
use crate::db::repositories_v3::UserPreferenceRepository;

/// 生成策略
#[derive(Debug, Clone)]
pub struct GenerationStrategy {
    /// 温度（创造性 vs 确定性）
    pub temperature: f32,
    /// top-p（核采样）
    pub top_p: f32,
    /// 最大 token 数
    pub max_tokens: i32,
    /// 系统提示词权重增强
    pub prompt_weight_adjustments: Vec<PromptWeightAdjustment>,
    /// 风格偏好注入
    pub style_injections: Vec<String>,
    /// 内容约束
    pub content_constraints: Vec<String>,
}

impl Default for GenerationStrategy {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            top_p: 0.95,
            max_tokens: 2000,
            prompt_weight_adjustments: vec![],
            style_injections: vec![],
            content_constraints: vec![],
        }
    }
}

/// Prompt 权重调整
#[derive(Debug, Clone)]
pub struct PromptWeightAdjustment {
    pub target: String,      // 调整目标（如"对话""描写"）
    pub direction: String,   // increase / decrease / maintain
    pub strength: f32,       // 0.0-1.0
    pub reason: String,
}

/// 自适应生成器
pub struct AdaptiveGenerator {
    pool: DbPool,
}

impl AdaptiveGenerator {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 为故事构建生成策略
    pub fn build_strategy(&self, story_id: &str) -> Result<GenerationStrategy, String> {
        let mut strategy = GenerationStrategy::default();

        let pref_repo = UserPreferenceRepository::new(self.pool.clone());
        let prefs = pref_repo.get_by_story(story_id).map_err(|e| e.to_string())?;

        for pref in &prefs {
            if pref.confidence < 0.6 {
                continue;
            }

            match pref.preference_type.to_string().as_str() {
                "dialogue" => self.apply_dialogue_preference(&mut strategy, pref),
                "content" => self.apply_content_preference(&mut strategy, pref),
                "pacing" => self.apply_pacing_preference(&mut strategy, pref),
                "style" => self.apply_style_preference(&mut strategy, pref),
                _ => {}
            }
        }

        // 综合调整 temperature
        strategy.temperature = self.calculate_temperature(&strategy);

        Ok(strategy)
    }

    fn apply_dialogue_preference(&self, strategy: &mut GenerationStrategy, pref: &crate::db::models_v3::UserPreference) {
        match pref.preference_key.as_str() {
            "dialogue_ratio" => {
                match pref.preference_value.as_str() {
                    "prefer_more_dialogue" => {
                        strategy.prompt_weight_adjustments.push(PromptWeightAdjustment {
                            target: "对话".to_string(),
                            direction: "increase".to_string(),
                            strength: pref.confidence,
                            reason: "用户偏好更多对话".to_string(),
                        });
                        strategy.content_constraints.push("增加对话比例，让角色通过对话推动情节".to_string());
                    }
                    "prefer_less_dialogue" => {
                        strategy.prompt_weight_adjustments.push(PromptWeightAdjustment {
                            target: "对话".to_string(),
                            direction: "decrease".to_string(),
                            strength: pref.confidence,
                            reason: "用户偏好减少对话".to_string(),
                        });
                        strategy.content_constraints.push("减少对话，增加叙述和描写".to_string());
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn apply_content_preference(&self, strategy: &mut GenerationStrategy, pref: &crate::db::models_v3::UserPreference) {
        match pref.preference_key.as_str() {
            "description_ratio" => {
                match pref.preference_value.as_str() {
                    "prefer_more_description" => {
                        strategy.prompt_weight_adjustments.push(PromptWeightAdjustment {
                            target: "环境描写".to_string(),
                            direction: "increase".to_string(),
                            strength: pref.confidence,
                            reason: "用户偏好更多环境描写".to_string(),
                        });
                        strategy.content_constraints.push("增加环境描写和氛围渲染".to_string());
                        strategy.temperature = (strategy.temperature + 0.05).min(1.0);
                    }
                    "prefer_less_description" => {
                        strategy.prompt_weight_adjustments.push(PromptWeightAdjustment {
                            target: "环境描写".to_string(),
                            direction: "decrease".to_string(),
                            strength: pref.confidence,
                            reason: "用户偏好减少环境描写".to_string(),
                        });
                        strategy.content_constraints.push("精简环境描写，聚焦于情节和动作".to_string());
                    }
                    _ => {}
                }
            }
            "interior_monologue" => {
                match pref.preference_value.as_str() {
                    "prefer_more_interior_monologue" => {
                        strategy.content_constraints.push("增加角色内心独白和心理活动描写".to_string());
                    }
                    "prefer_less_interior_monologue" => {
                        strategy.content_constraints.push("减少内心独白，多展示角色的外在行为和对话".to_string());
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn apply_pacing_preference(&self, strategy: &mut GenerationStrategy, pref: &crate::db::models_v3::UserPreference) {
        match pref.preference_key.as_str() {
            "sentence_length" => {
                match pref.preference_value.as_str() {
                    "prefer_slower_pacing" => {
                        strategy.prompt_weight_adjustments.push(PromptWeightAdjustment {
                            target: "节奏".to_string(),
                            direction: "decrease".to_string(),
                            strength: pref.confidence,
                            reason: "用户偏好慢节奏".to_string(),
                        });
                        strategy.content_constraints.push("使用更长、更复杂的句子，放慢叙事节奏".to_string());
                        strategy.temperature = (strategy.temperature - 0.05).max(0.5);
                    }
                    "prefer_faster_pacing" => {
                        strategy.prompt_weight_adjustments.push(PromptWeightAdjustment {
                            target: "节奏".to_string(),
                            direction: "increase".to_string(),
                            strength: pref.confidence,
                            reason: "用户偏好快节奏".to_string(),
                        });
                        strategy.content_constraints.push("使用短句、快节奏，增加动作密度".to_string());
                        strategy.temperature = (strategy.temperature + 0.05).min(1.0);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn apply_style_preference(&self, strategy: &mut GenerationStrategy, pref: &crate::db::models_v3::UserPreference) {
        match pref.preference_key.as_str() {
            "overall_satisfaction" => {
                match pref.preference_value.as_str() {
                    "needs_improvement" => {
                        // 降低 temperature 以增加可控性
                        strategy.temperature = (strategy.temperature - 0.1).max(0.5);
                        strategy.style_injections.push("注意：用户近期满意度较低，请严格遵循风格和结构规范".to_string());
                    }
                    "high_satisfaction" => {
                        // 可适当提高创造性
                        strategy.temperature = (strategy.temperature + 0.05).min(1.0);
                        strategy.style_injections.push("用户满意度较高，保持当前风格即可".to_string());
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn calculate_temperature(&self, strategy: &GenerationStrategy) -> f32 {
        // 基础温度 0.8
        let base = 0.8f32;

        // 根据调整约束数量微调
        let constraint_count = strategy.content_constraints.len() as f32;
        let constraint_adjustment = if constraint_count > 3.0 {
            -0.05 // 约束多时降低温度以提高可控性
        } else {
            0.0
        };

        (base + constraint_adjustment).clamp(0.5, 1.0)
    }

    /// 将策略转换为 prompt 扩展文本
    pub fn strategy_to_prompt(strategy: &GenerationStrategy) -> String {
        let mut parts = Vec::new();

        if !strategy.style_injections.is_empty() {
            for injection in &strategy.style_injections {
                parts.push(injection.clone());
            }
        }

        if !strategy.content_constraints.is_empty() {
            parts.push("\n【内容调整】".to_string());
            for constraint in &strategy.content_constraints {
                parts.push(format!("- {}", constraint));
            }
        }

        if !strategy.prompt_weight_adjustments.is_empty() {
            parts.push("\n【生成策略调整】".to_string());
            for adj in &strategy.prompt_weight_adjustments {
                let direction_cn = match adj.direction.as_str() {
                    "increase" => "增加",
                    "decrease" => "减少",
                    _ => "保持",
                };
                parts.push(format!(
                    "- {}「{}」比重（置信度: {:.0}%）",
                    direction_cn, adj.target, adj.strength * 100.0
                ));
            }
        }

        if parts.is_empty() {
            String::new()
        } else {
            parts.join("\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_strategy() {
        let s = GenerationStrategy::default();
        assert_eq!(s.temperature, 0.8);
        assert_eq!(s.max_tokens, 2000);
    }

    #[test]
    fn test_strategy_to_prompt() {
        let mut s = GenerationStrategy::default();
        s.content_constraints.push("增加对话比例".to_string());
        s.prompt_weight_adjustments.push(PromptWeightAdjustment {
            target: "对话".to_string(),
            direction: "increase".to_string(),
            strength: 0.8,
            reason: "test".to_string(),
        });

        let prompt = AdaptiveGenerator::strategy_to_prompt(&s);
        assert!(prompt.contains("增加对话比例"));
        assert!(prompt.contains("增加「对话」比重"));
    }

    #[test]
    fn test_calculate_temperature() {
        let g = AdaptiveGenerator::new(crate::db::DbPool::new(
            r2d2_sqlite::SqliteConnectionManager::memory()
        ).unwrap());
        let mut s = GenerationStrategy::default();
        s.content_constraints.push("c1".to_string());
        s.content_constraints.push("c2".to_string());
        s.content_constraints.push("c3".to_string());
        s.content_constraints.push("c4".to_string());
        let temp = g.calculate_temperature(&s);
        assert_eq!(temp, 0.75); // 约束多，降低
    }
}
