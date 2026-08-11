#![allow(dead_code)]
//! 指导书提炼 Models

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

// ==================== 提炼状态 ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DistillationStatus {
    Pending,
    Extracting,
    Distilling,
    Merging,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for DistillationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DistillationStatus::Pending => "pending",
            DistillationStatus::Extracting => "extracting",
            DistillationStatus::Distilling => "distilling",
            DistillationStatus::Merging => "merging",
            DistillationStatus::Completed => "completed",
            DistillationStatus::Failed => "failed",
            DistillationStatus::Cancelled => "cancelled",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for DistillationStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(DistillationStatus::Pending),
            "extracting" => Ok(DistillationStatus::Extracting),
            "distilling" => Ok(DistillationStatus::Distilling),
            "merging" => Ok(DistillationStatus::Merging),
            "completed" => Ok(DistillationStatus::Completed),
            "failed" => Ok(DistillationStatus::Failed),
            "cancelled" => Ok(DistillationStatus::Cancelled),
            _ => Err(format!("Unknown distillation status: {}", s)),
        }
    }
}

// ==================== 指导书主表模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guidebook {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub word_count: Option<i64>,
    pub file_format: Option<String>,
    pub file_hash: Option<String>,
    pub file_path: Option<String>,
    pub methodology_id: Option<String>,
    pub status: DistillationStatus,
    pub progress: i32,
    pub error: Option<String>,
    pub task_id: Option<String>,
    pub merge_into_methodology_id: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidebookListItem {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub word_count: Option<i64>,
    pub file_format: Option<String>,
    pub methodology_id: Option<String>,
    pub merge_into_methodology_id: Option<String>,
    pub status: String,
    pub progress: i32,
    pub created_at: String,
}

// ==================== 自定义方法论 ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MethodologyStep {
    pub title: String,
    pub instruction: String,
    #[serde(default)]
    pub checklist: Vec<String>,
}

/// 技巧模式库条目（提炼自指导书的具名技巧）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Technique {
    pub name: String,
    #[serde(default)]
    pub when_to_use: String,
    #[serde(default)]
    pub how: String,
}

/// 反模式（避免什么 + 为什么）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AntiPattern {
    pub what: String,
    #[serde(default)]
    pub why: String,
}

/// 决策速查表（决策规则 + 反模式）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Cheatsheet {
    #[serde(default)]
    pub decision_rules: Vec<String>,
    #[serde(default)]
    pub anti_patterns: Vec<AntiPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMethodology {
    pub id: String,
    pub guidebook_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<MethodologyStep>,
    #[serde(default)]
    pub patterns: Vec<Technique>,
    #[serde(default)]
    pub cheatsheet: Cheatsheet,
    pub enabled: bool,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl CustomMethodology {
    /// 最大步数（章节完成自动推进到顶后停留），至少为 1
    pub fn max_steps(&self) -> i32 {
        (self.steps.len() as i32).max(1)
    }
}

/// 解析 steps_json；坏数据返回空 vec（调用方按「无步骤」处理）
pub fn parse_steps(json: &str) -> Vec<MethodologyStep> {
    serde_json::from_str(json).unwrap_or_default()
}

/// 解析 patterns_json；坏数据返回空 vec
pub fn parse_patterns(json: &str) -> Vec<Technique> {
    serde_json::from_str(json).unwrap_or_default()
}

/// 解析 cheatsheet_json；坏数据返回默认空速查表
pub fn parse_cheatsheet(json: &str) -> Cheatsheet {
    serde_json::from_str(json).unwrap_or_default()
}

// ==================== 进度事件 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationProgressEvent {
    pub guidebook_id: String,
    pub status: String,
    pub progress: i32,
    pub current_step: String,
    pub message: Option<String>,
    #[serde(default)]
    pub active_threads: i32,
}

