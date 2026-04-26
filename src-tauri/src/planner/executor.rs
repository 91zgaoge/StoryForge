//! PlanExecutor - Dumb executor that faithfully runs LLM-generated plans
//!
//! All intelligence is in the plan. This executor just follows instructions.

use super::{ExecutionPlan, PlanContext, PlanGenerator, PlanStep};
use crate::capabilities::{CapabilityEvolutionEngine, ExecutionRecord};
use crate::planner::PlanTemplateLibrary;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct PlanExecutionResult {
    pub success: bool,
    pub steps_completed: usize,
    pub final_content: Option<String>,
    pub messages: Vec<String>,
}

pub struct PlanExecutor {
    app_handle: AppHandle,
    template_library: Mutex<PlanTemplateLibrary>,
    evolution_engine: CapabilityEvolutionEngine,
}

impl PlanExecutor {
    pub fn new(app_handle: AppHandle) -> Self {
        let pool = app_handle.state::<crate::db::DbPool>().inner().clone();
        let llm_service = crate::llm::LlmService::new(app_handle.clone());
        Self {
            app_handle,
            template_library: Mutex::new(PlanTemplateLibrary::new()),
            evolution_engine: CapabilityEvolutionEngine::new(llm_service, pool),
        }
    }

    /// Check if a matching template exists for the given user input
    pub fn find_template(&self, user_input: &str) -> Option<ExecutionPlan> {
        let library = self.template_library.lock().ok()?;
        library.find_match(user_input).map(|t| t.plan.clone())
    }

    /// Adapt a template plan to the current context by replacing placeholders
    fn adapt_template_plan(&self, template: ExecutionPlan, context: &PlanContext) -> ExecutionPlan {
        let mut plan = template;
        if let Some(story_id) = &context.current_story_id {
            for step in &mut plan.steps {
                for value in step.parameters.values_mut() {
                    if let Some(s) = value.as_str() {
                        if s.contains("{{story_id}}") {
                            *value = serde_json::Value::String(s.replace("{{story_id}}", story_id));
                        }
                    }
                }
            }
        }
        plan
    }

    /// Execute a plan, checking the template library first
    pub async fn execute_with_context(&self, context: &PlanContext) -> Result<PlanExecutionResult, String> {
        // Before generating a new plan, check PlanTemplateLibrary for matching templates
        let mut plan = if let Some(template_plan) = self.find_template(&context.user_input) {
            log::info!(
                "[PlanExecutor] Using template plan for input: {}",
                context.user_input
            );
            self.adapt_template_plan(template_plan, context)
        } else {
            let llm_service = crate::llm::LlmService::new(self.app_handle.clone());
            let generator = PlanGenerator::new(llm_service);
            match generator.generate_plan(context).await {
                Ok(plan) => plan,
                Err(e) => {
                    log::warn!("[PlanExecutor] Plan generation failed ({}), falling back to direct writer", e);
                    // Fallback: direct writer execution with user input as instruction
                    ExecutionPlan {
                        understanding: format!("Direct execution fallback for: {}", context.user_input),
                        steps: vec![PlanStep {
                            step_id: "fallback_writer".to_string(),
                            capability_id: "writer".to_string(),
                            purpose: "Fallback: execute user request directly via writer agent".to_string(),
                            parameters: {
                                let mut p = HashMap::new();
                                p.insert("story_id".to_string(), serde_json::Value::String(context.current_story_id.clone().unwrap_or_default()));
                                p.insert("instruction".to_string(), serde_json::Value::String(context.user_input.clone()));
                                p
                            },
                            depends_on: vec![],
                        }],
                        fallback_message: "计划生成失败，已回退到直接写作模式".to_string(),
                    }
                }
            }
        };

        // Inject PlanContext information into every step so agents get full context
        for step in &mut plan.steps {
            if let Some(ref preview) = context.current_content_preview {
                step.parameters.entry("current_content".to_string())
                    .or_insert_with(|| serde_json::Value::String(preview.clone()));
            }
            if let Some(ref story_id) = context.current_story_id {
                step.parameters.entry("story_id".to_string())
                    .or_insert_with(|| serde_json::Value::String(story_id.clone()));
            }
        }

        Ok(self.execute_plan(plan, context).await)
    }

