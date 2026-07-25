use super::RoleSpec;
use crate::{agency::models::AgentRole, router::TaskType};

/// 高频 Writer Agent 映射为 agency role。
pub fn spec() -> RoleSpec {
    RoleSpec {
        role: AgentRole::Writer,
        prompt_id: "agency_writer_system",
        task_type: TaskType::CreativeWriting,
        max_turns: 10,
        max_output_tokens: 8192,
        context_budget_chars: 24_000,
    }
}
