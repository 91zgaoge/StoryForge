use super::RoleSpec;
use crate::{agency::models::AgentRole, router::TaskType};

/// 高频 OutlinePlanner Agent 映射为 agency role。
pub fn spec() -> RoleSpec {
    RoleSpec {
        role: AgentRole::OutlinePlanner,
        prompt_id: "agency_outline_planner_system",
        task_type: TaskType::WorldBuilding,
        max_turns: 12,
        max_output_tokens: 4096,
        context_budget_chars: 16_000,
    }
}
