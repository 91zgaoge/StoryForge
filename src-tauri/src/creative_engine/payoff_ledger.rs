//! Payoff Ledger - 伏笔账本系统
//!
//! 扩展 ForeshadowingTracker，提供时间窗口追踪、逾期检测、风险信号、
//! 回收时机推荐等高级功能。
//!
//! v0.31.x 重构：本模块现在是一个薄包装，所有实际逻辑委托给
//! `story_system::ForeshadowingService`。原有类型统一从 `domain::foreshadowing`
//! 复用，保证单一模型、单一真源。

// Re-export unified domain types so existing `use
// crate::creative_engine::payoff_ledger::...` imports continue to work
// unchanged.
#[allow(unused_imports)]
pub use crate::domain::foreshadowing::{
    PayoffLedgerItem, PayoffRecommendation, PayoffStatus, ScopeType, UrgencyLevel,
};
use crate::{
    db::DbPool,
    domain::foreshadowing::{ForeshadowingProvider, ForeshadowingService},
    error::AppError,
    story_system::foreshadowing_service::ForeshadowingServiceImpl,
};

/// 伏笔账本
///
/// 保持原有公开 API，内部委托给 `story_system::ForeshadowingService`。
pub struct PayoffLedger {
    service: ForeshadowingServiceImpl,
}

impl PayoffLedger {
    pub fn new(pool: DbPool) -> Self {
        Self {
            service: ForeshadowingServiceImpl::new(pool),
        }
    }

    /// 获取故事的完整伏笔账本
    pub fn get_ledger(&self, story_id: &str) -> Result<Vec<PayoffLedgerItem>, AppError> {
        self.service.get_ledger(story_id)
    }

    /// 检测逾期伏笔
    pub fn detect_overdue(
        &self,
        story_id: &str,
        current_scene_number: i32,
    ) -> Result<Vec<PayoffLedgerItem>, AppError> {
        let ledger = self.service.get_ledger(story_id)?;

        let mut overdue_items = Vec::new();
        for mut item in ledger {
            let is_active = matches!(
                item.current_status,
                PayoffStatus::Setup | PayoffStatus::Hinted | PayoffStatus::PendingPayoff
            );
            if !is_active {
                continue;
            }

            let is_overdue = if let Some(target_end) = item.target_end_scene {
                target_end < current_scene_number
            } else if let Some(first_seen) = item.first_seen_scene {
                current_scene_number - first_seen > 10
            } else {
                false
            };

            if is_overdue {
                item.current_status = PayoffStatus::Overdue;
                overdue_items.push(item);
            }
        }

        overdue_items.sort_by(|a, b| b.importance.cmp(&a.importance));
        Ok(overdue_items)
    }

    /// 推荐回收时机
    pub fn recommend_payoff_timing(
        &self,
        story_id: &str,
        current_scene_number: i32,
    ) -> Result<Vec<PayoffRecommendation>, AppError> {
        self.service
            .recommend_payoffs(story_id, current_scene_number)
    }

    /// 更新伏笔的账本字段（供未来 UI 调用）
    pub fn update_ledger_fields(
        &self,
        foreshadowing_id: &str,
        target_start_scene: Option<i32>,
        target_end_scene: Option<i32>,
        risk_signals: Option<Vec<String>>,
        scope_type: Option<ScopeType>,
        ledger_key: Option<String>,
        setup_event_id: Option<String>,
        payoff_event_id: Option<String>,
        risk_signals_score: Option<f32>,
    ) -> Result<(), AppError> {
        self.service.update_ledger_fields(
            foreshadowing_id,
            target_start_scene,
            target_end_scene,
            risk_signals,
            scope_type,
            ledger_key,
            setup_event_id,
            payoff_event_id,
            risk_signals_score,
        )
    }
}

impl crate::domain::creative_engine::PayoffLedgerPort for PayoffLedger {
    fn detect_overdue_payoffs(&self, story_id: &str) -> Result<Vec<String>, AppError> {
        let current_sequence = self.service.current_scene_number(story_id)?;
        let items = self.detect_overdue(story_id, current_sequence)?;
        Ok(items.into_iter().map(|item| item.summary).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_type_display() {
        assert_eq!(ScopeType::Story.to_string(), "story");
        assert_eq!(ScopeType::Arc.to_string(), "arc");
        assert_eq!(ScopeType::Scene.to_string(), "scene");
    }

    #[test]
    fn test_payoff_status_display() {
        assert_eq!(PayoffStatus::Setup.to_string(), "setup");
        assert_eq!(PayoffStatus::Overdue.to_string(), "overdue");
    }

    #[test]
    fn test_urgency_level_order() {
        let mut levels = vec![
            UrgencyLevel::Low,
            UrgencyLevel::Critical,
            UrgencyLevel::Medium,
            UrgencyLevel::High,
        ];
        levels.sort_by(|a, b| {
            let order = |u: &UrgencyLevel| match u {
                UrgencyLevel::Critical => 0,
                UrgencyLevel::High => 1,
                UrgencyLevel::Medium => 2,
                UrgencyLevel::Low => 3,
            };
            order(a).cmp(&order(b))
        });
        assert!(matches!(levels[0], UrgencyLevel::Critical));
        assert!(matches!(levels[3], UrgencyLevel::Low));
    }
}
