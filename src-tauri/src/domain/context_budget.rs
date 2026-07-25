//! 上下文预算类型。
//!
//! 控制上下文构建时的 token 预算分配。默认使用 8192 tokens / 80% 预算比例，
//! 与 `default_max_context_length` 保持一致。
//!
//! 本类型只包含纯计算逻辑，不涉及任何 I/O 或状态变更。

const DEFAULT_MAX_CONTEXT_LENGTH: usize = 8192;
const DEFAULT_CONTEXT_BUDGET_RATIO: f32 = 0.8;

/// 上下文预算策略
///
/// 控制上下文构建时的 token 预算分配。默认使用 8192 tokens / 80% 预算比例，
/// 与 `default_max_context_length` 保持一致。
#[derive(Debug, Clone)]
pub struct ContextBudget {
    /// 目标模型最大上下文长度（token）
    pub max_context_length: usize,
    /// 实际使用的预算比例（0.0 - 1.0）
    pub budget_ratio: f32,
    /// 用于 token 计数的模型 family
    pub model_family: String,
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self {
            max_context_length: DEFAULT_MAX_CONTEXT_LENGTH,
            budget_ratio: DEFAULT_CONTEXT_BUDGET_RATIO,
            model_family: "cl100k".to_string(),
        }
    }
}

impl ContextBudget {
    /// 总可用预算（不含系统提示预留）
    pub fn total_budget(&self) -> usize {
        (self.max_context_length as f32 * self.budget_ratio.clamp(0.1, 0.95)) as usize
    }

    /// 为系统提示/指令保留的 token 数
    pub fn system_budget(&self) -> usize {
        (self.total_budget() as f32 * 0.15) as usize
    }

    /// 为世界/角色/风格等关键设定保留的预算
    pub fn story_context_budget(&self) -> usize {
        (self.total_budget() as f32 * 0.25) as usize
    }

    /// 为近期场景/当前内容保留的预算
    pub fn scene_budget(&self) -> usize {
        (self.total_budget() as f32 * 0.40) as usize
    }

    /// 为用户输入/选中内容保留的预算
    pub fn user_input_budget(&self) -> usize {
        (self.total_budget() as f32 * 0.20) as usize
    }

    /// 尝试从总预算中分配 `needed` tokens。
    ///
    /// 这是一个无副作用的预算查询：若 `needed` 不超过总预算，返回
    /// `Some(needed)`； 否则返回 `None`。调用方仍需自行决定如何截断或降级。
    pub fn allocate(&self, needed: usize) -> Option<usize> {
        let total = self.total_budget();
        if needed <= total {
            Some(needed)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_budget_ratio_is_applied() {
        let budget = ContextBudget::default();
        assert_eq!(
            budget.total_budget(),
            (DEFAULT_MAX_CONTEXT_LENGTH as f32 * DEFAULT_CONTEXT_BUDGET_RATIO) as usize
        );
    }

    #[test]
    fn budget_ratio_is_clamped() {
        let low = ContextBudget {
            budget_ratio: 0.05,
            ..ContextBudget::default()
        };
        assert_eq!(
            low.total_budget(),
            (DEFAULT_MAX_CONTEXT_LENGTH as f32 * 0.1) as usize
        );

        let high = ContextBudget {
            budget_ratio: 1.0,
            ..ContextBudget::default()
        };
        assert_eq!(
            high.total_budget(),
            (DEFAULT_MAX_CONTEXT_LENGTH as f32 * 0.95) as usize
        );
    }

    #[test]
    fn category_budgets_sum_to_total() {
        let budget = ContextBudget::default();
        let sum = budget.system_budget()
            + budget.story_context_budget()
            + budget.scene_budget()
            + budget.user_input_budget();
        // 由于各分类都是向下取整，总和可能略小于 total_budget。
        assert!(sum <= budget.total_budget());
        // 误差应小于分类数量（4）。
        assert!(budget.total_budget() - sum < 4);
    }

    #[test]
    fn allocate_returns_needed_when_within_budget() {
        let budget = ContextBudget::default();
        let needed = budget.total_budget() / 2;
        assert_eq!(budget.allocate(needed), Some(needed));
    }

    #[test]
    fn allocate_returns_none_when_over_budget() {
        let budget = ContextBudget::default();
        assert_eq!(budget.allocate(budget.total_budget() + 1), None);
    }

    #[test]
    fn allocate_zero_succeeds() {
        let budget = ContextBudget::default();
        assert_eq!(budget.allocate(0), Some(0));
    }

    #[test]
    fn allocate_exact_total_succeeds() {
        let budget = ContextBudget::default();
        let total = budget.total_budget();
        assert_eq!(budget.allocate(total), Some(total));
    }
}
