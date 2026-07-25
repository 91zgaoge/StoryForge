#![allow(dead_code)]
//! 中性伏笔类型
//!
//! 被 creative_engine / narrative 等模块共享，避免 narrative 直接依赖
//! creative_engine。

use chrono::{DateTime, TimeDelta, Utc};

/// 伏笔状态
#[derive(Debug, Clone)]
pub enum ForeshadowingStatus {
    /// 已设置，未回收
    Setup,
    /// 已回收
    Payoff,
    /// 已放弃
    Abandoned,
}

impl std::fmt::Display for ForeshadowingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForeshadowingStatus::Setup => write!(f, "setup"),
            ForeshadowingStatus::Payoff => write!(f, "payoff"),
            ForeshadowingStatus::Abandoned => write!(f, "abandoned"),
        }
    }
}

/// 伏笔记录
#[derive(Debug, Clone)]
pub struct ForeshadowingRecord {
    pub id: String,
    pub story_id: String,
    pub content: String,
    pub setup_scene_id: Option<String>,
    pub payoff_scene_id: Option<String>,
    // LitSeg: 叙事事件关联（从 narrative_threads.foreshadow 合并）
    pub setup_event_id: Option<String>,
    pub payoff_event_id: Option<String>,
    pub risk_signals_score: Option<f32>,
    pub status: ForeshadowingStatus,
    pub importance: i32, // 1-10
    pub created_at: String,
    pub resolved_at: Option<String>,
}

impl ForeshadowingRecord {
    /// 判断该伏笔是否已解决（已回收或已放弃）。
    pub fn is_resolved(&self) -> bool {
        matches!(
            self.status,
            ForeshadowingStatus::Payoff | ForeshadowingStatus::Abandoned
        )
    }

    /// 判断该伏笔是否已逾期。
    ///
    /// 仅对 `Setup` 状态的伏笔生效。由于 `ForeshadowingRecord` 不携带场景序号，
    /// 此处使用创建时间与 `now` 的时间差，按重要性分级判断：
    /// - 重要性 8-10：超过 5 天视为逾期
    /// - 重要性 5-7：超过 10 天视为逾期
    /// - 重要性 1-4：超过 15 天视为逾期
    ///
    /// 若 `created_at` 无法解析，保守返回 `false`。
    pub fn is_overdue(&self, now: DateTime<Utc>) -> bool {
        if !matches!(self.status, ForeshadowingStatus::Setup) {
            return false;
        }
        let Ok(created_at) = self.created_at.parse::<DateTime<Utc>>() else {
            return false;
        };
        if created_at > now {
            return false;
        }
        let threshold_days = match self.importance {
            8..=10 => 5,
            5..=7 => 10,
            _ => 15,
        };
        let threshold = TimeDelta::days(threshold_days);
        now - created_at > threshold
    }
}

/// 伏笔查询端口，供 narrative 等模块在不依赖 creative_engine 的情况下读取伏笔。
pub trait ForeshadowingProvider: Send + Sync {
    fn get_all(
        &self,
        story_id: &str,
    ) -> Result<Vec<ForeshadowingRecord>, Box<dyn std::error::Error + Send + Sync>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record_with(status: ForeshadowingStatus, created_at: DateTime<Utc>) -> ForeshadowingRecord {
        ForeshadowingRecord {
            id: "fs-1".to_string(),
            story_id: "story-1".to_string(),
            content: "一把神秘的钥匙".to_string(),
            setup_scene_id: None,
            payoff_scene_id: None,
            setup_event_id: None,
            payoff_event_id: None,
            risk_signals_score: None,
            status,
            importance: 5,
            created_at: created_at.to_rfc3339(),
            resolved_at: None,
        }
    }

    #[test]
    fn payoff_and_abandoned_are_resolved() {
        let now = Utc::now();
        assert!(record_with(ForeshadowingStatus::Payoff, now).is_resolved());
        assert!(record_with(ForeshadowingStatus::Abandoned, now).is_resolved());
    }

    #[test]
    fn setup_is_not_resolved() {
        let now = Utc::now();
        assert!(!record_with(ForeshadowingStatus::Setup, now).is_resolved());
    }

    #[test]
    fn setup_becomes_overdue_after_threshold() {
        let now = Utc::now();
        let just_setup = record_with(ForeshadowingStatus::Setup, now - TimeDelta::days(9));
        assert!(!just_setup.is_overdue(now));

        let overdue = record_with(ForeshadowingStatus::Setup, now - TimeDelta::days(11));
        assert!(overdue.is_overdue(now));
    }

    #[test]
    fn resolved_records_are_never_overdue() {
        let now = Utc::now();
        let old_payoff = record_with(ForeshadowingStatus::Payoff, now - TimeDelta::days(100));
        assert!(!old_payoff.is_overdue(now));

        let old_abandoned = record_with(ForeshadowingStatus::Abandoned, now - TimeDelta::days(100));
        assert!(!old_abandoned.is_overdue(now));
    }

    #[test]
    fn overdue_threshold_respects_importance() {
        let now = Utc::now();
        let mut high = record_with(ForeshadowingStatus::Setup, now - TimeDelta::days(6));
        high.importance = 9;
        assert!(high.is_overdue(now));

        let mut low = record_with(ForeshadowingStatus::Setup, now - TimeDelta::days(6));
        low.importance = 3;
        assert!(!low.is_overdue(now));

        low.created_at = (now - TimeDelta::days(16)).to_rfc3339();
        assert!(low.is_overdue(now));
    }

    #[test]
    fn future_created_at_is_not_overdue() {
        let now = Utc::now();
        let future = record_with(ForeshadowingStatus::Setup, now + TimeDelta::days(1));
        assert!(!future.is_overdue(now));
    }

    #[test]
    fn unparseable_created_at_is_not_overdue() {
        let now = Utc::now();
        let mut record = record_with(ForeshadowingStatus::Setup, now);
        record.created_at = "not-a-date".to_string();
        assert!(!record.is_overdue(now));
    }
}
