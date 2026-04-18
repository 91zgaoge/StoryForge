//! Subscription Service — Freemium 付费订阅系统
//!
//! 管理用户订阅状态、AI 使用配额追踪、付费功能权限检查。

use crate::db::DbPool;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};


pub mod commands;

/// 订阅层级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionTier {
    Free,
    Pro,
    Enterprise,
}

impl std::fmt::Display for SubscriptionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscriptionTier::Free => write!(f, "free"),
            SubscriptionTier::Pro => write!(f, "pro"),
            SubscriptionTier::Enterprise => write!(f, "enterprise"),
        }
    }
}

impl std::str::FromStr for SubscriptionTier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free" => Ok(SubscriptionTier::Free),
            "pro" => Ok(SubscriptionTier::Pro),
            "enterprise" => Ok(SubscriptionTier::Enterprise),
            _ => Err(format!("Unknown subscription tier: {}", s)),
        }
    }
}

/// 订阅状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionStatus {
    pub user_id: String,
    pub tier: String,
    pub status: String,
    pub daily_used: i32,
    pub daily_limit: i32,
    pub quota_resets_at: String,
    pub expires_at: Option<String>,
}

/// AI 使用配额检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaCheckResult {
    pub allowed: bool,
    pub remaining: i32,
    pub daily_limit: i32,
    pub daily_used: i32,
    pub resets_at: String,
    pub message: Option<String>,
}

/// 订阅服务
pub struct SubscriptionService {
    pool: DbPool,
}

