use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 合同类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractType {
    MasterSetting,
    Volume,
    Chapter,
    Review,
}

impl std::fmt::Display for ContractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ContractType::MasterSetting => "MASTER_SETTING",
            ContractType::Volume => "VOLUME",
            ContractType::Chapter => "CHAPTER",
            ContractType::Review => "REVIEW",
        };
        write!(f, "{}", s)
    }
}

/// MASTER_SETTING 合同结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterSettingContract {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    #[serde(rename = "contract_type")]
    pub contract_type: String,
    #[serde(rename = "generator_version")]
    pub generator_version: String,
    pub genre: String,
    #[serde(rename = "core_tone")]
    pub core_tone: String,
    #[serde(rename = "pacing_strategy")]
    pub pacing_strategy: String,
    #[serde(rename = "anti_patterns")]
    pub anti_patterns: Vec<String>,
    #[serde(rename = "world_rules")]
    pub world_rules: Vec<String>,
}

/// 章节合同结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterContract {
    #[serde(rename = "schema_version")]
    pub schema_version: String,
    #[serde(rename = "contract_type")]
    pub contract_type: String,
    #[serde(rename = "generator_version")]
    pub generator_version: String,
    #[serde(rename = "chapter_number")]
    pub chapter_number: i32,
    #[serde(rename = "chapter_directive")]
    pub chapter_directive: ChapterDirective,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterDirective {
    pub goal: String,
    #[serde(rename = "must_cover_nodes")]
    pub must_cover_nodes: Vec<String>,
    #[serde(rename = "forbidden_zones")]
    pub forbidden_zones: Vec<String>,
    #[serde(rename = "time_anchor")]
    pub time_anchor: Option<String>,
    #[serde(rename = "chapter_span")]
    pub chapter_span: Option<String>,
}

/// 运行时合同（写前加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeContract {
    pub master_setting: MasterSettingContract,
    pub chapter_contract: Option<ChapterContract>,
}

/// 合同履行度结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentResult {
    pub score: f64,
    pub covered_nodes: Vec<String>,
    pub violated_rules: Vec<String>,
    pub forbidden_zones_hit: Vec<String>,
}

impl RuntimeContract {
    /// 评估正文对运行时合同的履行情况。
    ///
    /// 规则：
    /// - 覆盖每个 must_cover_node 加分；未覆盖扣分。
    /// - 触及每个 forbidden_zone 扣分并记录。
    /// - 正文未体现 world_rules 时轻微扣分（作为简单一致性代理）。
    pub fn evaluate_fulfillment(&self, content: &str) -> FulfillmentResult {
        let mut covered_nodes = Vec::new();
        let mut violated_rules = Vec::new();
        let mut forbidden_zones_hit = Vec::new();
        let mut score = 1.0_f64;

        if let Some(ch) = &self.chapter_contract {
            for node in &ch.chapter_directive.must_cover_nodes {
                if node.is_empty() {
                    continue;
                }
                if content.contains(node) {
                    covered_nodes.push(node.clone());
                } else {
                    score -= 0.15;
                }
            }

            for zone in &ch.chapter_directive.forbidden_zones {
                if zone.is_empty() {
                    continue;
                }
                if content.contains(zone) {
                    forbidden_zones_hit.push(zone.clone());
                    score -= 0.25;
                }
            }
        }

        // 简单的世界规则代理：未出现则视为可能未遵守
        for rule in &self.master_setting.world_rules {
            if rule.is_empty() {
                continue;
            }
            if !content.contains(rule) {
                violated_rules.push(format!("可能未体现规则: {}", rule));
                score -= 0.05;
            }
        }

        // 空内容保护
        if content.trim().is_empty() {
            score = 0.0;
            violated_rules.push("内容为空".to_string());
        }

        score = score.clamp(0.0, 1.0);

        FulfillmentResult {
            score,
            covered_nodes,
            violated_rules,
            forbidden_zones_hit,
        }
    }

    /// 判断正文是否完全满足合同条件。
    ///
    /// 当前实现以履行度评分达到 1.0 为“完全满足”。
    pub fn fulfills(&self, content: &str) -> bool {
        self.evaluate_fulfillment(content).score >= 1.0
    }

    /// 将合同转换为 prompt 模板变量表。
    /// 配合 PromptRegistry 中的 writer_contract_constraints /
    /// inspector_contract_compliance / write_time_bundle_contract /
    /// review_contract_criteria / refine_contract_criteria 使用。
    pub fn to_constraint_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        let master = &self.master_setting;

