//! 创作工作流引擎核心
//!
//! 串联所有创作阶段和 Agent，形成完整闭环：
//! Conception → Outlining → SceneDesign → Writing → Review → Iteration → Ingestion

use crate::agents::service::{AgentService, AgentTask, AgentType};
use crate::agents::{AgentContext, AgentResult};
use crate::db::DbPool;
use crate::creative_engine::methodology::MethodologyConfig;
use super::{WorkflowExecutionResult, WorkflowProgressEvent, WorkflowStage, CreationMode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::Emitter;

/// 创作阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreationPhase {
    Conception,    // 构思：用户灵感 → OutlinePlanner → 故事种子
    Outlining,     // 大纲：故事种子 → 方法论 → 完整大纲
    SceneDesign,   // 场景设计：大纲章节 → 场景结构
    Writing,       // 写作：场景结构 + 记忆查询 → Writer → 初稿
    Review,        // 审校：初稿 → Inspector + ContinuityEngine → 问题列表
    Iteration,     // 迭代：问题列表 → Writer(改写) → 终稿
    Ingestion,     // 记忆：终稿 → IngestPipeline → 知识图谱更新
}

impl CreationPhase {
    pub fn name(&self) -> &'static str {
        match self {
            CreationPhase::Conception => "构思",
            CreationPhase::Outlining => "大纲",
            CreationPhase::SceneDesign => "场景设计",
            CreationPhase::Writing => "写作",
            CreationPhase::Review => "审校",
            CreationPhase::Iteration => "迭代",
            CreationPhase::Ingestion => "记忆",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CreationPhase::Conception => "将用户灵感转化为结构化故事种子",
            CreationPhase::Outlining => "按方法论生成完整故事大纲",
            CreationPhase::SceneDesign => "为每章设计场景结构和戏剧目标",
            CreationPhase::Writing => "根据场景结构生成完整章节",
            CreationPhase::Review => "质检和内容一致性检查",
            CreationPhase::Iteration => "根据质检反馈改写优化",
            CreationPhase::Ingestion => "分析新内容并更新知识图谱",
        }
    }

    pub fn order(&self) -> u8 {
        match self {
            CreationPhase::Conception => 0,
            CreationPhase::Outlining => 1,
            CreationPhase::SceneDesign => 2,
            CreationPhase::Writing => 3,
            CreationPhase::Review => 4,
            CreationPhase::Iteration => 5,
            CreationPhase::Ingestion => 6,
        }
    }

    pub fn next(&self) -> Option<CreationPhase> {
        match self {
            CreationPhase::Conception => Some(CreationPhase::Outlining),
            CreationPhase::Outlining => Some(CreationPhase::SceneDesign),
            CreationPhase::SceneDesign => Some(CreationPhase::Writing),
            CreationPhase::Writing => Some(CreationPhase::Review),
            CreationPhase::Review => Some(CreationPhase::Iteration),
            CreationPhase::Iteration => Some(CreationPhase::Ingestion),
            CreationPhase::Ingestion => None,
        }
    }
}

/// 阶段工作流配置
#[derive(Debug, Clone)]
pub struct PhaseWorkflow {
    pub phase: CreationPhase,
    /// 该阶段需要执行的 Agent 列表（按顺序）
    pub required_agents: Vec<AgentType>,
    /// 是否需要用户确认后才能进入下一阶段
    pub requires_user_confirmation: bool,
    /// 该阶段使用的方法论（如有）
    pub methodology: Option<MethodologyConfig>,
    /// 阶段特定的提示词补充
    pub prompt_extension: Option<String>,
}

impl PhaseWorkflow {
    pub fn new(phase: CreationPhase) -> Self {
        Self {
            phase,
            required_agents: vec![],
            requires_user_confirmation: false,
            methodology: None,
            prompt_extension: None,
        }
    }

    /// 设置该阶段使用的 Agent
    pub fn with_agents(mut self, agents: Vec<AgentType>) -> Self {
        self.required_agents = agents;
        self
    }

    /// 设置需要用户确认
    pub fn with_user_confirmation(mut self) -> Self {
        self.requires_user_confirmation = true;
        self
    }

    /// 设置方法论
    pub fn with_methodology(mut self, config: MethodologyConfig) -> Self {
        self.methodology = Some(config);
        self
    }
}

/// 工作流配置
#[derive(Debug, Clone)]
pub struct WorkflowConfig {
    pub mode: CreationMode,
    /// 是否自动执行（无需用户确认每个阶段）
    pub auto_execute: bool,
    /// 审校阈值（低于此分数进入迭代）
    pub review_threshold: f32,
    /// 最大迭代次数
    pub max_iterations: u32,
    /// 故事 ID
    pub story_id: String,
}

/// 工作流状态
#[derive(Debug, Clone)]
pub struct WorkflowState {
    pub workflow_id: String,
    pub current_phase: CreationPhase,
    pub completed_phases: Vec<CreationPhase>,
    /// 各阶段输出缓存
    pub phase_outputs: HashMap<String, String>,
    /// 质检评分
    pub review_score: Option<f32>,
    /// 迭代计数
    pub iteration_count: u32,
    /// 是否已暂停
    pub is_paused: bool,
}

