//! Book Deconstruction Service
//!
//! 业务逻辑层：整合解析器、分块器、分析器，对外提供高层 API。

use super::analyzer::BookAnalyzer;
use super::chunker::create_chunks;
use super::models::*;
use super::parser::parse_book;
use super::repository::*;
use crate::db::DbPool;
use crate::db::{CreateCharacterRequest, CreateStoryRequest, StoryRepository};
use crate::db::repositories_v3::{SceneRepository, WorldBuildingRepository};
use crate::llm::LlmService;
use chrono::Local;
use sha2::{Digest, Sha256};
use std::path::Path;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB

pub struct BookDeconstructionService {
    pool: DbPool,
    llm_service: LlmService,
    app_handle: AppHandle,
}

impl BookDeconstructionService {
    pub fn new(pool: DbPool, llm_service: LlmService, app_handle: AppHandle) -> Self {
        Self {
            pool,
            llm_service,
            app_handle,
        }
    }

    // ==================== 上传并分析 ====================

    pub async fn upload_and_analyze(&self, file_path: &Path) -> Result<String, ParseError> {
        // 1. 校验文件
        self.validate_file(file_path)?;

        // 2. 计算文件哈希
        let file_hash = self.compute_file_hash(file_path).await?;

        // 3. 检查重复
        let book_repo = ReferenceBookRepository::new(self.pool.clone());
        if let Ok(Some(existing)) = book_repo.get_by_hash(&file_hash) {
            log::info!("[BookDeconstruction] File already exists: {}", existing.id);
            return Ok(existing.id);
        }

        // 4. 生成 book_id
        let book_id = Uuid::new_v4().to_string();

        // 5. 复制到应用数据目录
        let app_dir = self
            .app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        let books_dir = app_dir.join("books");
        std::fs::create_dir_all(&books_dir).map_err(|e| {
            ParseError::IoError(format!("Failed to create books directory: {}", e))
        })?;

        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt")
            .to_lowercase();
        let dest_path = books_dir.join(format!("{}.{}", book_id, ext));

        tokio::fs::copy(file_path, &dest_path)
            .await
            .map_err(|e| ParseError::IoError(format!("Failed to copy file: {}", e)))?;

        // 6. 解析文件
        self.emit_progress(&book_id, "extracting", 0, "正在解析文件...").await;
        let parsed = parse_book(file_path)?;

        // 7. 创建数据库记录
        let now = Local::now();
        let book = ReferenceBook {
            id: book_id.clone(),
            title: parsed.title.clone().unwrap_or_else(|| "未命名".to_string()),
            author: parsed.author.clone(),
            genre: None,
            word_count: Some(parsed.word_count as i64),
            file_format: Some(ext),
            file_hash: Some(file_hash),
            file_path: Some(dest_path.to_string_lossy().to_string()),
            world_setting: None,
            plot_summary: None,
            story_arc: None,
            analysis_status: AnalysisStatus::Pending,
            analysis_progress: 0,
            analysis_error: None,
            created_at: now,
            updated_at: now,
        };

        book_repo
            .create(&book)
            .map_err(|e| ParseError::StorageError(format!("Failed to create book record: {}", e)))?;

        // 8. 启动后台异步分析
        let pool = self.pool.clone();
        let llm_service = self.llm_service.clone();
        let app_handle = self.app_handle.clone();
        let book_id_clone = book_id.clone();
        let chunks = create_chunks(&parsed);
        let word_count = parsed.word_count;

        tauri::async_runtime::spawn(async move {
            let service = BookDeconstructionService::new(pool.clone(), llm_service.clone(), app_handle.clone());
            if let Err(e) = service.run_analysis(&book_id_clone, &chunks, word_count).await {
                log::error!("[BookDeconstruction] Analysis failed for {}: {}", book_id_clone, e);
                let repo = ReferenceBookRepository::new(pool.clone());
                let _ = repo.update_error(&book_id_clone, &e.to_string());
                let _ = service.emit_progress(&book_id_clone, "failed", 0, &format!("分析失败: {}", e)).await;
            }
        });

        Ok(book_id)
    }