// ==================== LLM 响应类型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmGuidebookMetadataResponse {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmDistillChunkResponse {
    #[serde(default, alias = "key_points")]
    pub points: Vec<String>,
    #[serde(default)]
    pub techniques: Vec<Technique>,
    #[serde(default)]
    pub decision_rules: Vec<String>,
    #[serde(default)]
    pub anti_patterns: Vec<AntiPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmDistillMergeResponse {
    #[serde(default)]
    pub principles: Vec<String>,
    #[serde(default)]
    pub techniques: Vec<Technique>,
    #[serde(default)]
    pub decision_rules: Vec<String>,
    #[serde(default)]
    pub anti_patterns: Vec<AntiPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMethodologyStepResponse {
    pub title: String,
    pub instruction: String,
    #[serde(default)]
    pub checklist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMethodologyResponse {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<LlmMethodologyStepResponse>,
}

/// 提炼流水线的最终产出
#[derive(Debug, Clone)]
pub struct DistillationOutput {
    pub metadata: LlmGuidebookMetadataResponse,
    pub methodology: LlmMethodologyResponse,
    pub techniques: Vec<Technique>,
    pub cheatsheet: Cheatsheet,
}

// ==================== 聚合结果（给前端） ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidebookResult {
    pub guidebook: Guidebook,
    pub methodology: Option<CustomMethodology>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_roundtrip() {
        for s in [
            DistillationStatus::Pending,
            DistillationStatus::Extracting,
            DistillationStatus::Distilling,
            DistillationStatus::Merging,
            DistillationStatus::Completed,
            DistillationStatus::Failed,
            DistillationStatus::Cancelled,
        ] {
            let text = s.to_string();
            assert_eq!(text.parse::<DistillationStatus>().unwrap(), s);
        }
        assert!("bogus".parse::<DistillationStatus>().is_err());
    }

    #[test]
    fn parse_steps_handles_valid_and_invalid() {
        let json = r#"[{"title":"步骤一","instruction":"做某事","checklist":["a","b"]}]"#;
        let steps = parse_steps(json);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].checklist, vec!["a", "b"]);
        // checklist 缺省
        let no_checklist = parse_steps(r#"[{"title":"t","instruction":"i"}]"#);
        assert!(no_checklist[0].checklist.is_empty());
        // 坏 JSON → 空
        assert!(parse_steps("not json").is_empty());
    }

    #[test]
    fn parse_patterns_handles_valid_and_invalid() {
        let json =
            r#"[{"name":"雪花写作法","when_to_use":"搭建大纲时","how":"从一句话扩展到段落"}]"#;
        let p = parse_patterns(json);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].name, "雪花写作法");
        assert_eq!(p[0].when_to_use, "搭建大纲时");
        // 缺省字段容错
        let minimal = parse_patterns(r#"[{"name":"x"}]"#);
        assert_eq!(minimal[0].when_to_use, "");
        // 坏 JSON → 空
        assert!(parse_patterns("not json").is_empty());
    }

    #[test]
    fn parse_cheatsheet_handles_valid_and_invalid() {
        let json = r#"{"decision_rules":["当冲突弱化时加码，因为张力是引擎"],"anti_patterns":[{"what":"流水账","why":"没有冲突驱动"}]}"#;
        let cs = parse_cheatsheet(json);
        assert_eq!(cs.decision_rules.len(), 1);
        assert_eq!(cs.anti_patterns[0].what, "流水账");
        // 坏 JSON → 默认空
        let empty = parse_cheatsheet("not json");
        assert!(empty.decision_rules.is_empty());
        assert!(empty.anti_patterns.is_empty());
    }

    #[test]
    fn chunk_response_deserializes_structured_assets() {
        let json = r#"{"key_points":["要点一"],
          "techniques":[{"name":"雪花写作法","when_to_use":"搭大纲","how":"逐步扩展"}],
          "decision_rules":["当冲突弱时加码，因为张力是引擎"],
          "anti_patterns":[{"what":"流水账","why":"无冲突驱动"}]}"#;
        let r: LlmDistillChunkResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.points, vec!["要点一"]);
        assert_eq!(r.techniques[0].name, "雪花写作法");
        assert_eq!(r.decision_rules.len(), 1);
        assert_eq!(r.anti_patterns[0].why, "无冲突驱动");
    }

    #[test]
    fn chunk_response_backward_compat_old_points_format() {
        // 旧格式（只有 points）不崩溃，新字段为空
        let r: LlmDistillChunkResponse = serde_json::from_str(r#"{"points":["要点"]}"#).unwrap();
        assert_eq!(r.points, vec!["要点"]);
        assert!(r.techniques.is_empty());
        assert!(r.decision_rules.is_empty());
        assert!(r.anti_patterns.is_empty());
    }

    #[test]
    fn merge_response_deserializes_classified_assets() {
        let json = r#"{"principles":["原则一"],
          "techniques":[{"name":"t","when_to_use":"w","how":"h"}],
          "decision_rules":["r"],
          "anti_patterns":[{"what":"x","why":"y"}]}"#;
        let r: LlmDistillMergeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.principles, vec!["原则一"]);
        assert_eq!(r.techniques.len(), 1);
        assert_eq!(r.anti_patterns[0].what, "x");
        // 旧格式兼容
        let old: LlmDistillMergeResponse = serde_json::from_str(r#"{"principles":["p"]}"#).unwrap();
        assert!(old.techniques.is_empty());
    }

    #[test]
    fn max_steps_at_least_one() {
        let cm = CustomMethodology {
            id: "custom_x".into(),
            guidebook_id: None,
            name: "n".into(),
            description: None,
            steps: vec![],
            patterns: vec![],
            cheatsheet: Cheatsheet::default(),
            enabled: true,
            created_at: Local::now(),
            updated_at: Local::now(),
        };
        assert_eq!(cm.max_steps(), 1);
    }
}
