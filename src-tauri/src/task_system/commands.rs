//! Task System Tauri IPC Commands

use super::models::*;
use super::service::TaskService;
use crate::db::DbPool;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn create_task(
    name: String,
    description: Option<String>,
    task_type: String,
    schedule_type: String,
    cron_pattern: Option<String>,
    payload: Option<String>,
    enabled: Option<bool>,
    max_retries: Option<i32>,
    heartbeat_timeout_seconds: Option<i32>,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<Task, String> {
    let service = TaskService::new(pool.inner().clone(), app_handle);
    let req = CreateTaskRequest {
        name,
        description,
        task_type,
        schedule_type,
        cron_pattern,
        payload,
        enabled,
        max_retries,
        heartbeat_timeout_seconds,
    };
    service.create_task(req)
}

#[tauri::command]
pub async fn update_task(
    id: String,
    name: Option<String>,
    description: Option<String>,
    enabled: Option<bool>,
    cron_pattern: Option<String>,
    max_retries: Option<i32>,
    heartbeat_timeout_seconds: Option<i32>,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<Task, String> {
    let service = TaskService::new(pool.inner().clone(), app_handle);
    let req = UpdateTaskRequest {
        name,
        description,
        enabled,
        cron_pattern,
        max_retries,
        heartbeat_timeout_seconds,
    };
    service.update_task(&id, req)
}

#[tauri::command]
pub async fn delete_task(
    id: String,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let service = TaskService::new(pool.inner().clone(), app_handle);
    service.delete_task(&id)
}

#[tauri::command]
pub async fn list_tasks(
    status_filter: Option<String>,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<Vec<Task>, String> {
    let service = TaskService::new(pool.inner().clone(), app_handle);
    service.list_tasks(status_filter)
}

#[tauri::command]
pub async fn get_task(
    id: String,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<Task, String> {
    let service = TaskService::new(pool.inner().clone(), app_handle);
    service.get_task(&id)
        .and_then(|opt| opt.ok_or_else(|| "Task not found".to_string()))
}

#[tauri::command]
pub async fn trigger_task(
    id: String,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let service = TaskService::new(pool.inner().clone(), app_handle);
    service.trigger_task(&id)
}

#[tauri::command]
pub async fn cancel_task(
    id: String,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let service = TaskService::new(pool.inner().clone(), app_handle);
    service.cancel_task(&id)
}

#[tauri::command]
pub async fn get_task_logs(
    task_id: String,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<Vec<TaskLog>, String> {
    let service = TaskService::new(pool.inner().clone(), app_handle);
    service.get_task_logs(&task_id)
}
