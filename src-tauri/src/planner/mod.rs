//! Plan Generator - 智能执行计划生成器
//!
//! 将用户的自然语言输入转化为结构化的执行计划，
//! 替代旧的 IntentParser + IntentExecutor 分类标签方式。
//! 核心设计：LLM 自由理解用户意图，自主选择能力组合，无预设分类。

use crate::capabilities::build_default_registry;
use crate::llm::LlmService;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod executor;
pub mod template_learning;
pub use template_learning::PlanTemplateLibrary;
#[allow(unused_imports)]
pub use template_learning::PlanTemplate;
pub use executor::{PlanExecutor, PlanExecutionResult};

/// 执行计划中的单个步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub step_id: String,
    pub capability_id: String,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// 完整的执行计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    #[serde(default)]
    pub understanding: String,
    pub steps: Vec<PlanStep>,
    #[serde(default)]
    pub fallback_message: String,
}

/// 生成计划所需的上下文
#[derive(Debug, Clone)]
pub struct PlanContext {
    pub current_story_id: Option<String>,
    pub has_story: bool,
    pub has_chapters: bool,
    pub chapter_count: usize,
    pub current_content_preview: Option<String>,
    pub user_input: String,
}

/// 计划生成器
pub struct PlanGenerator {
    llm_service: LlmService,
}

impl PlanGenerator {
    pub fn new(llm_service: LlmService) -> Self {
        Self { llm_service }
    }

    /// 根据用户输入和系统状态生成执行计划
    pub async fn generate_plan(&self, context: &PlanContext) -> Result<ExecutionPlan, String> {
        let registry = build_default_registry();
        let registry_context = registry.to_llm_context();

        // Sanitize inputs to prevent prompt injection / format breakage
        fn sanitize_for_prompt(s: &str) -> String {
            s.replace('"', "'")
                .replace('\n', " ")
                .replace('\r', "")
                .replace("{{", "〔")
                .replace("}}", "〕")
        }

        let preview = context.current_content_preview.as_deref().unwrap_or("none");
        let user_input_clean = sanitize_for_prompt(&context.user_input);
        let preview_clean = sanitize_for_prompt(preview);
        let registry_clean = sanitize_for_prompt(&registry_context);

        let prompt = format!(
            r#"You are an intelligent orchestrator for a creative writing application.

Current system state:
- Has story: {}
- Story ID: {}
- Has chapters: {}
- Chapter count: {}
- Current content preview: {}

User input: "{}"

{}

Your task: Analyze the user's intent and generate an execution plan.

Respond with JSON:
{{
  "understanding": "Your understanding of what the user wants (free text, not categories)",
  "steps": [
    {{
      "step_id": "step_1",
      "capability_id": "writer",
      "purpose": "Why this capability is chosen",
      "parameters": {{"story_id": "...", "instruction": "..."}},
      "depends_on": []
    }}
  ],
  "fallback_message": "If the plan fails, tell the user this..."
}}

Rules:
1. Do NOT use classification labels or keyword matching in your reasoning.
2. Choose capabilities based on what the user actually needs.
3. Use depends_on to order steps when one step needs another's output.
4. step_id must be unique within the plan.
5. fallback_message should be helpful if execution fails.
6. For parameters, you can reference output from a previous step using {{step_id}} syntax in string values.
7. Available capability_id values include: writer, inspector, outline_planner, style_mimic, plot_analyzer, create_story, create_chapter, create_character, builtin.style_enhancer, builtin.plot_twist, builtin.text_formatter, builtin.character_voice, builtin.emotion_pacing."#,
            context.has_story,
            context.current_story_id.as_deref().unwrap_or("none"),
            context.has_chapters,
            context.chapter_count,
            preview_clean,
            user_input_clean,
            registry_clean
        );

        let response = self.llm_service.generate(prompt, Some(2048), Some(0.3)).await?;

        // Robust JSON extraction: find first '{' and last '}'
        let content = response.content.trim();
        let json_str = if let (Some(start), Some(end)) = (content.find('{'), content.rfind('}')) {
            &content[start..=end]
        } else {
            // Fallback to markdown code block stripping
            content
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim()
        };

        let mut plan: ExecutionPlan = serde_json::from_str(json_str).map_err(|e| {
            format!(
                "Failed to parse plan JSON: {}. Extracted JSON: {}",
                e, json_str
            )
        })?;

        // 验证计划：确保所有 capability_id 在注册表中存在
        let registry = build_default_registry();
        plan.steps.retain(|step| {
            if registry.get_by_id(&step.capability_id).is_none() {
                log::warn!(
                    "[PlanGenerator] Removing step '{}' with unknown capability '{}'",
                    step.step_id,
                    step.capability_id
                );
                false
            } else {
                true
            }
        });

        Ok(plan)
    }
}
