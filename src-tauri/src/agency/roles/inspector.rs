use super::RoleSpec;
use crate::{agency::models::AgentRole, router::TaskType};

/// 高频 Inspector Agent 映射为 agency role（只读审查）。
///
/// 注：`agency_inspector_system` 为占位 prompt_id（无 bundled 文件），运行时
/// 回退到 `default_role_prompt`（见 coordinator）；不在 Agency 主流程。
pub fn spec() -> RoleSpec {
    RoleSpec {
        role: AgentRole::Inspector,
        prompt_id: "agency_inspector_system",
        task_type: TaskType::Proofreading,
        max_turns: 6,
        max_output_tokens: 2048,
        context_budget_chars: 10_000,
    }
}