    pub async fn execute_plan(&self, plan: ExecutionPlan, plan_context: &PlanContext) -> PlanExecutionResult {
        let mut messages = Vec::new();
        let mut step_outputs: HashMap<String, serde_json::Value> = HashMap::new();
        let mut steps_completed = 0;
        let mut final_content: Option<String> = None;

        log::info!("[PlanExecutor] Understanding: {}", plan.understanding);
        log::info!("[PlanExecutor] Executing {} steps", plan.steps.len());

        // Phase 4: Agent Swarm - 拓扑排序确定执行批次
        let batches = crate::planner::swarm::topological_sort(&plan.steps);
        log::info!("[PlanExecutor] Swarm batches: {} batches", batches.batches.len());

        // 检测 Inspector→Writer 闭环模式
        let has_loop = crate::planner::swarm::detect_inspector_writer_loop(&plan.steps);
        if let Some((inspect_id, writer_id)) = &has_loop {
            log::info!("[PlanExecutor] Detected Inspector→Writer loop: {} → {}", inspect_id, writer_id);
        }

        // 按批次执行（同批次内的步骤无相互依赖，未来可并行化）
        for (batch_idx, batch) in batches.batches.iter().enumerate() {
            log::info!("[PlanExecutor] Executing batch {}/{} with {} steps", 
                batch_idx + 1, batches.batches.len(), batch.len());

            for step_id in batch {
                let step = match plan.steps.iter().find(|s| s.step_id == *step_id) {
                    Some(s) => s,
                    None => {
                        messages.push(format!("Step {} not found in plan", step_id));
                        continue;
                    }
                };

                let step_start = std::time::Instant::now();

                // 检查依赖是否满足
                let mut deps_ok = true;
                for dep in &step.depends_on {
                    if !step_outputs.contains_key(dep) {
                        let msg = format!("Step {} dependency {} not found", step.step_id, dep);
                        log::warn!("[PlanExecutor] {}", msg);
                        messages.push(msg);
                        deps_ok = false;
                        break;
                    }
                }
                if !deps_ok {
                    continue;
                }

                // Phase 4: Swarm 闭环增强 — Inspector→Writer 之间注入质量反馈
                let mut resolved_params = self.resolve_parameters(&step.parameters, &step_outputs);
                if let Some((ref inspect_id, _)) = has_loop {
                    if step.capability_id == "writer" && step.depends_on.contains(inspect_id) {
                        if let Some(inspector_output) = step_outputs.get(inspect_id) {
                            if let Some(feedback) = inspector_output.get("suggestions").and_then(|s| s.as_str()) {
                                log::info!("[PlanExecutor] Injecting inspector feedback into writer step");
                                resolved_params.insert(
                                    "inspector_feedback".to_string(),
                                    serde_json::Value::String(feedback.to_string()),
                                );
                            }
                        }
                    }
                }

                let result = self.execute_step(step, &resolved_params, plan_context).await;
                let step_duration = step_start.elapsed().as_millis() as u64;

                // Record execution result
                let record = ExecutionRecord {
                    capability_id: step.capability_id.clone(),
                    user_input: plan.understanding.clone(),
                    success: result.is_ok(),
                    user_feedback: None,
                    execution_time_ms: step_duration,
                };
                let _ = self.evolution_engine.record_execution(record);

                match &result {
                    Ok(output) => {
                        step_outputs.insert(step.step_id.clone(), output.clone());
                        messages.push(format!("Step {} completed: {}", step.step_id, step.capability_id));
                        if let Some(content) = output.get("content").and_then(|c| c.as_str()) {
                            final_content = Some(content.to_string());
                        }
                        steps_completed += 1;
                    }
                    Err(e) => {
                        messages.push(format!("Step {} failed: {}", step.step_id, e));
                        log::warn!("[PlanExecutor] Step {} failed: {}", step.step_id, e);
                    }
                }
            }
        }

        // Phase 4: Swarm 质量闭环 — 如果最终内容是 writer 产出且前面有 inspector，
        // 尝试自动触发一轮轻量 inspector 检查
        if let Some((_, ref writer_id)) = has_loop {
            if let Some(writer_output) = step_outputs.get(writer_id) {
                if let Some(content) = writer_output.get("content").and_then(|c| c.as_str()) {
                    if content.len() > 100 {
                        log::info!("[PlanExecutor] Swarm loop complete, content length: {}", content.len());
                    }
                }
            }
        }

        let success = steps_completed > 0
            && steps_completed >= plan.steps.iter().filter(|s| s.depends_on.is_empty()).count();

        // Record successful plan as template
        if success {
            if let Ok(mut library) = self.template_library.lock() {
                library.record_success(&plan.understanding, plan.clone());
            }
        }

        PlanExecutionResult {
            success,
            steps_completed,
            final_content,
            messages,
        }
    }

