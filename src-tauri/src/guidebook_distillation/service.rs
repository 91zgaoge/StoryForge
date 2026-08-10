//! 指导书提炼 Service：上传校验 → 去重 → 解析 → 任务系统执行提炼。

use std::{path::Path, sync::Arc};

use chrono::Local;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use super::{
    distiller::GuidebookDistiller,
    models::*,
    repository::{CustomMethodologyRepository, GuidebookRepository},
};
use crate::{
    book_deconstruction::{
        chunker::create_chunks,
        models::{AnalysisError, ParseError, TextChunk},
        parser::parse_book,
    },
    db::DbPool,
    error::AppError,
    llm::LlmService,
    task_system::{models::CreateTaskRequest, service::TaskService},
};

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB

pub struct GuidebookDistillationService {
    pool: DbPool,
    llm_service: LlmService,
    app_handle: AppHandle,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuidebookStatusResponse {
    pub guidebook_id: String,
    pub status: String,
    pub progress: i32,
    pub current_step: Option<String>,
    pub error: Option<String>,
}

impl GuidebookDistillationService {
    pub fn new(pool: DbPool, llm_service: LlmService, app_handle: AppHandle) -> Self {
        Self {
            pool,
            llm_service,
            app_handle,
        }
    }

    // ==================== 上传并提炼 ====================

    pub async fn upload_and_distill(&self, file_path: &Path) -> Result<String, ParseError> {
        self.validate_file(file_path)?;
        let file_hash = self.compute_file_hash(file_path).await?;

        let repo = GuidebookRepository::new(self.pool.clone());
        if let Ok(Some(existing)) = repo.get_by_hash(&file_hash) {
            log::info!(
                "[GuidebookDistillation] File already exists: {}",
                existing.id
            );
            return Ok(existing.id);
        }

        let guidebook_id = Uuid::new_v4().to_string();
        let app_dir = self
            .app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        let dir = app_dir.join("guidebooks");
        std::fs::create_dir_all(&dir)
            .map_err(|e| ParseError::IoError(format!("Failed to create guidebooks dir: {}", e)))?;
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt")
            .to_lowercase();
        let dest_path = dir.join(format!("{}.{}", guidebook_id, ext));
        tokio::fs::copy(file_path, &dest_path)
            .await
            .map_err(|e| ParseError::IoError(format!("Failed to copy file: {}", e)))?;

        let parsed = parse_book(&dest_path, None)?;

        let now = Local::now();
        let book = Guidebook {
            id: guidebook_id.clone(),
            title: parsed.title.clone().unwrap_or_else(|| "未命名".to_string()),
            author: parsed.author.clone(),
            subject: None,
            word_count: Some(parsed.word_count as i64),
            file_format: Some(ext),
            file_hash: Some(file_hash),
            file_path: Some(dest_path.to_string_lossy().to_string()),
            methodology_id: None,
            status: DistillationStatus::Pending,
            progress: 0,
            error: None,
            task_id: None,
            created_at: now,
            updated_at: now,
        };
        repo.create(&book)
            .map_err(|e| ParseError::StorageError(format!("Failed to create record: {}", e)))?;

        let payload = serde_json::json!({
            "guidebook_id": guidebook_id,
            "file_path": dest_path.to_string_lossy().to_string(),
        })
        .to_string();
        let task_req = CreateTaskRequest {
            name: format!(
                "指导书提炼: {}",
                parsed.title.clone().unwrap_or_else(|| "未命名".to_string())
            ),
            description: Some(format!("提炼 {} 字的指导书", parsed.word_count)),
            task_type: "guidebook_distillation".to_string(),
            schedule_type: "once".to_string(),
            cron_pattern: None,
            payload: Some(payload),
            enabled: Some(true),
            max_retries: Some(3),
            // 心跳：600s 无进度则 HeartbeatMonitor 杀僵死任务；墙钟由
            // TaskService 对 guidebook_distillation 放宽到 12h（同拆书）。
            heartbeat_timeout_seconds: Some(600),
        };

        let task_service = self.app_handle.state::<TaskService>();
        match task_service.create_task(task_req) {
            Ok(task) => {
                let _ = repo.update_task_id(&guidebook_id, &task.id);
                let _ = repo.update_status(&guidebook_id, DistillationStatus::Pending, 0);
            }
            Err(e) => {
                log::error!(
                    "[GuidebookDistillation] 任务创建失败，回退直接后台提炼: {}",
                    e
                );
                let pool = self.pool.clone();
                let llm_service = self.llm_service.clone();
                let app_handle = self.app_handle.clone();
                let gid = guidebook_id.clone();
                let chunks = create_chunks(&parsed);
                tauri::async_runtime::spawn(async move {
                    let service =
                        GuidebookDistillationService::new(pool.clone(), llm_service, app_handle);
                    if let Err(e) = service.run_distillation(&gid, &chunks, None, None).await {
                        log::error!("[GuidebookDistillation] 回退提炼失败 {}: {}", gid, e);
                        let repo = GuidebookRepository::new(pool.clone());
                        let _ = repo.update_error(&gid, &e.to_string());
                    }
                });
            }
        }

        Ok(guidebook_id)
    }

