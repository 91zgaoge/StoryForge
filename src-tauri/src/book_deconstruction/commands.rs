//! Book Deconstruction Tauri Commands
//!
//! IPC 命令层，暴露给前端调用。

use std::sync::Arc;

use tauri::{command, AppHandle, Manager};

use super::{
    models::*,
    service::{AnalysisStatusResponse, BookDeconstructionService},
};
use crate::{
    db::DbPool,
    error::AppError,
    llm::LlmService,
    ports::VectorStore,
    subscription::{identity, SubscriptionService},
};

fn new_service(app_handle: &AppHandle) -> Result<BookDeconstructionService, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let vector_store = app_handle.state::<Arc<dyn VectorStore>>().inner().clone();
    let llm_service = LlmService::new(app_handle.clone());
    Ok(BookDeconstructionService::new(
        pool,
        llm_service,
        app_handle.clone(),
        vector_store,
    ))
}

/// 上传文件并开始分析
#[command(rename_all = "snake_case")]
pub async fn upload_book(file_path: String, app_handle: AppHandle) -> Result<String, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let user_id = identity::resolve_user_id(&app_handle, &pool);
    let subscription = SubscriptionService::new(pool.clone());
    if !subscription.has_feature_access(&user_id, "book_deconstruction")? {
        return Err(AppError::subscription_required(
            "book_deconstruction",
            "拆书功能需要 Pro 订阅，请升级以继续使用",
        ));
    }

    let service = new_service(&app_handle)?;

    service
        .upload_and_analyze(std::path::Path::new(&file_path))
        .await
        .map_err(AppError::from)
}

/// 获取分析状态
#[command(rename_all = "snake_case")]
pub async fn get_analysis_status(
    book_id: String,
    app_handle: AppHandle,
) -> Result<AnalysisStatusResponse, AppError> {
    let service = new_service(&app_handle)?;

    service.get_status(&book_id)
}

/// 获取完整分析结果
#[command(rename_all = "snake_case")]
pub async fn get_book_analysis(
    book_id: String,
    app_handle: AppHandle,
) -> Result<BookAnalysisResult, AppError> {
    let service = new_service(&app_handle)?;

    service.get_analysis(&book_id)
}

/// 获取已拆书籍列表
#[command(rename_all = "snake_case")]
pub async fn list_reference_books(
    app_handle: AppHandle,
) -> Result<Vec<ReferenceBookListItem>, AppError> {
    let service = new_service(&app_handle)?;

    service.list_books()
}

/// 删除参考书籍
#[command(rename_all = "snake_case")]
pub async fn delete_reference_book(book_id: String, app_handle: AppHandle) -> Result<(), AppError> {
    let service = new_service(&app_handle)?;

    service.delete_book(&book_id)
}

/// 一键转为故事项目
#[command(rename_all = "snake_case")]
pub async fn convert_book_to_story(
    book_id: String,
    app_handle: AppHandle,
) -> Result<String, AppError> {
    let service = new_service(&app_handle)?;

    service.convert_to_story(&book_id).await
}

/// 取消拆书分析
#[command(rename_all = "snake_case")]
pub async fn cancel_book_analysis(book_id: String, app_handle: AppHandle) -> Result<(), AppError> {
    let service = new_service(&app_handle)?;

    service.cancel_analysis(&book_id)
}