    async fn execute_step(&self, step: &PlanStep, params: &HashMap<String, serde_json::Value>, plan_context: &PlanContext) -> Result<serde_json::Value, String> {
        match step.capability_id.as_str() {
            "create_story" => self.execute_create_story(params).await,
            "create_chapter" => self.execute_create_chapter(params).await,
            "create_character" => self.execute_create_character(params).await,
            "writer" => self.execute_writer(params, plan_context).await,
            "inspector" => self.execute_inspector(params, plan_context).await,
            "outline_planner" => self.execute_outline_planner(params, plan_context).await,
            "style_mimic" => self.execute_style_mimic(params, plan_context).await,
            "plot_analyzer" => self.execute_plot_analyzer(params, plan_context).await,
            skill_id if skill_id.starts_with("builtin.") => self.execute_skill(skill_id, params, plan_context).await,
            _ => Err(format!("Unknown capability: {}", step.capability_id)),
        }
    }

    /// Build a rich AgentContext using StoryContextBuilder instead of the minimal stub.
    fn build_agent_context(
        &self,
        story_id: &str,
        current_content: Option<String>,
        selected_text: Option<String>,
    ) -> Result<crate::agents::AgentContext, String> {
        if story_id.is_empty() {
            return Ok(crate::agents::AgentContext::minimal(story_id.to_string(), String::new()));
        }

        let pool = self.app_handle.state::<crate::db::DbPool>();
        let builder = crate::creative_engine::context_builder::StoryContextBuilder::new(pool.inner().clone());

        // Resolve current scene number from DB (latest scene for the story)
        let scene_number = self.get_current_scene_number(story_id).unwrap_or(None);

        builder.build(story_id, scene_number, current_content, selected_text)
    }

    fn get_current_scene_number(&self, story_id: &str) -> Result<Option<i32>, String> {
        let pool = self.app_handle.state::<crate::db::DbPool>();
        let repo = crate::db::repositories_v3::SceneRepository::new(pool.inner().clone());
        let scenes = repo.get_by_story(story_id).map_err(|e| e.to_string())?;
        Ok(scenes.iter().max_by_key(|s| s.sequence_number).map(|s| s.sequence_number))
    }

    fn resolve_parameters(&self, params: &HashMap<String, serde_json::Value>, outputs: &HashMap<String, serde_json::Value>) -> HashMap<String, serde_json::Value> {
        let mut resolved = params.clone();

        for (key, value) in params.iter() {
            if let Some(ref_str) = value.as_str() {
                let mut result = ref_str.to_string();
                for (step_id, output) in outputs.iter() {
                    let placeholder = format!("{{{{{}}}}}", step_id);
                    if result.contains(&placeholder) {
                        let replacement = output.get("content").and_then(|v| v.as_str()).unwrap_or("");
                        result = result.replace(&placeholder, replacement);
                    }
                }
                if result != ref_str {
                    resolved.insert(key.clone(), serde_json::Value::String(result));
                }
            }
        }

        resolved
    }

