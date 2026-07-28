use super::RoleSpec;
use crate::{agency::models::AgentRole, router::TaskType};

/// 高频 OutlinePlanner Agent 映射为 agency role。
///
/// 注：`agency_outline_planner_system` 为占位 prompt_id（无 bundled 文件），
/// 运行时回退到 `default_role_prompt`（见 coordinator）；不在 Agency 主流程。
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
