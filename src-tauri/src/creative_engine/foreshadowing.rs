#![allow(dead_code)]
//! Foreshadowing Tracker - 伏笔追踪系统
//!
//! 追踪故事中的伏笔（setup）和回收（payoff），在写作时提醒作者回收未解伏笔。
//!
//! v0.31.x 重构：本模块现在是一个薄包装，所有实际读写逻辑委托给
//! `story_system::ForeshadowingService`，后者是伏笔 / 线索 /
//! 回报模型的单一真源。

use crate::{
    db::DbPool,
    domain::foreshadowing::{
        ForeshadowingError, ForeshadowingProvider, ForeshadowingRecord, ForeshadowingService,
    },
    error::AppError,
    story_system::foreshadowing_service::ForeshadowingServiceImpl,
};

/// 伏笔追踪器
///
/// 保持原有公开 API，内部委托给 `story_system::ForeshadowingService`。
pub struct ForeshadowingTracker {
    service: ForeshadowingServiceImpl,
}

impl ForeshadowingTracker {
    pub fn new(pool: DbPool) -> Self {
        Self {
            service: ForeshadowingServiceImpl::new(pool),
        }
    }

    /// 添加新伏笔
    pub fn add_foreshadowing(
        &self,
        story_id: &str,
        content: &str,
        setup_scene_id: Option<&str>,
        importance: i32,
    ) -> Result<String, String> {
        self.service
            .create(story_id, content, setup_scene_id, importance)
            .map_err(|e| e.to_string())
    }

    /// 标记伏笔为已回收
    pub fn mark_payoff(
        &self,
        foreshadowing_id: &str,
        payoff_scene_id: Option<&str>,
    ) -> Result<(), String> {
        self.service
            .mark_payoff(foreshadowing_id, payoff_scene_id)
            .map_err(|e| e.to_string())
    }

    /// 放弃伏笔
    pub fn abandon(&self, foreshadowing_id: &str) -> Result<(), String> {
        self.service
            .abandon(foreshadowing_id)
            .map_err(|e| e.to_string())
    }

    /// v0.30.16: 编辑伏笔内容/重要性/设置场景（保留 status/payoff/resolved_at
    /// 不变）
    pub fn update_foreshadowing(
        &self,
        foreshadowing_id: &str,
        content: &str,
        importance: i32,
        setup_scene_id: Option<&str>,
    ) -> Result<(), String> {
        self.service
            .update(foreshadowing_id, content, importance, setup_scene_id)
            .map_err(|e| e.to_string())
    }

    /// v0.30.16: 删除伏笔
    pub fn delete_foreshadowing(&self, foreshadowing_id: &str) -> Result<(), String> {
        self.service
            .delete(foreshadowing_id)
            .map_err(|e| e.to_string())
    }

    /// 获取故事中未回收的伏笔
    pub fn get_unresolved(&self, story_id: &str) -> Result<Vec<ForeshadowingRecord>, String> {
        self.service
            .get_unresolved(story_id)
            .map_err(|e| e.to_string())
    }

    /// 获取所有伏笔（用于幕后看板）
    pub fn get_all(&self, story_id: &str) -> Result<Vec<ForeshadowingRecord>, String> {
        self.service
            .list_by_story(story_id)
            .map_err(|e| e.to_string())
    }

    /// 获取写作时的轻量提示文本
    pub fn get_writing_hints(&self, story_id: &str, limit: usize) -> Result<Vec<String>, String> {
        self.service
            .get_writing_hints(story_id, limit)
            .map_err(|e| e.to_string())
    }
}

impl ForeshadowingProvider for ForeshadowingTracker {
    fn list_by_story(
        &self,
        story_id: &str,
    ) -> Result<Vec<ForeshadowingRecord>, ForeshadowingError> {
        self.service.list_by_story(story_id)
    }

    fn get_by_id(&self, id: &str) -> Result<Option<ForeshadowingRecord>, ForeshadowingError> {
        self.service.get_by_id(id)
    }

    fn get_unresolved(
        &self,
        story_id: &str,
    ) -> Result<Vec<ForeshadowingRecord>, ForeshadowingError> {
        self.service.get_unresolved(story_id)
    }

    fn get_overdue(
        &self,
        story_id: &str,
        current_scene_number: i32,
    ) -> Result<Vec<ForeshadowingRecord>, ForeshadowingError> {
        self.service.get_overdue(story_id, current_scene_number)
    }

    fn get_writing_hints(
        &self,
        story_id: &str,
        limit: usize,
    ) -> Result<Vec<String>, ForeshadowingError> {
        self.service.get_writing_hints(story_id, limit)
    }

    fn detect_payoffs(
        &self,
        story_id: &str,
    ) -> Result<Vec<crate::domain::foreshadowing::Payoff>, ForeshadowingError> {
        self.service.detect_payoffs(story_id)
    }

    fn recommend_payoffs(
        &self,
        story_id: &str,
        current_scene_number: i32,
    ) -> Result<Vec<crate::domain::foreshadowing::PayoffRecommendation>, ForeshadowingError> {
        self.service
            .recommend_payoffs(story_id, current_scene_number)
    }

    fn get_ledger(
        &self,
        story_id: &str,
    ) -> Result<Vec<crate::domain::foreshadowing::PayoffLedgerItem>, ForeshadowingError> {
        self.service.get_ledger(story_id)
    }
}

impl crate::domain::creative_engine::ForeshadowingPort for ForeshadowingTracker {
    fn get_writing_hints(&self, story_id: &str, limit: usize) -> Result<Vec<String>, AppError> {
        Ok(self.service.get_writing_hints(story_id, limit)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::foreshadowing::ForeshadowingStatus;

    #[test]
    fn test_foreshadowing_status_display() {
        assert_eq!(ForeshadowingStatus::Setup.to_string(), "setup");
        assert_eq!(ForeshadowingStatus::Payoff.to_string(), "payoff");
    }
}
