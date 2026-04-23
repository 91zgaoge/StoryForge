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
        if let Some(template_plan) = self.find_template(&context.user_input) {
            log::info!(
                "[PlanExecutor] Using template plan for input: {}",
                context.user_input
            );
            let adapted_plan = self.adapt_template_plan(template_plan, context);
            Ok(self.execute_plan(adapted_plan).await)
        } else {
            let llm_service = crate::llm::LlmService::new(self.app_handle.clone());
            let generator = PlanGenerator::new(llm_service);
            let plan = generator.generate_plan(context).await?;
            Ok(self.execute_plan(plan).await)
        }
    }

    pub async fn execute_plan(&self, plan: ExecutionPlan) -> PlanExecutionResult {
        let mut messages = Vec::new();
        let mut step_outputs: HashMap<String, serde_json::Value> = HashMap::new();
        let mut steps_completed = 0;
        let mut final_content: Option<String> = None;

        log::info!("[PlanExecutor] Understanding: {}", plan.understanding);
        log::info!("[PlanExecutor] Executing {} steps", plan.steps.len());

        for step in &plan.steps {
            let step_start = std::time::Instant::now();
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

            let resolved_params = self.resolve_parameters(&step.parameters, &step_outputs);
            let result = self.execute_step(step, &resolved_params).await;
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

    async fn execute_step(&self, step: &PlanStep, params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
        match step.capability_id.as_str() {
            "create_story" => self.execute_create_story(params).await,
            "create_chapter" => self.execute_create_chapter(params).await,
            "create_character" => self.execute_create_character(params).await,
            "writer" => self.execute_writer(params).await,
            "inspector" => self.execute_inspector(params).await,
            "outline_planner" => self.execute_outline_planner(params).await,
            "style_mimic" => self.execute_style_mimic(params).await,
            "plot_analyzer" => self.execute_plot_analyzer(params).await,
            skill_id if skill_id.starts_with("builtin.") => self.execute_skill(skill_id, params).await,
            _ => Err(format!("Unknown capability: {}", step.capability_id)),
        }
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

    async fn execute_writer(&self, params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let instruction = params.get("instruction").and_then(|v| v.as_str()).unwrap_or("Continue the story").to_string();

        let service = crate::agents::service::AgentService::new(self.app_handle.clone());
        let task = crate::agents::service::AgentTask {
            id: Uuid::new_v4().to_string(),
            agent_type: crate::agents::service::AgentType::Writer,
            context: crate::agents::AgentContext::minimal(story_id.clone(), instruction.clone()),
            input: instruction,
            parameters: params.clone(),
            tier: None,
        };

        let result = service.execute_task(task).await?;
        Ok(serde_json::json!({
            "content": result.content,
            "score": result.score,
        }))
    }

    async fn execute_inspector(&self, params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let draft = params.get("draft").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let service = crate::agents::service::AgentService::new(self.app_handle.clone());
        let task = crate::agents::service::AgentTask {
            id: Uuid::new_v4().to_string(),
            agent_type: crate::agents::service::AgentType::Inspector,
            context: crate::agents::AgentContext::minimal(story_id, draft.clone()),
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

    async fn execute_outline_planner(&self, params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let premise = params.get("premise").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let service = crate::agents::service::AgentService::new(self.app_handle.clone());
        let task = crate::agents::service::AgentTask {
            id: Uuid::new_v4().to_string(),
            agent_type: crate::agents::service::AgentType::OutlinePlanner,
            context: crate::agents::AgentContext::minimal(story_id, premise.clone()),
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

    async fn execute_style_mimic(&self, params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let service = crate::agents::service::AgentService::new(self.app_handle.clone());
        let mut task_params = params.clone();
        task_params.insert("style_sample".to_string(), params.get("style_sample").cloned().unwrap_or(serde_json::Value::Null));

        let task = crate::agents::service::AgentTask {
            id: Uuid::new_v4().to_string(),
            agent_type: crate::agents::service::AgentType::StyleMimic,
            context: crate::agents::AgentContext::minimal(story_id, content.clone()),
            input: content,
            parameters: task_params,
            tier: None,
        };

        let result = service.execute_task(task).await?;
        Ok(serde_json::json!({"content": result.content}))
    }

    async fn execute_plot_analyzer(&self, params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let service = crate::agents::service::AgentService::new(self.app_handle.clone());
        let task = crate::agents::service::AgentTask {
            id: Uuid::new_v4().to_string(),
            agent_type: crate::agents::service::AgentType::PlotAnalyzer,
            context: crate::agents::AgentContext::minimal(story_id, content.clone()),
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

    async fn execute_skill(&self, skill_id: &str, params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
        let story_id = params.get("story_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let mut params = params.clone();
        params.insert("story_id".to_string(), serde_json::Value::String(story_id));

        let manager = crate::SKILL_MANAGER.get().ok_or("Skill manager not initialized")?;
        let skill_manager = manager.lock().map_err(|e| e.to_string())?.clone();

        let agent_context = crate::agents::AgentContext::minimal(
            params.get("story_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            String::new(),
        );

        let result = skill_manager.execute_skill(skill_id, &agent_context, params).await?;

        if !result.success {
            return Err(result.error.unwrap_or_else(|| "Skill execution failed".to_string()));
        }

        Ok(result.data)
    }
}