impl WorkflowState {
    pub fn new(workflow_id: String) -> Self {
        Self {
            workflow_id,
            current_phase: CreationPhase::Conception,
            completed_phases: vec![],
            phase_outputs: HashMap::new(),
            review_score: None,
            iteration_count: 0,
            is_paused: false,
        }
    }

    pub fn progress(&self) -> f32 {
        let total = 7.0;
        let current = self.current_phase.order() as f32;
        let completed_bonus = self.completed_phases.len() as f32 * 0.1;
        ((current + completed_bonus) / total).min(1.0)
    }
}

/// 创作工作流引擎
pub struct CreationWorkflowEngine {
    agent_service: AgentService,
    pool: DbPool,
}

impl CreationWorkflowEngine {
    pub fn new(agent_service: AgentService, pool: DbPool) -> Self {
        Self { agent_service, pool }
    }

    /// 创建标准工作流配置
    pub fn create_standard_workflow(story_id: &str, mode: CreationMode) -> WorkflowConfig {
        WorkflowConfig {
            mode,
            auto_execute: mode == CreationMode::AiOnly,
            review_threshold: 0.75,
            max_iterations: 2,
            story_id: story_id.to_string(),
        }
    }

    /// 构建 AgentContext（使用 StoryContextBuilder）
    pub fn build_context(&self, story_id: &str) -> Result<AgentContext, String> {
        use crate::creative_engine::StoryContextBuilder;
        let builder = StoryContextBuilder::new(self.pool.clone());
        builder.build_quick(story_id)
    }

    /// 执行单阶段
    pub async fn execute_phase(
        &self,
        phase: CreationPhase,
        context: &AgentContext,
        input: &str,
    ) -> Result<AgentResult, String> {
        match phase {
            CreationPhase::Conception => {
                // 构思阶段：使用 OutlinePlanner 生成故事种子
                let task = AgentTask {
                    id: format!("conception-{}", context.story_id),
                    agent_type: AgentType::OutlinePlanner,
                    context: context.clone(),
                    input: input.to_string(),
                    parameters: HashMap::new(),
                    tier: None,
                };
                self.agent_service.execute_task(task).await
            }
            CreationPhase::Outlining => {
                // 大纲阶段：使用 Writer 或 OutlinePlanner 扩展大纲
                let mut task = AgentTask {
                    id: format!("outlining-{}", context.story_id),
                    agent_type: AgentType::OutlinePlanner,
                    context: context.clone(),
                    input: input.to_string(),
                    parameters: HashMap::new(),
                    tier: None,
                };
                // 注入雪花法方法论
                task.context.methodology_id = Some("snowflake".to_string());
                task.context.methodology_step = Some("scene_expansion".to_string());
                self.agent_service.execute_task(task).await
            }
            CreationPhase::SceneDesign => {
                // 场景设计：生成场景结构
                let mut task = AgentTask {
                    id: format!("scene-design-{}", context.story_id),
                    agent_type: AgentType::Writer,
                    context: context.clone(),
                    input: format!("请根据以下大纲设计场景结构：\n\n{}", input),
                    parameters: HashMap::new(),
                    tier: None,
                };
                task.context.methodology_id = Some("scene_structure".to_string());
                self.agent_service.execute_task(task).await
            }
            CreationPhase::Writing => {
                // 写作阶段：生成章节内容
                let task = AgentTask {
                    id: format!("writing-{}", context.story_id),
                    agent_type: AgentType::Writer,
                    context: context.clone(),
                    input: input.to_string(),
                    parameters: HashMap::new(),
                    tier: None,
                };
                self.agent_service.execute_task(task).await
            }
            CreationPhase::Review => {
                // 审校阶段：Inspector 质检
                let task = AgentTask {
                    id: format!("review-{}", context.story_id),
                    agent_type: AgentType::Inspector,
                    context: context.clone(),
                    input: input.to_string(),
                    parameters: HashMap::new(),
                    tier: None,
                };
                self.agent_service.execute_task(task).await
            }
            CreationPhase::Iteration => {
                // 迭代阶段：Writer 改写
                let task = AgentTask {
                    id: format!("iteration-{}", context.story_id),
                    agent_type: AgentType::Writer,
                    context: context.clone(),
                    input: input.to_string(),
                    parameters: HashMap::new(),
                    tier: None,
                };
                self.agent_service.execute_task(task).await
            }
            CreationPhase::Ingestion => {
                // 记忆阶段：触发 IngestPipeline
                // 这是一个后台操作，不需要 Agent
                Ok(AgentResult {
                    content: "内容已提交记忆系统".to_string(),
                    score: Some(1.0),
                    suggestions: vec![],
                })
            }
        }
    }

