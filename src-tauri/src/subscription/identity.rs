//! 订阅身份解析：已登录 = server 账号 UUID；未登录 = 设备 machine_id

use tauri::{AppHandle, Manager, Runtime};

use crate::db::{DbPool, UserRepository};

#[derive(Debug, Clone, PartialEq)]
pub enum Identity {
    Account { user_id: String, token: String },
    Device { machine_id: String },
}

/// 设备标识（原 subscription/commands.rs get_user_id 逻辑移此，全仓唯一出处）
pub fn machine_id<R: Runtime>(app_handle: &AppHandle<R>) -> String {
    let app_dir = app_handle.path().app_data_dir().unwrap_or_default();
    let path = app_dir.join(".machine_id");
    if path.exists() {
        std::fs::read_to_string(&path)
            .unwrap_or_default()
            .trim()
            .to_string()
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        let _ = std::fs::create_dir_all(&app_dir);
        let _ = std::fs::write(&path, &id);
        id
    }
}

pub fn resolve_identity<R: Runtime>(app_handle: &AppHandle<R>, pool: &DbPool) -> Identity {
    let repo = UserRepository::new(pool.clone());
    if let Some((user, token)) = repo.find_latest_valid_session() {
        return Identity::Account {
            user_id: user.id,
            token,
        };
    }
    Identity::Device {
        machine_id: machine_id(app_handle),
    }
}

/// 现有 6 处同步订阅检查点的 user_id 来源（读本地缓存，无网络）
pub fn resolve_user_id<R: Runtime>(app_handle: &AppHandle<R>, pool: &DbPool) -> String {
    match resolve_identity(app_handle, pool) {
        Identity::Account { user_id, .. } => user_id,
        Identity::Device { machine_id } => machine_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pool() -> DbPool {
        crate::db::connection::create_test_pool().unwrap()
    }

    #[test]
    fn no_session_resolves_to_device_identity() {
        let app = tauri::test::mock_app();
        let pool = test_pool();
        match resolve_identity(app.handle(), &pool) {
            Identity::Device { machine_id } => assert!(!machine_id.is_empty()),
            Identity::Account { .. } => panic!("expected Device identity without session"),
        }
    }

    #[test]
    fn valid_session_resolves_to_account_identity() {
        let app = tauri::test::mock_app();
        let pool = test_pool();
        let repo = UserRepository::new(pool.clone());
        let user = repo
            .upsert_server_user("srv-id-u1", None, Some("Carol".to_string()), None)
            .unwrap();
        let expires_at = chrono::Local::now() + chrono::Duration::days(7);
        repo.create_session(&user.id, "token-id-1", expires_at)
            .unwrap();

        assert_eq!(
            resolve_identity(app.handle(), &pool),
            Identity::Account {
                user_id: "srv-id-u1".to_string(),
                token: "token-id-1".to_string(),
            }
        );
        assert_eq!(resolve_user_id(app.handle(), &pool), "srv-id-u1");
    }

    #[test]
    fn expired_session_falls_back_to_device_identity() {
        let app = tauri::test::mock_app();
        let pool = test_pool();
        let repo = UserRepository::new(pool.clone());
        let user = repo
            .upsert_server_user("srv-id-u2", None, None, None)
            .unwrap();
        let expires_at = chrono::Local::now() - chrono::Duration::days(1);
        repo.create_session(&user.id, "token-expired", expires_at)
            .unwrap();

        assert!(matches!(
            resolve_identity(app.handle(), &pool),
            Identity::Device { .. }
        ));
    }
}