    /// 执行提炼（任务系统与回退路径共用）
    pub async fn run_distillation(
        &self,
        guidebook_id: &str,
        chunks: &[TextChunk],
        heartbeat: Option<Box<dyn Fn() + Send + Sync>>,
        cancel_check: Option<Box<dyn Fn() -> bool + Send + Sync>>,
    ) -> Result<(), AnalysisError> {
        let repo = GuidebookRepository::new(self.pool.clone());
        repo.update_status(guidebook_id, DistillationStatus::Distilling, 0)
            .map_err(|e| AnalysisError::StorageError(e.to_string()))?;

        let concurrency = {
            let app_dir = self.app_handle.path().app_data_dir().unwrap_or_default();
            crate::config::AppConfig::load(&app_dir)
                .map(|c| c.book_deconstruction_concurrency)
                .unwrap_or(3)
        };

        let distiller = GuidebookDistiller::new(
            self.llm_service.clone(),
            self.app_handle.clone(),
            self.pool.clone(),
            concurrency,
        );
        let output = distiller
            .distill(guidebook_id, chunks, heartbeat, cancel_check)
            .await?;

        // 落库：自定义方法论
        let methodology_id = format!("custom_{}", Uuid::new_v4());
        let now = Local::now();
        let cm = CustomMethodology {
            id: methodology_id.clone(),
            guidebook_id: Some(guidebook_id.to_string()),
            name: output.methodology.name.clone(),
            description: output.methodology.description.clone(),
            steps: output
                .methodology
                .steps
                .iter()
                .map(|s| MethodologyStep {
                    title: s.title.clone(),
                    instruction: s.instruction.clone(),
                    checklist: s.checklist.clone(),
                })
                .collect(),
            patterns: vec![],
            cheatsheet: Cheatsheet::default(),
            enabled: true,
            created_at: now,
            updated_at: now,
        };
        CustomMethodologyRepository::new(self.pool.clone())
            .create(&cm)
            .map_err(|e| AnalysisError::StorageError(e.to_string()))?;

        repo.update_distilled(
            guidebook_id,
            output.metadata.title.as_deref(),
            output.metadata.author.as_deref(),
            output.metadata.subject.as_deref(),
            &methodology_id,
        )
        .map_err(|e| AnalysisError::StorageError(e.to_string()))?;
        repo.update_status(guidebook_id, DistillationStatus::Completed, 100)
            .map_err(|e| AnalysisError::StorageError(e.to_string()))?;
        log::info!(
            "[GuidebookDistillation] {} 提炼完成 → 方法论 {}（{}）",
            guidebook_id,
            methodology_id,
            cm.name
        );
        Ok(())
    }

    // ==================== 查询与管理 ====================

    pub fn get_status(&self, guidebook_id: &str) -> Result<GuidebookStatusResponse, AppError> {
        let repo = GuidebookRepository::new(self.pool.clone());
        let book = repo
            .get_by_id(guidebook_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("guidebook", guidebook_id))?;
        Ok(GuidebookStatusResponse {
            guidebook_id: book.id,
            status: book.status.to_string(),
            progress: book.progress,
            current_step: None,
            error: book.error,
        })
    }

    pub fn get_result(&self, guidebook_id: &str) -> Result<GuidebookResult, AppError> {
        let repo = GuidebookRepository::new(self.pool.clone());
        let book = repo
            .get_by_id(guidebook_id)
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::not_found("guidebook", guidebook_id))?;
        let methodology = book.methodology_id.as_deref().and_then(|mid| {
            CustomMethodologyRepository::new(self.pool.clone())
                .get_by_id(mid)
                .ok()
                .flatten()
        });
        Ok(GuidebookResult {
            guidebook: book,
            methodology,
        })
    }

    pub fn list_guidebooks(&self) -> Result<Vec<GuidebookListItem>, AppError> {
        GuidebookRepository::new(self.pool.clone())
            .list_all()
            .map_err(AppError::from)
    }

    pub fn delete_guidebook(&self, guidebook_id: &str) -> Result<(), AppError> {
        let repo = GuidebookRepository::new(self.pool.clone());
        if let Ok(Some(book)) = repo.get_by_id(guidebook_id) {
            if let Some(path) = book.file_path {
                let _ = std::fs::remove_file(path);
            }
        }
        repo.delete(guidebook_id).map_err(AppError::from)
    }

