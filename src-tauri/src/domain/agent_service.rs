//! Agent service port.
//!
//! Neutral trait exposed by `crate::agents::service::AgentService` so the
//! creative engine can orchestrate agents without creating a module cycle.

use async_trait::async_trait;

use crate::{
    domain::agent_types::{AgentResult, AgentTask},
    error::AppError,
};

#[async_trait]
pub trait AgentServicePort: Send + Sync {
    /// 执行单个 Agent 任务。
    async fn execute_task(&self, task: AgentTask) -> Result<AgentResult, AppError>;
}