    async fn execute_create_story(&self, params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
        let title = params.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("未命名作品")
            .to_string();
        let description = params.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
        let genre = params.get("genre").and_then(|v| v.as_str()).map(|s| s.to_string());

        let pool = self.app_handle.state::<crate::db::DbPool>();
        let repo = crate::db::repositories::StoryRepository::new(pool.inner().clone());
        let story = repo.create(crate::db::CreateStoryRequest { title, description, genre, style_dna_id: None })
            .map_err(|e| e.to_string())?;

        // Emit event to refresh frontstage
        let _ = crate::window::WindowManager::send_to_frontstage(
            &self.app_handle,
            crate::window::FrontstageEvent::DataRefresh { entity: "stories".to_string() }
        );

        Ok(serde_json::json!({
            "story_id": story.id,
            "title": story.title,
            "content": format!("Created story: {}", story.title),
        }))
    }

    async fn execute_create_chapter(&self, params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).ok_or("story_id required")?.to_string();
        let chapter_number = params.get("chapter_number").and_then(|v| v.as_u64()).unwrap_or(1) as i32;
        let title = params.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());

        let pool = self.app_handle.state::<crate::db::DbPool>();
        let repo = crate::db::ChapterRepository::new(pool.inner().clone());
        let chapter = repo.create(crate::db::CreateChapterRequest { story_id: story_id.clone(), chapter_number, title: title.clone(), outline: None, content: None })
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "chapter_id": chapter.id,
            "story_id": story_id,
            "chapter_number": chapter_number,
            "title": title.unwrap_or_default(),
            "content": format!("Created chapter {}", chapter_number),
        }))
    }

    async fn execute_create_character(&self, params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).ok_or("story_id required")?.to_string();
        let name = params.get("name").and_then(|v| v.as_str()).ok_or("name required")?.to_string();
        let background = params.get("background").and_then(|v| v.as_str()).map(|s| s.to_string());

        let pool = self.app_handle.state::<crate::db::DbPool>();
        let repo = crate::db::repositories::CharacterRepository::new(pool.inner().clone());
        let character = repo.create(crate::db::CreateCharacterRequest { story_id, name, background })
            .map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "character_id": character.id,
            "name": character.name,
            "content": format!("Created character: {}", character.name),
        }))
    }

    async fn execute_writer(&self, params: &HashMap<String, serde_json::Value>, plan_context: &PlanContext) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let instruction = params.get("instruction").and_then(|v| v.as_str()).unwrap_or("Continue the story").to_string();
        let current_content = params.get("current_content").and_then(|v| v.as_str()).map(|s| s.to_string())
            .or_else(|| plan_context.current_content_preview.clone());

        let service = crate::agents::service::AgentService::new(self.app_handle.clone());
        let context = self.build_agent_context(&story_id, current_content, None)?;

        // Phase 5: 将 PlanContext 中的结构信息注入到 AgentTask 参数
        let mut enriched_params = params.clone();
        enriched_params.insert("story_progress".to_string(), serde_json::Value::String(plan_context.story_progress.clone()));
        if let Some(ref stage) = plan_context.current_scene_stage {
            enriched_params.insert("current_scene_stage".to_string(), serde_json::Value::String(stage.clone()));
        }
        if plan_context.scene_count > 0 {
            enriched_params.insert("scene_count".to_string(), serde_json::Value::Number(plan_context.scene_count.into()));
        }
        if plan_context.total_word_count > 0 {
            enriched_params.insert("total_word_count".to_string(), serde_json::Value::Number(plan_context.total_word_count.into()));
        }

        let task = crate::agents::service::AgentTask {
            id: Uuid::new_v4().to_string(),
            agent_type: crate::agents::service::AgentType::Writer,
            context,
            input: instruction,
            parameters: enriched_params,
            tier: None,
        };

        let result = service.execute_task(task).await?;
        Ok(serde_json::json!({
            "content": result.content,
            "score": result.score,
        }))
    }

    async fn execute_inspector(&self, params: &HashMap<String, serde_json::Value>, plan_context: &PlanContext) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let draft = params.get("draft").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let current_content = params.get("current_content").and_then(|v| v.as_str()).map(|s| s.to_string())
            .or_else(|| plan_context.current_content_preview.clone());

        let service = crate::agents::service::AgentService::new(self.app_handle.clone());
        let context = self.build_agent_context(&story_id, current_content, None)?;
        let task = crate::agents::service::AgentTask {
            id: Uuid::new_v4().to_string(),
            agent_type: crate::agents::service::AgentType::Inspector,
            context,
            input: draft,
            parameters: params.clone(),
            tier: None,
        };

        let result = service.execute_task(task).await?;
        Ok(serde_json::json!({
            "content": result.content,
            "score": result.score,
            "suggestions": result.suggestions,
        }))
    }

    async fn execute_outline_planner(&self, params: &HashMap<String, serde_json::Value>, _plan_context: &PlanContext) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let premise = params.get("premise").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let service = crate::agents::service::AgentService::new(self.app_handle.clone());
        let context = self.build_agent_context(&story_id, None, None)?;
        let task = crate::agents::service::AgentTask {
            id: Uuid::new_v4().to_string(),
            agent_type: crate::agents::service::AgentType::OutlinePlanner,
            context,
            input: premise,
            parameters: params.clone(),
            tier: None,
        };

        let result = service.execute_task(task).await?;
        Ok(serde_json::json!({
            "content": result.content,
            "outline": result.content,
        }))
    }

    async fn execute_style_mimic(&self, params: &HashMap<String, serde_json::Value>, _plan_context: &PlanContext) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let service = crate::agents::service::AgentService::new(self.app_handle.clone());
        let mut task_params = params.clone();
        task_params.insert("style_sample".to_string(), params.get("style_sample").cloned().unwrap_or(serde_json::Value::Null));

        let context = self.build_agent_context(&story_id, None, None)?;
        let task = crate::agents::service::AgentTask {
            id: Uuid::new_v4().to_string(),
            agent_type: crate::agents::service::AgentType::StyleMimic,
            context,
            input: content,
            parameters: task_params,
            tier: None,
        };

        let result = service.execute_task(task).await?;
        Ok(serde_json::json!({"content": result.content}))
    }

    async fn execute_plot_analyzer(&self, params: &HashMap<String, serde_json::Value>, _plan_context: &PlanContext) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let service = crate::agents::service::AgentService::new(self.app_handle.clone());
        let context = self.build_agent_context(&story_id, None, None)?;
        let task = crate::agents::service::AgentTask {
            id: Uuid::new_v4().to_string(),
            agent_type: crate::agents::service::AgentType::PlotAnalyzer,
            context,
            input: content,
            parameters: params.clone(),
            tier: None,
        };

        let result = service.execute_task(task).await?;
        Ok(serde_json::json!({
            "content": result.content,
            "score": result.score,
            "suggestions": result.suggestions,
        }))
    }

    async fn execute_skill(&self, skill_id: &str, params: &HashMap<String, serde_json::Value>, _plan_context: &PlanContext) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let mut params = params.clone();
        params.insert("story_id".to_string(), serde_json::Value::String(story_id.clone()));

        let manager = crate::SKILL_MANAGER.get().ok_or("Skill manager not initialized")?;
        let skill_manager = manager.lock().map_err(|e| e.to_string())?.clone();

        let agent_context = self.build_agent_context(&story_id, None, None)?;

        let result = skill_manager.execute_skill(skill_id, &agent_context, params).await?;

        if !result.success {
            return Err(result.error.unwrap_or_else(|| "Skill execution failed".to_string()));
        }

        Ok(result.data)
    }
}