    /// 取消：置 Cancelled；任务系统侧取消照搬
    /// BookDeconstructionService::cancel_analysis 的做法（有 task_id 则
    /// 通过 TaskService 取消任务，executor 的 cancel_check 负责中断 LLM）。
    pub fn cancel_distillation(&self, guidebook_id: &str) -> Result<(), AppError> {
        let repo = GuidebookRepository::new(self.pool.clone());
        if let Ok(Some(book)) = repo.get_by_id(guidebook_id) {
            if let Some(task_id) = book.task_id {
                let task_service = self.app_handle.state::<TaskService>();
                if let Err(e) = task_service.cancel_task(&task_id) {
                    log::warn!(
                        "[GuidebookDistillation] Failed to cancel task {}: {}",
                        task_id,
                        e
                    );
                }
            }
        }
        repo.update_status(guidebook_id, DistillationStatus::Cancelled, 0)
            .map_err(AppError::from)
    }

    // ==================== 文件工具（与拆书同款规则） ====================

    fn validate_file(&self, file_path: &Path) -> Result<(), ParseError> {
        if !file_path.exists() {
            return Err(ParseError::IoError("文件不存在".to_string()));
        }
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !["txt", "pdf", "epub"].contains(&ext.as_str()) {
            return Err(ParseError::InvalidFormat(format!(
                "不支持的格式 .{}，仅支持 txt/pdf/epub",
                ext
            )));
        }
        let size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
        if size > MAX_FILE_SIZE {
            return Err(ParseError::FileTooLarge(format!(
                "文件大小 {}MB 超过 100MB 上限",
                size / 1024 / 1024
            )));
        }
        Ok(())
    }

    async fn compute_file_hash(&self, file_path: &Path) -> Result<String, ParseError> {
        let content = tokio::fs::read(file_path)
            .await
            .map_err(|e| ParseError::IoError(format!("读取文件失败: {}", e)))?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }
}

/// 续写注入点用：渲染自定义方法论当前步骤的约束文本。
/// 未知 id / 已禁用 / 无步骤 → None（调用方静默跳过注入）。
pub fn render_custom_methodology_extension(
    pool: &DbPool,
    methodology_id: &str,
    step: i32,
) -> Option<String> {
    let cm = CustomMethodologyRepository::new(pool.clone())
        .get_by_id(methodology_id)
        .ok()
        .flatten()?;
    if !cm.enabled || cm.steps.is_empty() {
        return None;
    }
    let idx = ((step.max(1) as usize) - 1).min(cm.steps.len() - 1);
    let s = &cm.steps[idx];
    let checklist = if s.checklist.is_empty() {
        String::new()
    } else {
        format!(
            "\n检查清单：\n{}",
            s.checklist
                .iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    Some(format!(
        "【创作方法论（{}·第{}步：{}）】\n{}{}",
        cm.name,
        idx + 1,
        s.title,
        s.instruction,
        checklist
    ))
}

#[cfg(test)]
mod extension_tests {
    use super::*;
    use crate::db::connection::create_test_pool;

    fn seed_cm(pool: &DbPool, enabled: bool) {
        CustomMethodologyRepository::new(pool.clone())
            .create(&CustomMethodology {
                id: "custom_t1".into(),
                guidebook_id: None,
                name: "冲突驱动法".into(),
                description: None,
                steps: vec![
                    MethodologyStep {
                        title: "立冲突".into(),
                        instruction: "确立核心冲突".into(),
                        checklist: vec!["冲突明确吗？".into()],
                    },
                    MethodologyStep {
                        title: "升级".into(),
                        instruction: "升级冲突".into(),
                        checklist: vec![],
                    },
                ],
                patterns: vec![],
                cheatsheet: Cheatsheet::default(),
                enabled,
                created_at: chrono::Local::now(),
                updated_at: chrono::Local::now(),
            })
            .unwrap();
    }

    #[test]
    fn render_extension_picks_step_and_formats() {
        let pool = create_test_pool().unwrap();
        seed_cm(&pool, true);
        let text = render_custom_methodology_extension(&pool, "custom_t1", 2).unwrap();
        assert!(text.contains("冲突驱动法"));
        assert!(text.contains("第2步：升级"));
        assert!(text.contains("升级冲突"));
        // 越界 step 钳到最后一步
        let clamped = render_custom_methodology_extension(&pool, "custom_t1", 99).unwrap();
        assert!(clamped.contains("第2步"));
        // 第 1 步带检查清单
        let step1 = render_custom_methodology_extension(&pool, "custom_t1", 1).unwrap();
        assert!(step1.contains("检查清单"));
        assert!(step1.contains("冲突明确吗？"));
    }

    #[test]
    fn render_extension_none_when_disabled_or_unknown() {
        let pool = create_test_pool().unwrap();
        seed_cm(&pool, false);
        assert!(render_custom_methodology_extension(&pool, "custom_t1", 1).is_none());
        assert!(render_custom_methodology_extension(&pool, "custom_unknown", 1).is_none());
    }
}