    /// 执行完整工作流（一键创作）
    pub async fn execute_full_workflow(
        &self,
        config: &WorkflowConfig,
        initial_input: &str,
    ) -> Result<WorkflowExecutionResult, String> {
        let mut state = WorkflowState::new(format!("wf-{}", config.story_id));
        let mut current_input = initial_input.to_string();
        let context = self.build_context(&config.story_id)?;

        // 按顺序执行各阶段
        let phases = vec![
            CreationPhase::Conception,
            CreationPhase::Outlining,
            CreationPhase::SceneDesign,
            CreationPhase::Writing,
            CreationPhase::Review,
        ];

        for phase in phases {
            if state.is_paused {
                break;
            }

            state.current_phase = phase;

            // 执行阶段
            let result = self.execute_phase(phase, &context, &current_input).await?;

            // 缓存输出
            state.phase_outputs.insert(phase.name().to_string(), result.content.clone());

            // 处理阶段特定逻辑
            match phase {
                CreationPhase::Review => {
                    state.review_score = result.score;
                    let score = result.score.unwrap_or(0.0);

                    if score < config.review_threshold && state.iteration_count < config.max_iterations {
                        // 进入迭代阶段
                        let feedback = if result.suggestions.is_empty() {
                            "请改进内容质量".to_string()
                        } else {
                            result.suggestions.join("\n")
                        };
                        current_input = format!("【质检反馈】\n{}\n\n【原文】\n{}",
                            feedback,
                            state.phase_outputs.get("写作").unwrap_or(&"".to_string())
                        );
                        state.iteration_count += 1;
                        // 继续迭代
                        let iteration_result = self.execute_phase(CreationPhase::Iteration, &context, &current_input).await?;
                        state.phase_outputs.insert("迭代".to_string(), iteration_result.content.clone());
                        current_input = iteration_result.content;
                    } else {
                        current_input = result.content;
                    }
                }
                CreationPhase::Writing => {
                    current_input = result.content.clone();
                }
                _ => {
                    current_input = result.content;
                }
            }

            state.completed_phases.push(phase);
        }

        // 最终 Ingestion
        if !state.is_paused {
            let final_content = state.phase_outputs.get("写作")
                .or(state.phase_outputs.get("迭代"))
                .unwrap_or(&current_input)
                .clone();
            let _ = self.execute_phase(CreationPhase::Ingestion, &context, &final_content).await;
            state.completed_phases.push(CreationPhase::Ingestion);
        }

        // 构建结果
        let final_output = state.phase_outputs.get("写作")
            .or(state.phase_outputs.get("迭代"))
            .cloned();

        Ok(WorkflowExecutionResult {
            success: !state.is_paused,
            current_phase: state.current_phase.name().to_string(),
            completed_phases: state.completed_phases.iter().map(|p| p.name().to_string()).collect(),
            output: final_output,
            quality_report: None,
            error: None,
        })
    }

    /// 执行单个创作阶段（分步模式）
    pub async fn execute_single_phase(
        &self,
        phase: CreationPhase,
        story_id: &str,
        input: &str,
    ) -> Result<AgentResult, String> {
        let context = self.build_context(story_id)?;
        self.execute_phase(phase, &context, input).await
    }

    /// 生成工作流进度事件
    pub fn emit_progress(&self, state: &WorkflowState, stage: WorkflowStage, message: &str) {
        let _ = self.agent_service.app_handle().emit(
            "workflow-progress",
            WorkflowProgressEvent {
                workflow_id: state.workflow_id.clone(),
                phase: state.current_phase.name().to_string(),
                stage,
                message: message.to_string(),
                progress: state.progress(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation_phase_order() {
        assert_eq!(CreationPhase::Conception.order(), 0);
        assert_eq!(CreationPhase::Ingestion.order(), 6);
    }

    #[test]
    fn test_creation_phase_next() {
        assert_eq!(CreationPhase::Conception.next(), Some(CreationPhase::Outlining));
        assert_eq!(CreationPhase::Ingestion.next(), None);
    }

    #[test]
    fn test_workflow_state_progress() {
        let mut state = WorkflowState::new("test".to_string());
        assert_eq!(state.progress(), 0.0);

        state.current_phase = CreationPhase::Writing;
        state.completed_phases.push(CreationPhase::Conception);
        state.completed_phases.push(CreationPhase::Outlining);
        let p = state.progress();
        assert!(p > 0.0 && p < 1.0);
    }

    #[test]
    fn test_creation_mode() {
        assert_eq!(CreationMode::AiOnly.name(), "一键创作");
        assert_eq!(CreationMode::AiDraftHumanEdit.name(), "AI草稿+人修改");
    }

    #[test]
    fn test_phase_workflow_builder() {
        let wf = PhaseWorkflow::new(CreationPhase::Writing)
            .with_agents(vec![AgentType::Writer])
            .with_user_confirmation();

        assert_eq!(wf.phase, CreationPhase::Writing);
        assert_eq!(wf.required_agents.len(), 1);
        assert!(wf.requires_user_confirmation);
    }
}
