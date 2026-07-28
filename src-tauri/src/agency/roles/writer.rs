use super::RoleSpec;
use crate::{agency::models::AgentRole, router::TaskType};

/// 高频 Writer Agent 映射为 agency role。
///
/// 注：`agency_writer_system` 为占位 prompt_id（无 bundled 文件），运行时
/// `resolve_role_prompt` 回退到 `default_role_prompt`（见 coordinator）。本角色
/// 不在 Agency genesis/continue 主流程（仅
/// LeadWriter/Producer/EditorAuditor）， 属 Swarm/planner 子系统映射。
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
