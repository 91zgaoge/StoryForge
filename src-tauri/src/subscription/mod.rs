//! Subscription Service — Freemium 付费订阅系统
//!
//! 管理用户订阅状态、AI 使用配额追踪、付费功能权限检查。
//! V2: 按功能区分配额（仅 auto_write / auto_revise 收费，其余免费）

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

/// V2 配额详情（按功能区分）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaDetail {
    pub auto_write_used: i32,
    pub auto_write_limit: i32,
    pub auto_revise_used: i32,
    pub auto_revise_limit: i32,
    pub max_chars_per_call: i32,
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

    /// 获取或创建配额记录 (V2: 包含按功能区分的字段)
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

        let is_pro = tier == "pro" || tier == "enterprise";
        let new_limit = if is_pro { 999999 } else { 10 };

        if let Some((used, limit, old_reset, old_tier)) = existing {
            // 检查是否需要重置配额（过了重置时间）
            let should_reset = if let Ok(old) = chrono::DateTime::parse_from_rfc3339(&old_reset) {
                now > old.with_timezone(&chrono::Local)
            } else {
                false
            };

            if should_reset || old_tier != tier {
                conn.execute(
                    "UPDATE ai_usage_quota SET daily_used = 0, daily_limit = ?1, quota_reset_at = ?2, tier = ?3, updated_at = ?4, auto_write_used = 0, auto_write_limit = ?5, auto_revise_used = 0, auto_revise_limit = ?5, max_chars_per_call = ?6 WHERE user_id = ?7",
                    params![new_limit, reset_at, tier, now.to_rfc3339(), new_limit, if is_pro { 999999 } else { 1000 }, user_id],
                ).map_err(|e| e.to_string())?;
                Ok((0, new_limit, reset_at))
            } else {
                Ok((used, limit, old_reset))
            }
        } else {
            // 创建新配额记录
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO ai_usage_quota (id, user_id, tier, daily_limit, daily_used, quota_reset_at, updated_at, auto_write_used, auto_write_limit, auto_revise_used, auto_revise_limit, max_chars_per_call) VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, 0, ?4, 0, ?4, ?7)",
                params![id, user_id, tier, new_limit, reset_at, now.to_rfc3339(), if is_pro { 999999 } else { 1000 }],
            ).map_err(|e| e.to_string())?;
            Ok((0, new_limit, reset_at))
        }
    }

    /// 获取 V2 配额详情
    pub fn get_quota_detail(&self, user_id: &str) -> Result<QuotaDetail, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;

        let row: Option<(i32, i32, i32, i32, i32)> = conn
            .query_row(
                "SELECT auto_write_used, auto_write_limit, auto_revise_used, auto_revise_limit, max_chars_per_call FROM ai_usage_quota WHERE user_id = ?1",
                params![user_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        if let Some((aw_used, aw_limit, ar_used, ar_limit, max_chars)) = row {
            Ok(QuotaDetail {
                auto_write_used: aw_used,
                auto_write_limit: aw_limit,
                auto_revise_used: ar_used,
                auto_revise_limit: ar_limit,
                max_chars_per_call: max_chars,
            })
        } else {
            // 记录不存在，返回默认值
            let status = self.get_or_create_subscription(user_id)?;
            let is_pro = status.tier == "pro" || status.tier == "enterprise";
            Ok(QuotaDetail {
                auto_write_used: 0,
                auto_write_limit: if is_pro { 999999 } else { 10 },
                auto_revise_used: 0,
                auto_revise_limit: if is_pro { 999999 } else { 10 },
                max_chars_per_call: if is_pro { 999999 } else { 1000 },
            })
        }
    }

    /// 检查自动续写配额
    pub fn check_auto_write_quota(&self, user_id: &str, requested_chars: i32) -> Result<QuotaCheckResult, String> {
        let status = self.get_or_create_subscription(user_id)?;

        if status.tier == "pro" || status.tier == "enterprise" {
            return Ok(QuotaCheckResult {
                allowed: true,
                remaining: 999999,
                daily_limit: 999999,
                daily_used: 0,
                resets_at: status.quota_resets_at,
                message: None,
            });
        }

        let detail = self.get_quota_detail(user_id)?;
        let remaining = detail.auto_write_limit - detail.auto_write_used;
        let allowed = remaining > 0 && requested_chars <= detail.max_chars_per_call;

        let message = if remaining <= 0 {
            Some("今日自动续写次数已用完，升级专业版解锁无限次".to_string())
        } else if requested_chars > detail.max_chars_per_call {
            Some(format!("免费用户每次最多 {} 字，升级专业版解锁无限", detail.max_chars_per_call))
        } else {
            None
        };

        Ok(QuotaCheckResult {
            allowed,
            remaining: remaining.max(0),
            daily_limit: detail.auto_write_limit,
            daily_used: detail.auto_write_used,
            resets_at: status.quota_resets_at,
            message,
        })
    }

    /// 检查自动修改配额
    pub fn check_auto_revise_quota(&self, user_id: &str, requested_chars: i32) -> Result<QuotaCheckResult, String> {
        let status = self.get_or_create_subscription(user_id)?;

        if status.tier == "pro" || status.tier == "enterprise" {
            return Ok(QuotaCheckResult {
                allowed: true,
                remaining: 999999,
                daily_limit: 999999,
                daily_used: 0,
                resets_at: status.quota_resets_at,
                message: None,
            });
        }

        let detail = self.get_quota_detail(user_id)?;
        let remaining = detail.auto_revise_limit - detail.auto_revise_used;
        let allowed = remaining > 0 && requested_chars <= detail.max_chars_per_call;

        let message = if remaining <= 0 {
            Some("今日自动修改次数已用完，升级专业版解锁无限次".to_string())
        } else if requested_chars > detail.max_chars_per_call {
            Some(format!("免费用户每次最多 {} 字，升级专业版解锁无限", detail.max_chars_per_call))
        } else {
            None
        };

        Ok(QuotaCheckResult {
            allowed,
            remaining: remaining.max(0),
            daily_limit: detail.auto_revise_limit,
            daily_used: detail.auto_revise_used,
            resets_at: status.quota_resets_at,
            message,
        })
    }

    /// 消费一次自动续写配额（原子操作）
    pub fn consume_auto_write_quota(&self, user_id: &str, _actual_chars: i32) -> Result<QuotaCheckResult, String> {
        let status = self.get_or_create_subscription(user_id)?;

        if status.tier == "pro" || status.tier == "enterprise" {
            return Ok(QuotaCheckResult {
                allowed: true,
                remaining: 999999,
                daily_limit: 999999,
                daily_used: 0,
                resets_at: status.quota_resets_at,
                message: None,
            });
        }

        let mut conn = self.pool.get().map_err(|e| e.to_string())?;
        let now = chrono::Local::now().to_rfc3339();
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        let (aw_used, aw_limit, resets_at, _max_chars): (i32, i32, String, i32) = tx.query_row(
            "SELECT auto_write_used, auto_write_limit, quota_reset_at, max_chars_per_call FROM ai_usage_quota WHERE user_id = ?1",
            params![user_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).map_err(|e| e.to_string())?;

        if aw_used >= aw_limit {
            tx.commit().map_err(|e| e.to_string())?;
            return Ok(QuotaCheckResult {
                allowed: false,
                remaining: 0,
                daily_limit: aw_limit,
                daily_used: aw_used,
                resets_at,
                message: Some("今日自动续写次数已用完，升级专业版解锁无限次".to_string()),
            });
        }

        // 原子扣减
        tx.execute(
            "UPDATE ai_usage_quota SET auto_write_used = auto_write_used + 1, total_used = total_used + 1, updated_at = ?1 WHERE user_id = ?2",
            params![now, user_id],
        ).map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;

        Ok(QuotaCheckResult {
            allowed: true,
            remaining: aw_limit - aw_used - 1,
            daily_limit: aw_limit,
            daily_used: aw_used + 1,
            resets_at,
            message: None,
        })
    }

    /// 消费一次自动修改配额（原子操作）
    pub fn consume_auto_revise_quota(&self, user_id: &str, _actual_chars: i32) -> Result<QuotaCheckResult, String> {
        let status = self.get_or_create_subscription(user_id)?;

        if status.tier == "pro" || status.tier == "enterprise" {
            return Ok(QuotaCheckResult {
                allowed: true,
                remaining: 999999,
                daily_limit: 999999,
                daily_used: 0,
                resets_at: status.quota_resets_at,
                message: None,
            });
        }

        let mut conn = self.pool.get().map_err(|e| e.to_string())?;
        let now = chrono::Local::now().to_rfc3339();
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        let (ar_used, ar_limit, resets_at, _max_chars): (i32, i32, String, i32) = tx.query_row(
            "SELECT auto_revise_used, auto_revise_limit, quota_reset_at, max_chars_per_call FROM ai_usage_quota WHERE user_id = ?1",
            params![user_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).map_err(|e| e.to_string())?;

        if ar_used >= ar_limit {
            tx.commit().map_err(|e| e.to_string())?;
            return Ok(QuotaCheckResult {
                allowed: false,
                remaining: 0,
                daily_limit: ar_limit,
                daily_used: ar_used,
                resets_at,
                message: Some("今日自动修改次数已用完，升级专业版解锁无限次".to_string()),
            });
        }

        // 原子扣减
        tx.execute(
            "UPDATE ai_usage_quota SET auto_revise_used = auto_revise_used + 1, total_used = total_used + 1, updated_at = ?1 WHERE user_id = ?2",
            params![now, user_id],
        ).map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;

        Ok(QuotaCheckResult {
            allowed: true,
            remaining: ar_limit - ar_used - 1,
            daily_limit: ar_limit,
            daily_used: ar_used + 1,
            resets_at,
            message: None,
        })
    }

    /// 【已弃用】通用 AI 配额检查 — 现为向后兼容保留，所有功能已免费开放
    pub fn check_ai_quota(&self, user_id: &str) -> Result<QuotaCheckResult, String> {
        let status = self.get_or_create_subscription(user_id)?;
        Ok(QuotaCheckResult {
            allowed: true,
            remaining: 999999,
            daily_limit: status.daily_limit,
            daily_used: status.daily_used,
            resets_at: status.quota_resets_at,
            message: None,
        })
    }

    /// 【已弃用】通用 AI 配额消费 — 现为向后兼容保留，不实际扣减
    pub fn consume_ai_quota(&self, user_id: &str) -> Result<QuotaCheckResult, String> {
        let status = self.get_or_create_subscription(user_id)?;
        Ok(QuotaCheckResult {
            allowed: true,
            remaining: 999999,
            daily_limit: status.daily_limit,
            daily_used: status.daily_used,
            resets_at: status.quota_resets_at,
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
            "UPDATE ai_usage_quota SET tier = ?1, daily_limit = ?2, daily_used = 0, quota_reset_at = ?3, updated_at = ?4, auto_write_used = 0, auto_write_limit = ?2, auto_revise_used = 0, auto_revise_limit = ?2, max_chars_per_call = ?5 WHERE user_id = ?6",
            params![tier, new_limit, reset_at, now.to_rfc3339(), new_limit, user_id],
        ).map_err(|e| e.to_string())?;

        self.get_or_create_subscription(user_id)
    }
}
