//! Workflow Node Task Executor
//!
//! 将 workflow 节点执行接入 task_system，使 scheduler 不再直接依赖
//! agents / llm / ingest 等具体实现。

use std::collections::HashMap;

use tauri::{AppHandle, Manager};

use crate::{
    agents::{
        orchestrator::{AgentOrchestrator, GenerationMode, WorkflowConfig},
        service::{AgentService, AgentTask, AgentType},
    },
    db::DbPool,
    domain::agent_context::AgentContext,
    llm::LlmService,
    memory::ingest::{IngestContent, IngestPipeline},
    task_system::{
        executor::{TaskExecutionContext, TaskExecutor},
        models::*,
    },
};

pub struct WorkflowNodeExecutor {
    pool: DbPool,
    app_handle: AppHandle,
}

impl WorkflowNodeExecutor {
    pub fn new(app_handle: AppHandle) -> Self {
        let pool = app_handle.state::<DbPool>().inner().clone();
        Self { pool, app_handle }
    }

    fn build_orchestrator(&self) -> AgentOrchestrator {
        let agent_service = AgentService::new(self.app_handle.clone());
        let app_dir = self.app_handle.path().app_data_dir().unwrap_or_default();
        let config = crate::config::AppConfig::load(&app_dir)
            .map(|c| WorkflowConfig::from_app_config(&c))
            .unwrap_or_default();
        AgentOrchestrator::new(agent_service, config, self.app_handle.clone())
    }

    fn minimal_context(&self, story_id: &str) -> AgentContext {
        AgentContext::minimal(story_id.to_string(), String::new())
    }
}

#[derive(Debug, serde::Deserialize)]
struct WorkflowNodePayload {
    instance_id: String,
    node_id: String,
    story_id: String,
    node_type: String,
    input: String,
    #[serde(default)]
    parameters: HashMap<String, serde_json::Value>,
}

#[async_trait::async_trait]
impl TaskExecutor for WorkflowNodeExecutor {
    fn can_handle(&self, task_type: &TaskType) -> bool {
        *task_type == TaskType::WorkflowNode
    }

    async fn execute(&self, task: &Task) -> Result<TaskResult, Box<dyn std::error::Error>> {
        let ctx =
            TaskExecutionContext::new(task.id.clone(), self.pool.clone(), self.app_handle.clone());

        let payload: WorkflowNodePayload = match task.payload.as_deref() {
            Some(p) => match serde_json::from_str(p) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(TaskResult {
                        success: false,
                        result_json: None,
                        error_message: Some(format!("Invalid workflow node payload: {}", e)),
                    });
                }
            },
            None => {
                return Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some("Missing workflow node payload".to_string()),
                });
            }
        };

        if payload.story_id.is_empty() {
            return Ok(TaskResult {
                success: false,
                result_json: None,
                error_message: Some("Missing story_id in workflow node payload".to_string()),
            });
        }

        ctx.update_progress("prepare", 5, "准备工作流节点执行...");
        ctx.heartbeat();

        match payload.node_type.as_str() {
            "WriteChapter" => self.execute_write_chapter(&ctx, &payload).await,
            "Inspect" => self.execute_inspect(&ctx, &payload).await,
            "Revise" => self.execute_revise(&ctx, &payload).await,
            "AnalyzePlot" => self.execute_analyze_plot(&ctx, &payload).await,
            "VectorIndex" => self.execute_vector_index(&ctx, &payload).await,
            other => Ok(TaskResult {
                success: false,
                result_json: None,
                error_message: Some(format!("Unsupported workflow node type: {}", other)),
            }),
        }
    }
}

