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
pub mod bootstrap;
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
    #[serde(default)]
    pub steps: Vec<PlanStep>,
    #[serde(default)]
    pub fallback_message: String,
}

/// 场景结构摘要（用于计划生成）
#[derive(Debug, Clone)]
pub struct SceneStructureSummary {
    pub scene_id: String,
    pub sequence_number: i32,
    pub title: Option<String>,
    pub execution_stage: Option<String>,
    pub has_content: bool,
    pub word_count: usize,
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
    // Phase 3: 场景/章节结构感知
    pub scene_count: usize,
    pub scenes_summary: Vec<SceneStructureSummary>,
    pub current_scene_id: Option<String>,
    pub current_scene_stage: Option<String>,
    pub total_word_count: usize,
    pub latest_chapter_word_count: usize,
    pub story_progress: String, // "just_started" | "developing" | "midpoint" | "climax" | "resolution"
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

        // Build scene structure summary for prompt
        let scenes_summary = if context.scenes_summary.is_empty() {
            "No scenes yet".to_string()
        } else {
            context.scenes_summary.iter()
                .map(|s| {
                    let stage = s.execution_stage.as_deref().unwrap_or("unknown");
                    let title = s.title.as_deref().unwrap_or("Untitled");
                    let content_flag = if s.has_content { "✓" } else { "○" };
                    format!("  #{} [{}] {} {} ({} words)", s.sequence_number, stage, title, content_flag, s.word_count)
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        let current_scene_info = if let Some(ref id) = context.current_scene_id {
            format!("Current scene ID: {} (stage: {})", id, context.current_scene_stage.as_deref().unwrap_or("unknown"))
        } else {
            "No current scene".to_string()
        };

        let prompt = format!(
            r#"You are an intelligent orchestrator for a creative writing application.

Current system state:
- Has story: {}
- Story ID: {}
- Has chapters: {}
- Chapter count: {}
- Total word count: {}
- Latest chapter words: {}
- Story progress: {}
- Scene count: {}
{}

Scene structure:
{}

Current content preview: {}

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
7. Available capability_id values include: writer, inspector, outline_planner, style_mimic, plot_analyzer, create_story, create_chapter, create_character, builtin.style_enhancer, builtin.plot_twist, builtin.text_formatter, builtin.character_voice, builtin.emotion_pacing.
8. CRITICAL: If the user wants to continue writing and the current scene has no content or is in 'planning'/'outline' stage, use 'writer' to generate draft content.
9. If the user wants to improve/refine text and there IS content, use 'inspector' first then 'writer'.
10. If story progress is 'just_started' and user asks for next chapter/scene, use 'create_chapter' or 'outline_planner' first.
11. If scenes are stuck in 'planning' or 'outline' stage, prioritize 'writer' to move them to 'drafting'."#,
            context.has_story,
            context.current_story_id.as_deref().unwrap_or("none"),
            context.has_chapters,
            context.chapter_count,
            context.total_word_count,
            context.latest_chapter_word_count,
            context.story_progress,
            context.scene_count,
            current_scene_info,
            scenes_summary,
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
