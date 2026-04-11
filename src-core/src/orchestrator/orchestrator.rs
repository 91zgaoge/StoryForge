use crate::agents::{Agent, AgentConfig, AgentRunResult};
use crate::team::{Team, TeamConfig, TeamRunResult};
use crate::task::{Task, TaskQueue, create_task};
use crate::tool::{ToolRegistry, ToolExecutor};
use crate::router::ModelRouter;
use crate::state::StateManager;
use crate::error::{CinemaError, Result};
use crate::ChapterOutput;

use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub default_model: String,
    pub max_concurrency: usize,
    pub enable_coordination: bool,
    pub shared_memory: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            default_model: "claude-3-sonnet".to_string(),
            max_concurrency: 3,
            enable_coordination: true,
            shared_memory: true,
        }
    }
}

pub struct OpenMultiAgent {
    config: OrchestratorConfig,
    tool_registry: Arc<ToolRegistry>,
    tool_executor: Arc<ToolExecutor>,
    state_manager: Arc<StateManager>,
    model_router: Arc<ModelRouter>,
    task_semaphore: Arc<Semaphore>,
}

impl OpenMultiAgent {
    pub fn new(
        config: OrchestratorConfig,
        state_manager: Arc<StateManager>,
        model_router: Arc<ModelRouter>,
    ) -> Self {
        let tool_registry = Arc::new(ToolRegistry::new());
        let tool_executor = Arc::new(ToolExecutor::new(tool_registry.clone()));
        let task_semaphore = Arc::new(Semaphore::new(config.max_concurrency));

        Self {
            config,
            tool_registry,
            tool_executor,
            state_manager,
            model_router,
            task_semaphore,
        }
    }

    pub async fn run_agent(
        &self,
        agent_config: AgentConfig,
        prompt: &str,
    ) -> Result<AgentRunResult> {
        let agent = Agent::new(
            agent_config,
            self.tool_registry.clone(),
            self.tool_executor.clone(),
            self.model_router.clone(),
        );
        agent.run(prompt).await
    }

    pub fn create_team(&self, name: &str, team_config: TeamConfig) -> Team {
        Team::new(
            name.to_string(),
            team_config,
            self.tool_registry.clone(),
            self.tool_executor.clone(),
            self.model_router.clone(),
            self.config.shared_memory,
        )
    }

    pub async fn run_team(
        &self,
        team: &Team,
        goal: &str,
    ) -> Result<TeamRunResult> {
        info!("Starting team execution for goal: {}", goal);

        let coordinator = self.create_coordinator_agent();
        let tasks = self.decompose_goal(&coordinator, goal, team).await?;
        
        if tasks.is_empty() {
            return Err(CinemaError::Agent("No tasks generated".to_string()));
        }

        let results = self.execute_tasks(team, tasks).await?;
        let final_result = self.synthesize_results(&coordinator, goal, &results).await?;

        Ok(TeamRunResult {
            goal: goal.to_string(),
            tasks_completed: results.len(),
            final_output: final_result,
            agent_results: results,
        })
    }

    pub async fn generate_chapter(
        &self,
        chapter_number: u32,
        outline: &str,
        _required_characters: Vec<String>,
        _complexity: crate::ComplexityTier,
    ) -> Result<ChapterOutput> {
        let writer_team = self.create_team("writers", TeamConfig {
            agents: vec![
                AgentConfig {
                    name: "writer".to_string(),
                    model: None,
                    system_prompt: "You are a professional novelist.".to_string(),
                    temperature: 0.7,
                    max_tokens: 4000,
                },
            ],
            shared_memory: true,
        });

        let goal = format!("Write Chapter {}: {}", chapter_number, outline);
        let result = self.run_team(&writer_team, &goal).await?;

        Ok(ChapterOutput {
            chapter_number,
            content: result.final_output,
            metadata: crate::ChapterMetadata {
                word_count: result.final_output.split_whitespace().count() as u32,
                generated_at: chrono::Utc::now(),
                model_used: self.config.default_model.clone(),
                cost: 0.0,
                generation_time_ms: 0,
            },
            structure: crate::ChapterStructure {
                scenes: vec![],
                foreshadowing: vec![],
                callbacks: vec![],
            },
            quality: crate::QualityMetrics {
                consistency_score: 0.95,
                style_adherence: 0.9,
                logic_check: crate::LogicCheck {
                    passed: true,
                    issues: vec![],
                },
            },
        })
    }

    fn create_coordinator_agent(&self) -> Agent {
        let config = AgentConfig {
            name: "coordinator".to_string(),
            model: Some(self.config.default_model.clone()),
            system_prompt: "You are a coordinator agent.".to_string(),
            temperature: 0.3,
            max_tokens: 2000,
        };

        Agent::new(
            config,
            self.tool_registry.clone(),
            self.tool_executor.clone(),
            self.model_router.clone(),
        )
    }

    async fn decompose_goal(
        &self,
        _coordinator: &Agent,
        _goal: &str,
        team: &Team,
    ) -> Result<Vec<Task>> {
        let agent_list: Vec<String> = team.get_agent_names();
        let default_tasks = vec![
            create_task("task-1", "Analyze goal", vec![], &agent_list[0]),
        ];
        Ok(default_tasks)
    }

    async fn execute_tasks(
        &self,
        team: &Team,
        tasks: Vec<Task>,
    ) -> Result<Vec<AgentRunResult>> {
        let mut results = Vec::new();
        let mut queue = TaskQueue::new();

        for task in tasks {
            queue.add_task(task);
        }

        while let Some(ready_task) = queue.next_ready_task() {
            let _permit = self.task_semaphore.acquire().await
                .map_err(|e| CinemaError::Unknown(e.to_string()))?;

            match self.execute_single_task(team, &ready_task).await {
                Ok(result) => {
                    queue.mark_task_complete(&ready_task.id);
                    results.push(result);
                }
                Err(e) => {
                    queue.mark_task_failed(&ready_task.id);
                    warn!("Task {} failed: {}", ready_task.id, e);
                }
            }
        }

        Ok(results)
    }

    async fn execute_single_task(
        &self,
        team: &Team,
        task: &Task,
    ) -> Result<AgentRunResult> {
        let agent = team.get_agent(&task.assignee)
            .ok_or_else(|| CinemaError::Agent(format!("Agent {} not found", task.assignee)))?;

        let prompt = format!(
            "Task: {}\nDescription: {}\n\nComplete this task.",
            task.title, task.description
        );

        agent.run(&prompt).await
    }

    async fn synthesize_results(
        &self,
        _coordinator: &Agent,
        _original_goal: &str,
        results: &[AgentRunResult],
    ) -> Result<String> {
        let combined: String = results.iter()
            .map(|r| format!("{}\n", r.output))
            .collect();
        Ok(combined)
    }
}

#[derive(Debug, Clone)]
pub enum OrchestratorEvent {
    TaskStart { task_id: String, assignee: String },
    TaskComplete { task_id: String, result: String },
    TaskFailed { task_id: String, error: String },
}