impl WorkflowNodeExecutor {
    async fn execute_write_chapter(
        &self,
        ctx: &TaskExecutionContext,
        payload: &WorkflowNodePayload,
    ) -> Result<TaskResult, Box<dyn std::error::Error>> {
        ctx.update_progress("generate", 30, "执行写作节点...");
        ctx.heartbeat();

        let orchestrator = self.build_orchestrator();
        let agent_task = AgentTask {
            id: uuid::Uuid::new_v4().to_string(),
            agent_type: AgentType::Writer,
            context: self.minimal_context(&payload.story_id),
            input: payload.input.clone(),
            parameters: HashMap::new(),
            tier: None,
        };

        match orchestrator
            .generate(agent_task, GenerationMode::Full)
            .await
        {
            Ok(result) => {
                let result_json = serde_json::to_string(&serde_json::json!({
                    "content": result.final_content,
                    "score": result.final_score,
                    "was_rewritten": result.was_rewritten,
                    "rewrite_count": result.rewrite_count,
                    "request_id": result.request_id,
                }))?;
                ctx.update_progress("complete", 100, "写作节点完成");
                Ok(TaskResult {
                    success: true,
                    result_json: Some(result_json),
                    error_message: None,
                })
            }
            Err(e) => {
                ctx.log("error", &format!("写作节点失败: {}", e));
                Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some(format!("Writer execution failed: {}", e)),
                })
            }
        }
    }

    async fn execute_inspect(
        &self,
        ctx: &TaskExecutionContext,
        payload: &WorkflowNodePayload,
    ) -> Result<TaskResult, Box<dyn std::error::Error>> {
        if payload.input.is_empty() {
            return Ok(TaskResult {
                success: true,
                result_json: Some(serde_json::to_string(&serde_json::json!({
                    "content": "",
                    "score": 0.0,
                    "warning": "No content to inspect",
                }))?),
                error_message: None,
            });
        }

        ctx.update_progress("inspect", 30, "执行审校节点...");
        ctx.heartbeat();

        let agent_service = AgentService::new(self.app_handle.clone());
        let task = AgentTask {
            id: uuid::Uuid::new_v4().to_string(),
            agent_type: AgentType::Inspector,
            context: self.minimal_context(&payload.story_id),
            input: payload.input.clone(),
            parameters: HashMap::new(),
            tier: None,
        };

        match agent_service.execute_task(task).await {
            Ok(result) => {
                let result_json = serde_json::to_string(&serde_json::json!({
                    "content": result.content,
                    "score": result.score,
                }))?;
                ctx.update_progress("complete", 100, "审校节点完成");
                Ok(TaskResult {
                    success: true,
                    result_json: Some(result_json),
                    error_message: None,
                })
            }
            Err(e) => {
                ctx.log("error", &format!("审校节点失败: {}", e));
                Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some(format!("Inspector execution failed: {}", e)),
                })
            }
        }
    }

    async fn execute_revise(
        &self,
        ctx: &TaskExecutionContext,
        payload: &WorkflowNodePayload,
    ) -> Result<TaskResult, Box<dyn std::error::Error>> {
        if payload.input.is_empty() {
            return Ok(TaskResult {
                success: true,
                result_json: Some(serde_json::to_string(&serde_json::json!({
                    "content": "",
                    "score": 0.0,
                    "warning": "No content to revise",
                }))?),
                error_message: None,
            });
        }

        ctx.update_progress("revise", 30, "执行修订节点...");
        ctx.heartbeat();

        let orchestrator = self.build_orchestrator();
        let agent_task = AgentTask {
            id: uuid::Uuid::new_v4().to_string(),
            agent_type: AgentType::Writer,
            context: self.minimal_context(&payload.story_id),
            input: payload.input.clone(),
            parameters: HashMap::new(),
            tier: None,
        };

        match orchestrator
            .generate(agent_task, GenerationMode::Full)
            .await
        {
            Ok(result) => {
                let result_json = serde_json::to_string(&serde_json::json!({
                    "content": result.final_content,
                    "score": Some(result.final_score as f64),
                    "request_id": result.request_id,
                }))?;
                ctx.update_progress("complete", 100, "修订节点完成");
                Ok(TaskResult {
                    success: true,
                    result_json: Some(result_json),
                    error_message: None,
                })
            }
            Err(e) => {
                ctx.log("error", &format!("修订节点失败: {}", e));
                Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some(format!("Revision failed: {}", e)),
                })
            }
        }
    }

    async fn execute_analyze_plot(
        &self,
        ctx: &TaskExecutionContext,
        payload: &WorkflowNodePayload,
    ) -> Result<TaskResult, Box<dyn std::error::Error>> {
        if payload.input.is_empty() {
            return Ok(TaskResult {
                success: true,
                result_json: Some(serde_json::to_string(&serde_json::json!({
                    "content": "",
                    "score": 0.0,
                    "warning": "No content to analyze",
                }))?),
                error_message: None,
            });
        }

        ctx.update_progress("analyze", 30, "执行情节分析节点...");
        ctx.heartbeat();

        let agent_service = AgentService::new(self.app_handle.clone());
        let task = AgentTask {
            id: uuid::Uuid::new_v4().to_string(),
            agent_type: AgentType::PlotAnalyzer,
            context: self.minimal_context(&payload.story_id),
            input: payload.input.clone(),
            parameters: HashMap::new(),
            tier: None,
        };

        match agent_service.execute_task(task).await {
            Ok(result) => {
                let result_json = serde_json::to_string(&serde_json::json!({
                    "content": result.content,
                    "score": result.score,
                }))?;
                ctx.update_progress("complete", 100, "情节分析节点完成");
                Ok(TaskResult {
                    success: true,
                    result_json: Some(result_json),
                    error_message: None,
                })
            }
            Err(e) => {
                ctx.log("error", &format!("情节分析节点失败: {}", e));
                Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some(format!("Plot analysis failed: {}", e)),
                })
            }
        }
    }

    async fn execute_vector_index(
        &self,
        ctx: &TaskExecutionContext,
        payload: &WorkflowNodePayload,
    ) -> Result<TaskResult, Box<dyn std::error::Error>> {
        if payload.input.len() <= 50 {
            return Ok(TaskResult {
                success: true,
                result_json: Some(serde_json::to_string(&serde_json::json!({
                    "indexed": false,
                    "reason": "content too short",
                }))?),
                error_message: None,
            });
        }

        ctx.update_progress("index", 30, "执行向量索引节点...");
        ctx.heartbeat();

        let llm_service = LlmService::new(self.app_handle.clone());
        let pipeline = IngestPipeline::new(llm_service).with_pool(self.pool.clone());
        let ingest_content = IngestContent {
            text: payload.input.clone(),
            source: format!("workflow:{}", payload.instance_id),
            story_id: payload.story_id.clone(),
            scene_id: None,
        };

        match pipeline.ingest(&ingest_content).await {
            Ok(result) => {
                let result_json = serde_json::to_string(&serde_json::json!({
                    "indexed": true,
                    "entities": result.entities.len(),
                    "relations": result.relations.len(),
                }))?;
                ctx.update_progress("complete", 100, "向量索引节点完成");
                Ok(TaskResult {
                    success: true,
                    result_json: Some(result_json),
                    error_message: None,
                })
            }
            Err(e) => {
                log::warn!("[WorkflowNodeExecutor] Ingest failed: {}", e);
                let result_json = serde_json::to_string(&serde_json::json!({
                    "indexed": false,
                    "error": e.to_string(),
                }))?;
                Ok(TaskResult {
                    success: true,
                    result_json: Some(result_json),
                    error_message: None,
                })
            }
        }
    }
}