        vars.insert("core_tone".to_string(), master.core_tone.clone());
        vars.insert(
            "pacing_strategy".to_string(),
            master.pacing_strategy.clone(),
        );
        vars.insert(
            "world_rules".to_string(),
            if master.world_rules.is_empty() {
                "无".to_string()
            } else {
                master
                    .world_rules
                    .iter()
                    .enumerate()
                    .map(|(i, r)| format!("{}. {}", i + 1, r))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
        );

        if let Some(ref ch) = self.chapter_contract {
            vars.insert(
                "chapter_goal".to_string(),
                ch.chapter_directive.goal.clone(),
            );
            vars.insert(
                "must_cover_nodes".to_string(),
                if ch.chapter_directive.must_cover_nodes.is_empty() {
                    "无".to_string()
                } else {
                    ch.chapter_directive
                        .must_cover_nodes
                        .iter()
                        .enumerate()
                        .map(|(i, n)| format!("{}. {}", i + 1, n))
                        .collect::<Vec<_>>()
                        .join("\n")
                },
            );
            vars.insert(
                "forbidden_zones".to_string(),
                if ch.chapter_directive.forbidden_zones.is_empty() {
                    "无".to_string()
                } else {
                    ch.chapter_directive
                        .forbidden_zones
                        .iter()
                        .enumerate()
                        .map(|(i, n)| format!("{}. {}", i + 1, n))
                        .collect::<Vec<_>>()
                        .join("\n")
                },
            );
            return vars;
        }

        vars.insert("chapter_goal".to_string(), "（未指定）".to_string());
        vars.insert("must_cover_nodes".to_string(), "无".to_string());
        vars.insert("forbidden_zones".to_string(), "无".to_string());
        vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_contract() -> RuntimeContract {
        RuntimeContract {
            master_setting: MasterSettingContract {
                schema_version: "1".to_string(),
                contract_type: "MASTER_SETTING".to_string(),
                generator_version: "0.22.5".to_string(),
                genre: "玄幻".to_string(),
                core_tone: "黑暗压抑".to_string(),
                pacing_strategy: "慢热铺陈".to_string(),
                anti_patterns: vec![],
                world_rules: vec!["灵气不可再生".to_string()],
            },
            chapter_contract: Some(ChapterContract {
                schema_version: "1".to_string(),
                contract_type: "CHAPTER".to_string(),
                generator_version: "0.22.5".to_string(),
                chapter_number: 1,
                chapter_directive: ChapterDirective {
                    goal: "主角发现真相".to_string(),
                    must_cover_nodes: vec!["主角出场".to_string(), "灵气异常".to_string()],
                    forbidden_zones: vec!["提前揭示反派".to_string()],
                    time_anchor: None,
                    chapter_span: None,
                },
            }),
        }
    }

    #[test]
    fn perfect_fulfillment_scores_one() {
        let contract = sample_contract();
        let content = "主角出场，发现灵气异常，这个世界灵气不可再生。";
        let result = contract.evaluate_fulfillment(content);
        assert!((result.score - 1.0).abs() < f64::EPSILON);
        assert_eq!(result.covered_nodes.len(), 2);
        assert!(result.forbidden_zones_hit.is_empty());
        assert!(contract.fulfills(content));
    }

    #[test]
    fn missing_nodes_and_forbidden_zone_reduce_score() {
        let contract = sample_contract();
        let content = "提前揭示反派身份，但主角还没有出场。";
        let result = contract.evaluate_fulfillment(content);
        assert!(result.score < 1.0);
        assert!(result.score > 0.0);
        assert_eq!(result.forbidden_zones_hit.len(), 1);
        assert!(result.covered_nodes.is_empty());
        assert!(!contract.fulfills(content));
    }

    #[test]
    fn fulfills_returns_false_for_empty_content() {
        let contract = sample_contract();
        assert!(!contract.fulfills(""));
        let result = contract.evaluate_fulfillment("");
        assert_eq!(result.score, 0.0);
        assert!(result.violated_rules.iter().any(|r| r.contains("内容为空")));
    }

    #[test]
    fn fulfills_without_chapter_contract_uses_world_rules() {
        let mut contract = sample_contract();
        contract.chapter_contract = None;
        assert!(contract.fulfills("灵气不可再生"));
        assert!(!contract.fulfills("与合同无关的内容"));
    }
}