    /// 执行分析（后台任务）
    async fn run_analysis(
        &self,
        book_id: &str,
        chunks: &[TextChunk],
        word_count: usize,
    ) -> Result<(), AnalysisError> {
        log::info!("[BookDeconstruction] Running analysis for {}", book_id);

        // 更新状态为分析中
        let repo = ReferenceBookRepository::new(self.pool.clone());
        repo.update_status(book_id, AnalysisStatus::Analyzing, 0)
            .map_err(|e| AnalysisError::StorageError(e.to_string()))?;

        // 执行 LLM 分析
        let analyzer = BookAnalyzer::new(
            self.llm_service.clone(),
            self.app_handle.clone(),
            self.pool.clone(),
        );

        let result = analyzer.analyze(book_id, chunks, word_count).await?;

        // 保存分析结果到数据库
        repo.update_analysis_result(
            book_id,
            result.book.genre.as_deref(),
            result.book.world_setting.as_deref(),
            result.book.plot_summary.as_deref(),
            result.book.story_arc.as_deref(),
        )
        .map_err(|e| AnalysisError::StorageError(e.to_string()))?;

        repo.update_status(book_id, AnalysisStatus::Completed, 100)
            .map_err(|e| AnalysisError::StorageError(e.to_string()))?;

        // 保存人物
        let char_repo = ReferenceCharacterRepository::new(self.pool.clone());
        char_repo.create_batch(&result.characters)
            .map_err(|e| AnalysisError::StorageError(e.to_string()))?;

        // 保存场景
        let scene_repo = ReferenceSceneRepository::new(self.pool.clone());
        scene_repo.create_batch(&result.scenes)
            .map_err(|e| AnalysisError::StorageError(e.to_string()))?;

        // 向量化存储
        self.store_embeddings(book_id, &result).await?;

        self.emit_progress(book_id, "completed", 100, "分析完成").await;
        log::info!("[BookDeconstruction] Analysis completed for {}", book_id);

        Ok(())
    }

    // ==================== 查询操作 ====================