impl SubscriptionService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// 获取或创建默认订阅状态
    pub fn get_or_create_subscription(&self, user_id: &str) -> Result<SubscriptionStatus, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;

        // 先尝试查找现有订阅
        let existing: Option<(String, String, String, Option<String>)> = conn
            .query_row(
                "SELECT tier, status, created_at, expires_at FROM subscriptions WHERE user_id = ?1 ORDER BY created_at DESC LIMIT 1",
                params![user_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        let (tier, status, expires_at) = if let Some((tier, status, _, expires)) = existing {
            (tier, status, expires)
        } else {
            // 创建默认免费订阅
            let now = chrono::Local::now().to_rfc3339();
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO subscriptions (id, user_id, tier, status, started_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?5)",
                params![id, user_id, "free", "active", now],
            ).map_err(|e| e.to_string())?;
            ("free".to_string(), "active".to_string(), None)
        };

        // 获取或创建配额记录
        let quota = self.get_or_create_quota(user_id, &tier)?;

        Ok(SubscriptionStatus {
            user_id: user_id.to_string(),
            tier,
            status,
            daily_used: quota.0,
            daily_limit: quota.1,
            quota_resets_at: quota.2,
            expires_at,
        })
    }

    /// 获取或创建配额记录
    fn get_or_create_quota(&self, user_id: &str, tier: &str) -> Result<(i32, i32, String), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;

        let existing: Option<(i32, i32, String, String)> = conn
            .query_row(
                "SELECT daily_used, daily_limit, quota_reset_at, tier FROM ai_usage_quota WHERE user_id = ?1",
                params![user_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        let now = chrono::Local::now();
        let reset_time = now.date_naive().succ_opt().unwrap_or(now.date_naive());
        let reset_at = format!("{}T00:00:00+08:00", reset_time);

        if let Some((used, limit, old_reset, old_tier)) = existing {
            // 检查是否需要重置配额（过了重置时间）
            let should_reset = if let Ok(old) = chrono::DateTime::parse_from_rfc3339(&old_reset) {
                now > old.with_timezone(&chrono::Local)
            } else {
                false
            };

            if should_reset || old_tier != tier {
                let new_limit = if tier == "pro" { 999999 } else { 10 };
                conn.execute(
                    "UPDATE ai_usage_quota SET daily_used = 0, daily_limit = ?1, quota_reset_at = ?2, tier = ?3, updated_at = ?4 WHERE user_id = ?5",
                    params![new_limit, reset_at, tier, now.to_rfc3339(), user_id],
                ).map_err(|e| e.to_string())?;
                Ok((0, new_limit, reset_at))
            } else {
                Ok((used, limit, old_reset))
            }
        } else {
            // 创建新配额记录
            let id = uuid::Uuid::new_v4().to_string();
            let limit = if tier == "pro" { 999999 } else { 10 };
            conn.execute(
                "INSERT INTO ai_usage_quota (id, user_id, tier, daily_limit, daily_used, quota_reset_at, updated_at) VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6)",
                params![id, user_id, tier, limit, reset_at, now.to_rfc3339()],
            ).map_err(|e| e.to_string())?;
            Ok((0, limit, reset_at))
        }
    }

    /// 检查 AI 配额
    pub fn check_ai_quota(&self, user_id: &str) -> Result<QuotaCheckResult, String> {
        let status = self.get_or_create_subscription(user_id)?;

        if status.tier == "pro" || status.tier == "enterprise" {
            return Ok(QuotaCheckResult {
                allowed: true,
                remaining: 999999,
                daily_limit: status.daily_limit,
                daily_used: status.daily_used,
                resets_at: status.quota_resets_at,
                message: None,
            });
        }

        let remaining = status.daily_limit - status.daily_used;
        let allowed = remaining > 0;

        Ok(QuotaCheckResult {
            allowed,
            remaining: remaining.max(0),
            daily_limit: status.daily_limit,
            daily_used: status.daily_used,
            resets_at: status.quota_resets_at,
            message: if allowed {
                None
            } else {
                Some("今日 AI 创作次数已用完，升级专业版解锁无限次".to_string())
            },
        })
    }

    /// 消费一次 AI 配额（原子操作：查询+扣减在一个事务内完成）
    pub fn consume_ai_quota(&self, user_id: &str) -> Result<QuotaCheckResult, String> {
        // 先确保订阅和配额记录存在，并处理过期重置
        let status = self.get_or_create_subscription(user_id)?;

        if status.tier == "pro" || status.tier == "enterprise" {
            return Ok(QuotaCheckResult {
                allowed: true,
                remaining: 999999,
                daily_limit: status.daily_limit,
                daily_used: status.daily_used,
                resets_at: status.quota_resets_at,
                message: None,
            });
        }

        let mut conn = self.pool.get().map_err(|e| e.to_string())?;
        let now = chrono::Local::now().to_rfc3339();

        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // 在事务内原子查询当前配额
        let (daily_used, daily_limit, resets_at): (i32, i32, String) = tx.query_row(
            "SELECT daily_used, daily_limit, quota_reset_at FROM ai_usage_quota WHERE user_id = ?1",
            params![user_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).map_err(|e| e.to_string())?;

        if daily_used >= daily_limit {
            tx.commit().map_err(|e| e.to_string())?;
            return Ok(QuotaCheckResult {
                allowed: false,
                remaining: 0,
                daily_limit,
                daily_used,
                resets_at,
                message: Some("今日 AI 创作次数已用完，升级专业版解锁无限次".to_string()),
            });
        }

        // 原子扣减
        tx.execute(
            "UPDATE ai_usage_quota SET daily_used = daily_used + 1, total_used = total_used + 1, updated_at = ?1 WHERE user_id = ?2",
            params![now, user_id],
        ).map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;

        Ok(QuotaCheckResult {
            allowed: true,
            remaining: daily_limit - daily_used - 1,
            daily_limit,
            daily_used: daily_used + 1,
            resets_at,
            message: None,
        })
    }

    /// 记录 AI 调用日志
    pub fn log_ai_usage(
        &self,
        user_id: &str,
        story_id: Option<&str>,
        chapter_id: Option<&str>,
        agent_type: &str,
        instruction: Option<&str>,
        prompt_tokens: Option<i32>,
        completion_tokens: Option<i32>,
        model_used: Option<&str>,
        cost: Option<f64>,
        duration_ms: Option<i32>,
        tier_at_time: &str,
    ) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let id = uuid::Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO ai_usage_logs (id, user_id, story_id, chapter_id, agent_type, instruction, prompt_tokens, completion_tokens, model_used, cost, duration_ms, tier_at_time, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                id,
                user_id,
                story_id,
                chapter_id,
                agent_type,
                instruction,
                prompt_tokens,
                completion_tokens,
                model_used,
                cost,
                duration_ms,
                tier_at_time,
                chrono::Local::now().to_rfc3339(),
            ],
        ).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 升级订阅（模拟，实际应对接支付系统）
    pub fn upgrade_subscription(&self, user_id: &str, tier: &str, expires_days: Option<i32>) -> Result<SubscriptionStatus, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let now = chrono::Local::now();
        let id = uuid::Uuid::new_v4().to_string();
        let expires_at = expires_days.map(|d| (now + chrono::Duration::days(d as i64)).to_rfc3339());

        conn.execute(
            "INSERT INTO subscriptions (id, user_id, tier, status, started_at, expires_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?5, ?5)",
            params![id, user_id, tier, "active", now.to_rfc3339(), expires_at],
        ).map_err(|e| e.to_string())?;

        // 更新配额
        let new_limit = if tier == "pro" || tier == "enterprise" { 999999 } else { 10 };
        let reset_time = now.date_naive().succ_opt().unwrap_or(now.date_naive());
        let reset_at = format!("{}T00:00:00+08:00", reset_time);

        conn.execute(
            "UPDATE ai_usage_quota SET tier = ?1, daily_limit = ?2, daily_used = 0, quota_reset_at = ?3, updated_at = ?4 WHERE user_id = ?5",
            params![tier, new_limit, reset_at, now.to_rfc3339(), user_id],
        ).map_err(|e| e.to_string())?;

        self.get_or_create_subscription(user_id)
    }
}
