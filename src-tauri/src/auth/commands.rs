//! Tauri IPC Commands — 认证（server 中转型）

use tauri::{AppHandle, Manager, State};

use super::OAuthProvider;
use crate::{
    db::{DbPool, UserInfo, UserRepository},
    error::AppError,
    server_client::ServerClient,
};

#[derive(Debug, serde::Serialize)]
pub struct CurrentSession {
    pub user: UserInfo,
    pub token: String,
}

/// 获取当前认证配置（登录入口恒指向 server，始终可用）
#[tauri::command]
pub fn get_auth_config() -> Result<serde_json::Value, AppError> {
    Ok(serde_json::json!({
        "google_enabled": true,
        "github_enabled": true,
        "wechat_enabled": false,
        "qq_enabled": false,
    }))
}

/// 开始 OAuth 登录：返回 server 授权 URL 与 dstate，前端打开浏览器后轮询
/// oauth_poll_login invite 为可选邀请码（内测期新用户注册需要，见 Task 2B）
#[tauri::command]
pub fn oauth_start(
    provider: String,
    invite: Option<String>,
) -> Result<serde_json::Value, AppError> {
    let provider = provider.parse::<OAuthProvider>().map_err(AppError::from)?;
    let dstate = uuid::Uuid::new_v4().to_string();
    let client = ServerClient::new();
    Ok(serde_json::json!({
        "auth_url": client.desktop_auth_url(&provider.to_string(), &dstate, invite.as_deref()),
        "dstate": dstate,
    }))
}

/// 轮询一次桌面登录结果；成功则落本地用户+session 并返回
#[tauri::command]
pub async fn oauth_poll_login(
    dstate: String,
    app_handle: AppHandle,
) -> Result<Option<CurrentSession>, AppError> {
    let client = ServerClient::new();
    let login = match client.desktop_poll(&dstate).await? {
        Some(l) => l,
        None => return Ok(None),
    };

    let pool = app_handle.state::<DbPool>().inner().clone();
    let repo = UserRepository::new(pool);
    let user = repo
        .upsert_server_user(
            &login.user_id,
            login.email,
            login.display_name,
            login.avatar_url,
        )
        .map_err(AppError::from)?;
    let expires_at = chrono::Local::now() + chrono::Duration::days(7);
    repo.create_session(&user.id, &login.token, expires_at)
        .map_err(AppError::from)?;

    // 登录后立即同步一次订阅缓存
    crate::subscription::sync_remote_subscription(&app_handle);

    Ok(Some(CurrentSession {
        user: repo.to_user_info(&user),
        token: login.token,
    }))
}

/// 获取当前登录会话（启动恢复用）
#[tauri::command]
pub fn get_current_user(pool: State<'_, DbPool>) -> Result<Option<CurrentSession>, AppError> {
    let repo = UserRepository::new(pool.inner().clone());
    Ok(repo
        .find_latest_valid_session()
        .map(|(user, token)| CurrentSession {
            user: repo.to_user_info(&user),
            token,
        }))
}

/// 注销：删本地 session + 尽力通知 server
#[tauri::command]
pub async fn logout(token: String, pool: State<'_, DbPool>) -> Result<(), AppError> {
    let repo = UserRepository::new(pool.inner().clone());
    repo.delete_session(&token).map_err(AppError::from)?;
    let _ = ServerClient::new().logout(&token).await;
    Ok(())
}