    pub fn get_status(&self, book_id: &str) -> Result<AnalysisStatusResponse, String> {
        let repo = ReferenceBookRepository::new(self.pool.clone());
        let book = repo
            .get_by_id(book_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Book not found".to_string())?;

        Ok(AnalysisStatusResponse {
            book_id: book_id.to_string(),
            status: book.analysis_status.to_string(),
            progress: book.analysis_progress,
            current_step: None,
            error: book.analysis_error,
        })
    }

    pub fn get_analysis(&self, book_id: &str) -> Result<BookAnalysisResult, String> {
        let book_repo = ReferenceBookRepository::new(self.pool.clone());
        let char_repo = ReferenceCharacterRepository::new(self.pool.clone());
        let scene_repo = ReferenceSceneRepository::new(self.pool.clone());

        let book = book_repo
            .get_by_id(book_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Book not found".to_string())?;

        let characters = char_repo.get_by_book(book_id).map_err(|e| e.to_string())?;
        let scenes = scene_repo.get_by_book(book_id).map_err(|e| e.to_string())?;

        Ok(BookAnalysisResult {
            book,
            characters,
            scenes,
        })
    }

    pub fn list_books(&self) -> Result<Vec<ReferenceBookSummary>, String> {
        let repo = ReferenceBookRepository::new(self.pool.clone());
        repo.list_all().map_err(|e| e.to_string())
    }

    // ==================== 删除 ====================

    pub fn delete_book(&self, book_id: &str) -> Result<(), String> {
        // 删除数据库记录
        let book_repo = ReferenceBookRepository::new(self.pool.clone());
        let char_repo = ReferenceCharacterRepository::new(self.pool.clone());
        let scene_repo = ReferenceSceneRepository::new(self.pool.clone());

        char_repo.delete_by_book(book_id).map_err(|e| e.to_string())?;
        scene_repo.delete_by_book(book_id).map_err(|e| e.to_string())?;
        book_repo.delete(book_id).map_err(|e| e.to_string())?;

        // 删除文件
        let app_dir = self
            .app_handle
            .path()
            .app_data_dir()
            .unwrap_or_default();
        let books_dir = app_dir.join("books");
        for ext in &["txt", "pdf", "epub"] {
            let file_path = books_dir.join(format!("{}.{}", book_id, ext));
            if file_path.exists() {
                let _ = std::fs::remove_file(&file_path);
            }
        }

        Ok(())
    }

    // ==================== 一键转故事 ====================

    pub async fn convert_to_story(&self, book_id: &str) -> Result<String, String> {
        let analysis = self.get_analysis(book_id)?;
        let pool = self.pool.clone();

        // 1. 创建故事
        let story_repo = StoryRepository::new(pool.clone());
        let story_id = Uuid::new_v4().to_string();
        story_repo
            .create(CreateStoryRequest {
                title: analysis.book.title.clone(),
                description: analysis.book.plot_summary.clone(),
                genre: analysis.book.genre.clone(),
            })
            .map_err(|e| e.to_string())?;

        // 2. 创建世界观
        if let Some(ref world_setting) = analysis.book.world_setting {
            let wb_repo = WorldBuildingRepository::new(pool.clone());
            wb_repo
                .create(&story_id, world_setting)
                .map_err(|e| e.to_string())?;
        }

        // 3. 创建角色
        for (_i, character) in analysis.characters.iter().enumerate() {
            let char_repo = crate::db::CharacterRepository::new(pool.clone());
            char_repo
                .create(CreateCharacterRequest {
                    story_id: story_id.clone(),
                    name: character.name.clone(),
                    background: character.appearance.clone(),
                })
                .map_err(|e| e.to_string())?;
        }

        // 4. 创建场景（content 为空，仅保留 outline）
        for scene in &analysis.scenes {
            let scene_repo = SceneRepository::new(pool.clone());
            scene_repo
                .create(
                    &story_id,
                    scene.sequence_number,
                    scene.title.as_deref(),
                )
                .map_err(|e| e.to_string())?;
        }

        Ok(story_id)
    }

    // ==================== 内部辅助 ====================

    fn validate_file(&self, file_path: &Path) -> Result<(), ParseError> {
        // 检查文件大小
        let metadata = std::fs::metadata(file_path).map_err(|e| {
            ParseError::IoError(format!("Failed to read file metadata: {}", e))
        })?;

        if metadata.len() > MAX_FILE_SIZE {
            return Err(ParseError::FileTooLarge(format!(
                "File size {} exceeds maximum {}",
                metadata.len(),
                MAX_FILE_SIZE
            )));
        }

        // 检查扩展名
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "txt" | "pdf" | "epub" => Ok(()),
            _ => Err(ParseError::InvalidFormat(format!(
                "Unsupported file format: {}",
                ext
            ))),
        }
    }

    async fn compute_file_hash(&self, file_path: &Path) -> Result<String, ParseError> {
        let bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| ParseError::IoError(format!("Failed to read file: {}", e)))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    async fn store_embeddings(
        &self,
        book_id: &str,
        _result: &BookAnalysisResult,
    ) -> Result<(), AnalysisError> {
        // 向量存储集成（简化版：记录到现有 vector store）
        // 实际实现需要调用 LanceVectorStore
        // 这里预留接口，待向量存储模块完善后接入
        log::info!("[BookDeconstruction] Embedding storage for {} (placeholder)", book_id);
        Ok(())
    }

    async fn emit_progress(&self, book_id: &str, status: &str, progress: i32, message: &str) {
        let event = BookAnalysisProgressEvent {
            book_id: book_id.to_string(),
            status: status.to_string(),
            progress,
            current_step: message.to_string(),
            message: Some(message.to_string()),
        };
        let _ = self.app_handle.emit("book-analysis-progress", event);
    }
}

// ==================== 响应类型 ====================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalysisStatusResponse {
    pub book_id: String,
    pub status: String,
    pub progress: i32,
    pub current_step: Option<String>,
    pub error: Option<String>,
}


