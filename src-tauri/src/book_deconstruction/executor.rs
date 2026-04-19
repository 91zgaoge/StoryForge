//! Book Deconstruction Task Executor
//!
//! 将拆书分析实现为 TaskExecutor trait，接入任务系统。

use super::analyzer::BookAnalyzer;
use super::chunker::create_chunks;
use super::models::*;
use super::parser::parse_book;
use super::repository::*;
use crate::db::DbPool;
use crate::llm::LlmService;
use crate::task_system::executor::{TaskExecutionContext, TaskExecutor};
use crate::task_system::models::*;
use tauri::AppHandle;

pub struct BookDeconstructionExecutor {
    pool: DbPool,
    llm_service: LlmService,
    app_handle: AppHandle,
}

impl BookDeconstructionExecutor {
    pub fn new(pool: DbPool, llm_service: LlmService, app_handle: AppHandle) -> Self {
        Self {
            pool,
            llm_service,
            app_handle,
        }
    }
}

#[async_trait::async_trait]
impl TaskExecutor for BookDeconstructionExecutor {
    fn can_handle(&self, task_type: &TaskType) -> bool {
        *task_type == TaskType::BookDeconstruction
    }

    async fn execute(
        &self,
        task: &Task,
    ) -> Result<TaskResult, Box<dyn std::error::Error>> {
        let ctx = TaskExecutionContext::new(
            task.id.clone(),
            self.pool.clone(),
            self.app_handle.clone(),
        );

        ctx.log("info", "开始拆书分析任务");

        // 解析 payload
        let payload: serde_json::Value = match task.payload.as_deref() {
            Some(p) => serde_json::from_str(p).unwrap_or(serde_json::json!({})),
            None => serde_json::json!({}),
        };

        let book_id = payload.get("book_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing book_id in task payload")?;
        let file_path_str = payload.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or("Missing file_path in task payload")?;
        let file_path = std::path::Path::new(file_path_str);

        ctx.update_progress("parsing", 0, "正在解析文件...");
        ctx.heartbeat();

        // 解析文件
        let parsed = match parse_book(file_path) {
            Ok(p) => p,
            Err(e) => {
                ctx.log("error", &format!("文件解析失败: {}", e));
                return Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some(format!("文件解析失败: {}", e)),
                });
            }
        };

        ctx.update_progress("chunking", 5, "正在分块处理...");
        ctx.heartbeat();

        let chunks = create_chunks(&parsed);
        let word_count = parsed.word_count;

        // 更新 book 记录中的状态为分析中
        {
            let repo = ReferenceBookRepository::new(self.pool.clone());
            let _ = repo.update_status(book_id, AnalysisStatus::Analyzing, 5);
        }

        ctx.update_progress("analyzing", 10, "开始LLM分析...");
        ctx.heartbeat();

        // 执行分析
        let analyzer = BookAnalyzer::new(
            self.llm_service.clone(),
            self.app_handle.clone(),
            self.pool.clone(),
        );

        // 心跳回调：每完成一个主要步骤调用一次
        let heartbeat_ctx = TaskExecutionContext::new(
            task.id.clone(),
            self.pool.clone(),
            self.app_handle.clone(),
        );
        let heartbeat_callback: Box<dyn Fn() + Send + Sync> = Box::new(move || {
            heartbeat_ctx.heartbeat();
        });

        let analysis_result = match analyzer.analyze(
            book_id,
            &chunks,
            word_count,
            Some(heartbeat_callback),
        ).await {
            Ok(r) => r,
            Err(e) => {
                ctx.log("error", &format!("分析失败: {}", e));
                let repo = ReferenceBookRepository::new(self.pool.clone());
                let _ = repo.update_error(book_id, &e.to_string());
                return Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some(format!("分析失败: {}", e)),
                });
            }
        };

        ctx.update_progress("saving", 90, "正在保存结果...");
        ctx.heartbeat();

        // 保存分析结果
        {
            let repo = ReferenceBookRepository::new(self.pool.clone());
            let _ = repo.update_analysis_result(
                book_id,
                analysis_result.book.genre.as_deref(),
                analysis_result.book.world_setting.as_deref(),
                analysis_result.book.plot_summary.as_deref(),
                analysis_result.book.story_arc.as_deref(),
            );
            let _ = repo.update_status(book_id, AnalysisStatus::Completed, 100);

            let char_repo = ReferenceCharacterRepository::new(self.pool.clone());
            let _ = char_repo.create_batch(&analysis_result.characters);

            let scene_repo = ReferenceSceneRepository::new(self.pool.clone());
            let _ = scene_repo.create_batch(&analysis_result.scenes);
        }

        ctx.update_progress("completed", 100, "分析完成");
        ctx.log("info", "拆书分析任务完成");

        // 构建结果 JSON
        let result_json = serde_json::json!({
            "book_id": book_id,
            "title": analysis_result.book.title,
            "author": analysis_result.book.author,
            "genre": analysis_result.book.genre,
            "word_count": word_count,
            "character_count": analysis_result.characters.len(),
            "scene_count": analysis_result.scenes.len(),
        });

        Ok(TaskResult {
            success: true,
            result_json: Some(result_json.to_string()),
            error_message: None,
        })
    }
}
