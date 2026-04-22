//! Audit System IPC Commands

use super::{AuditService, AuditReport};
use crate::db::DbPool;
use tauri::{command, AppHandle, State};

/// 审计场景
#[command]
pub async fn audit_scene(
    scene_id: String,
    audit_type: String,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<AuditReport, String> {
    let service = AuditService::new(pool.inner().clone());
    service.audit_scene(&scene_id, &audit_type, Some(&app_handle)).await
}
