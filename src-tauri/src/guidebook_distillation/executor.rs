//! 指导书提炼 Task Executor
//!
//! 不走 LitSeg pipeline：取消用共享 AtomicBool 桥接任务系统取消到
//! distiller 的 cancel_check。

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tauri::AppHandle;

use super::service::GuidebookDistillationService;
use crate::{
    book_deconstruction::{chunker::create_chunks, parser::parse_book},
    db::DbPool,
    guidebook_distillation::repository::GuidebookRepository,
    llm::LlmService,
    task_system::{
        executor::{TaskExecutionContext, TaskExecutor},
        models::*,
    },
};

pub struct GuidebookDistillationExecutor {
    pool: DbPool,
    llm_service: LlmService,
    app_handle: AppHandle,
}

impl GuidebookDistillationExecutor {
    pub fn new(pool: DbPool, llm_service: LlmService, app_handle: AppHandle) -> Self {
        Self {
            pool,
            llm_service,
            app_handle,
        }
    }
}

#[async_trait::async_trait]
impl TaskExecutor for GuidebookDistillationExecutor {
    fn can_handle(&self, task_type: &TaskType) -> bool {
        *task_type == TaskType::GuidebookDistillation
    }

    async fn execute(&self, task: &Task) -> Result<TaskResult, Box<dyn std::error::Error>> {
        log::info!("[GuidebookDistillationExecutor] Task {} started", task.id);
        let ctx =
            TaskExecutionContext::new(task.id.clone(), self.pool.clone(), self.app_handle.clone());
        ctx.log("info", "开始指导书提炼任务");

        let payload: serde_json::Value = match task.payload.as_deref() {
            Some(p) => serde_json::from_str(p).unwrap_or_else(|_| serde_json::json!({})),
            None => serde_json::json!({}),
        };
        let guidebook_id = match payload.get("guidebook_id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => {
                return Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some("Missing guidebook_id in task payload".to_string()),
                })
            }
        };
        let file_path = match payload.get("file_path").and_then(|v| v.as_str()) {
            Some(p) => p.to_string(),
            None => {
                return Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some("Missing file_path in task payload".to_string()),
                })
            }
        };

        // 取消监控：任务系统取消 → 共享标志 → distiller 的 cancel_check
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let cancel_monitor = {
            let flag = cancel_flag.clone();
            let loop_task_id = task.id.clone();
            let loop_pool = self.pool.clone();
            let loop_app = self.app_handle.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    let check_ctx = TaskExecutionContext::new(
                        loop_task_id.clone(),
                        loop_pool.clone(),
                        loop_app.clone(),
                    );
                    if check_ctx.is_cancelled() {
                        flag.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            })
        };

        ctx.update_progress("parsing", 0, "正在解析文件...");
        ctx.heartbeat();

        // 解析文件（同步操作，用 spawn_blocking 避免阻塞异步运行时）
        let path = std::path::PathBuf::from(file_path);
        let parsed = match tokio::task::spawn_blocking(move || parse_book(&path, None)).await {
            Ok(Ok(p)) => p,
            Ok(Err(e)) => {
                let _ = GuidebookRepository::new(self.pool.clone())
                    .update_error(&guidebook_id, &format!("文件解析失败: {}", e));
                return Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some(format!("文件解析失败: {}", e)),
                });
            }
            Err(e) => {
                return Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some(format!("解析任务异常: {}", e)),
                });
            }
        };

        let chunks = create_chunks(&parsed);
        let service = GuidebookDistillationService::new(
            self.pool.clone(),
            self.llm_service.clone(),
            self.app_handle.clone(),
        );
        let flag_for_check = cancel_flag.clone();
        // 心跳闭包：每个主要步骤回调一次，刷新任务系统心跳
        // （与 book_deconstruction/executor.rs 传给 pipeline 的 heartbeat 同模式）
        let hb_pool = self.pool.clone();
        let hb_app = self.app_handle.clone();
        let hb_task_id = task.id.clone();
        let heartbeat = Box::new(move || {
            let ctx =
                TaskExecutionContext::new(hb_task_id.clone(), hb_pool.clone(), hb_app.clone());
            ctx.heartbeat();
        });
        let result = service
            .run_distillation(
                &guidebook_id,
                &chunks,
                Some(heartbeat),
                Some(Box::new(move || flag_for_check.load(Ordering::Relaxed))),
            )
            .await;

        cancel_monitor.abort();
        let _ = cancel_monitor.await;

        match result {
            Ok(()) => {
                ctx.update_progress("completed", 100, "提炼完成");
                Ok(TaskResult {
                    success: true,
                    result_json: Some(
                        serde_json::json!({ "guidebook_id": guidebook_id }).to_string(),
                    ),
                    error_message: None,
                })
            }
            Err(e) => {
                let _ = GuidebookRepository::new(self.pool.clone())
                    .update_error(&guidebook_id, &e.to_string());
                Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some(e.to_string()),
                })
            }
        }
    }
}
