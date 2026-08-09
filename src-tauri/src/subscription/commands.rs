//! Subscription Tauri Commands

use tauri::{command, AppHandle, Manager};

use super::{identity, SubscriptionService, SubscriptionStatus};
use crate::{db::DbPool, error::AppError};

/// 获取当前用户订阅状态（已登录：远程优先，离线走本地缓存；未登录：设备本地）
#[command]
pub async fn get_subscription_status(
    app_handle: AppHandle,
) -> Result<SubscriptionStatus, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let service = SubscriptionService::new(pool.clone());
    match identity::resolve_identity(&app_handle, &pool) {
        identity::Identity::Account { user_id, token } => {
            let client = crate::server_client::ServerClient::new();
            match client.get_subscription(&token).await {
                Ok(remote) => {
                    let _ = super::cache_remote_status(&pool, &user_id, &remote);
                    Ok(SubscriptionStatus {
                        user_id,
                        tier: remote.tier,
                        status: remote.status,
                        expires_at: remote.expires_at,
                    })
                }
                Err(_) => service.get_or_create_subscription(&user_id), // 离线走缓存
            }
        }
        identity::Identity::Device { machine_id } => {
            service.get_or_create_subscription(&machine_id)
        }
    }
}

/// 模拟升级订阅（开发测试用；已登录走 server，失败不降级写本地）
#[command]
pub async fn dev_upgrade_subscription(
    tier: String,
    app_handle: AppHandle,
) -> Result<SubscriptionStatus, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let service = SubscriptionService::new(pool.clone());
    match identity::resolve_identity(&app_handle, &pool) {
        identity::Identity::Account { user_id, token } => {
            let client = crate::server_client::ServerClient::new();
            let remote = client.dev_upgrade(&token, &tier).await?;
            let _ = super::cache_remote_status(&pool, &user_id, &remote);
            let _ = crate::state_sync::StateSync::emit_subscription_changed(
                &app_handle,
                &user_id,
                &remote.tier,
            );
            Ok(SubscriptionStatus {
                user_id,
                tier: remote.tier,
                status: remote.status,
                expires_at: remote.expires_at,
            })
        }
        identity::Identity::Device { machine_id } => {
            let expires_days = if tier == "pro" { Some(30) } else { None };
            let result = service.upgrade_subscription(&machine_id, &tier, expires_days);
            if result.is_ok() {
                let _ = crate::state_sync::StateSync::emit_subscription_changed(
                    &app_handle,
                    &machine_id,
                    &tier,
                );
            }
            result
        }
    }
}

/// 模拟降级订阅（开发测试用）
#[command]
pub async fn dev_downgrade_subscription(
    app_handle: AppHandle,
) -> Result<SubscriptionStatus, AppError> {
    dev_upgrade_subscription("free".to_string(), app_handle).await
}
