//! Task System Tauri IPC Commands

use super::models::*;
use super::service::TaskService;

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
    service: tauri::State<'_, TaskService>,
) -> Result<Task, String> {
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
    service: tauri::State<'_, TaskService>,
) -> Result<Task, String> {
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
    service: tauri::State<'_, TaskService>,
) -> Result<(), String> {
    service.delete_task(&id)
}

#[tauri::command]
pub async fn list_tasks(
    status_filter: Option<String>,
    service: tauri::State<'_, TaskService>,
) -> Result<Vec<Task>, String> {
    service.list_tasks(status_filter)
}

#[tauri::command]
pub async fn get_task(
    id: String,
    service: tauri::State<'_, TaskService>,
) -> Result<Task, String> {
    service.get_task(&id)
        .and_then(|opt| opt.ok_or_else(|| "Task not found".to_string()))
}

#[tauri::command]
pub async fn trigger_task(
    id: String,
    service: tauri::State<'_, TaskService>,
) -> Result<(), String> {
    service.trigger_task(&id)
}

#[tauri::command]
pub async fn cancel_task(
    id: String,
    service: tauri::State<'_, TaskService>,
) -> Result<(), String> {
    service.cancel_task(&id)
}

#[tauri::command]
pub async fn get_task_logs(
    task_id: String,
    service: tauri::State<'_, TaskService>,
) -> Result<Vec<TaskLog>, String> {
    service.get_task_logs(&task_id)
}
