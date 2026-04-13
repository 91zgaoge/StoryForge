//! Intent Parser - 意图解析引擎
//!
//! 将创作者的自然语言输入解析为结构化意图，
//! 驱动 workflow::scheduler 调用正确的 Agent 执行创作任务。

use crate::llm::{GenerateResponse, LlmService};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

/// 意图类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IntentType {
    TextGenerate,
    TextRewrite,
    PlotSuggest,
    CharacterCheck,
    WorldConsistency,
    StyleShift,
    MemoryIngest,
    VisualGenerate,
    SceneReorder,
    OutlineExpand,
    Unknown,
}

/// 执行模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Serial,
    Parallel,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode::Serial
    }
}

/// 反馈类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackType {
    DirectApply,
    SuggestionCard,
    DiffPreview,
    SystemNotice,
    VisualHighlight,
}

impl Default for FeedbackType {
    fn default() -> Self {
        FeedbackType::SuggestionCard
    }
}

/// 意图目标
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntentTarget {
    pub target_type: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
}

/// 结构化意图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    #[serde(rename = "intent_type")]
    pub intent_type: IntentType,
    #[serde(default)]
    pub target: IntentTarget,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub required_agents: Vec<String>,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    #[serde(default)]
    pub feedback_type: FeedbackType,
    /// 原始用户输入（补充字段，不由LLM生成）
    #[serde(skip)]
    pub raw_input: String,
}

impl Intent {
    pub fn unknown(raw_input: impl Into<String>) -> Self {
        Self {
            intent_type: IntentType::Unknown,
            target: IntentTarget::default(),
            constraints: vec![],
            required_agents: vec![],
            execution_mode: ExecutionMode::default(),
            feedback_type: FeedbackType::default(),
            raw_input: raw_input.into(),
        }
    }
}

/// 意图解析器
pub struct IntentParser {
    llm_service: LlmService,
}

impl IntentParser {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            llm_service: LlmService::new(app_handle),
        }
    }

    /// 解析用户输入为结构化意图
    pub async fn parse(&self, user_input: &str) -> Result<Intent, String> {
        let prompt = Self::build_intent_prompt(user_input);
        
        match self.llm_service.generate(prompt, Some(512), Some(0.1)).await {
            Ok(GenerateResponse { content, .. }) => {
                Self::parse_intent_json(&content, user_input)
            }
            Err(e) => {
                log::error!("[IntentParser] LLM generation failed: {}", e);
                Ok(Intent::unknown(user_input))
            }
        }
    }

    fn build_intent_prompt(user_input: &str) -> String {
        format!(
            r#"你是一个专业的创作助手意图解析器。请将用户的输入解析为固定的 JSON 格式。

可识别的意图类型 (intent_type):
- text_generate: 文本续写、扩展内容
- text_rewrite: 改写、润色已有文本
- plot_suggest: 情节建议、反转设计、剧情推进
- character_check: 角色一致性检查、角色动机分析
- world_consistency: 世界设定一致性检查
- style_shift: 文风切换、文风模仿
- memory_ingest: 知识摄取、更新记忆
- visual_generate: 生成图像、概念图
- scene_reorder: 场景结构调整、排序
- outline_expand: 大纲扩展
- unknown: 无法识别或闲聊

执行模式 (execution_mode):
- serial: 串行执行（默认）
- parallel: 并行执行

反馈类型 (feedback_type):
- direct_apply: 直接修改（适用于续写）
- suggestion_card: 建议卡片（适用于情节建议）
- diff_preview: Diff预览（适用于改写）
- system_notice: 系统通知（适用于异步任务）
- visual_highlight: 可视化高亮（适用于检查结果）

可用 Agent (required_agents):
- writer
- style_mimic
- plot_analyzer
- outline_planner
- character_agent
- world_building_agent
- memory_agent
- inspector

规则:
1. 必须且只能返回合法的 JSON，不要包含 markdown 代码块标记。
2. target 字段用于指明操作对象，如场景、角色等。target_type 可选值: scene, character, story, paragraph。
3. constraints 是用户对结果的具体约束条件列表。
4. 如果用户只是打招呼或闲聊，返回 intent_type: unknown。

JSON Schema:
{{
  "intent_type": "string",
  "target": {{
    "target_type": "string | null",
    "id": "string | null",
    "name": "string | null"
  }},
  "constraints": ["string"],
  "required_agents": ["string"],
  "execution_mode": "serial | parallel",
  "feedback_type": "direct_apply | suggestion_card | diff_preview | system_notice | visual_highlight"
}}

用户输入: "{}"

请直接输出 JSON:"#,
            user_input
        )
    }

    fn parse_intent_json(content: &str, user_input: &str) -> Result<Intent, String> {
        // 尝试清理可能存在的 markdown 代码块
        let json_str = content
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        match serde_json::from_str::<Intent>(json_str) {
            Ok(mut intent) => {
                intent.raw_input = user_input.to_string();
                Ok(intent)
            }
            Err(e) => {
                log::warn!(
                    "[IntentParser] Failed to parse JSON: {}. Raw content: {}",
                    e,
                    content
                );
                Ok(Intent::unknown(user_input))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_intent_json() {
        let json = r#"{
            "intent_type": "text_rewrite",
            "target": {"target_type": "scene", "id": "scene_2", "name": null},
            "constraints": ["增强紧张感", "保持 K-7 语气"],
            "required_agents": ["writer", "style_mimic"],
            "execution_mode": "serial",
            "feedback_type": "diff_preview"
        }"#;

        let intent = IntentParser::parse_intent_json(json, "把 Scene 2 改得更紧张").unwrap();
        assert_eq!(intent.intent_type, IntentType::TextRewrite);
        assert_eq!(intent.target.target_type, Some("scene".to_string()));
        assert_eq!(intent.target.id, Some("scene_2".to_string()));
        assert_eq!(intent.constraints.len(), 2);
        assert_eq!(intent.required_agents, vec!["writer", "style_mimic"]);
        assert_eq!(intent.execution_mode, ExecutionMode::Serial);
        assert_eq!(intent.feedback_type, FeedbackType::DiffPreview);
    }

    #[test]
    fn test_parse_intent_json_with_markdown() {
        let json = "```json\n{\"intent_type\": \"plot_suggest\", \"target\": {}, \"constraints\": [], \"required_agents\": [\"plot_analyzer\"], \"execution_mode\": \"serial\", \"feedback_type\": \"suggestion_card\"}\n```";

        let intent = IntentParser::parse_intent_json(json, "帮我想个反转").unwrap();
        assert_eq!(intent.intent_type, IntentType::PlotSuggest);
    }

    #[test]
    fn test_parse_intent_json_fallback() {
        let invalid = "这不是 JSON";
        let intent = IntentParser::parse_intent_json(invalid, "你好").unwrap();
        assert_eq!(intent.intent_type, IntentType::Unknown);
    }
}
