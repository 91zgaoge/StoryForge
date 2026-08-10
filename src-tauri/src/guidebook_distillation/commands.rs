//! 指导书提炼 Tauri Commands
//!
//! IPC 命令层，暴露给前端调用。

use tauri::{command, AppHandle, Manager};

use super::{
    models::*,
    repository::CustomMethodologyRepository,
    service::{GuidebookDistillationService, GuidebookStatusResponse},
};
use crate::{
    db::DbPool,
    error::AppError,
    llm::LlmService,
    subscription::{identity, SubscriptionService},
};

fn new_service(app_handle: &AppHandle) -> Result<GuidebookDistillationService, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let llm_service = LlmService::new(app_handle.clone());
    Ok(GuidebookDistillationService::new(
        pool,
        llm_service,
        app_handle.clone(),
    ))
}

/// 上传指导书并开始提炼
#[command(rename_all = "snake_case")]
pub async fn upload_guidebook(
    file_path: String,
    app_handle: AppHandle,
) -> Result<String, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let user_id = identity::resolve_user_id(&app_handle, &pool);
    let subscription = SubscriptionService::new(pool.clone());
    if !subscription.has_feature_access(&user_id, "guidebook_distillation")? {
        return Err(AppError::subscription_required(
            "guidebook_distillation",
            "指导书提炼功能需要 Pro 订阅，请升级以继续使用",
        ));
    }
    let service = new_service(&app_handle)?;
    service
        .upload_and_distill(std::path::Path::new(&file_path))
        .await
        .map_err(AppError::from)
}

/// 获取提炼状态
#[command(rename_all = "snake_case")]
pub async fn get_guidebook_distillation_status(
    guidebook_id: String,
    app_handle: AppHandle,
) -> Result<GuidebookStatusResponse, AppError> {
    new_service(&app_handle)?.get_status(&guidebook_id)
}

/// 获取提炼结果（指导书 + 自定义方法论）
#[command(rename_all = "snake_case")]
pub async fn get_guidebook_result(
    guidebook_id: String,
    app_handle: AppHandle,
) -> Result<GuidebookResult, AppError> {
    new_service(&app_handle)?.get_result(&guidebook_id)
}

/// 指导书列表
#[command(rename_all = "snake_case")]
pub async fn list_guidebooks(app_handle: AppHandle) -> Result<Vec<GuidebookListItem>, AppError> {
    new_service(&app_handle)?.list_guidebooks()
}

/// 删除指导书
#[command(rename_all = "snake_case")]
pub async fn delete_guidebook(guidebook_id: String, app_handle: AppHandle) -> Result<(), AppError> {
    new_service(&app_handle)?.delete_guidebook(&guidebook_id)
}

/// 取消提炼
#[command(rename_all = "snake_case")]
pub async fn cancel_guidebook_distillation(
    guidebook_id: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    new_service(&app_handle)?.cancel_distillation(&guidebook_id)
}

/// 重试失败/已取消的提炼（复用已存文件，无需重新上传）
#[command(rename_all = "snake_case")]
pub async fn retry_guidebook_distillation(
    guidebook_id: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    new_service(&app_handle)?
        .retry_distillation(&guidebook_id)
        .await
        .map_err(AppError::from)
}

// ==================== 方法论清单与自定义方法论管理 ====================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MethodologyInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub max_steps: i32,
    pub is_custom: bool,
    pub source_book: Option<String>,
    pub enabled: bool,
}

/// 内置方法论 id 统一由 `crate::domain::methodology::methodology_type_id`
/// 提供。

/// 全量方法论清单：无 + 5 内置 + 自定义（含禁用，前端标记）
#[command(rename_all = "snake_case")]
pub async fn list_all_methodologies(
    app_handle: AppHandle,
) -> Result<Vec<MethodologyInfo>, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let mut out = vec![MethodologyInfo {
        id: String::new(),
        name: "无（自由创作）".to_string(),
        description: "不指定特定方法论，AI 自由发挥".to_string(),
        max_steps: 1,
        is_custom: false,
        source_book: None,
        enabled: true,
    }];
    for mt in crate::creative_engine::methodology::MethodologyEngine::list_available() {
        let id = crate::domain::methodology::methodology_type_id(&mt);
        out.push(MethodologyInfo {
            id: id.to_string(),
            name: mt.name().to_string(),
            description: mt.description().to_string(),
            max_steps: crate::domain::methodology::methodology_max_steps(id),
            is_custom: false,
            source_book: None,
            enabled: true,
        });
    }
    let cm_repo = CustomMethodologyRepository::new(pool.clone());
    let guidebook_repo = super::repository::GuidebookRepository::new(pool);
    for cm in cm_repo.list_all().map_err(AppError::from)? {
        let source_book = cm
            .guidebook_id
            .as_deref()
            .and_then(|gid| guidebook_repo.get_by_id(gid).ok().flatten())
            .map(|g| g.title);
        let max_steps = cm.max_steps();
        out.push(MethodologyInfo {
            id: cm.id,
            name: cm.name,
            description: cm.description.unwrap_or_default(),
            max_steps,
            is_custom: true,
            source_book,
            enabled: cm.enabled,
        });
    }
    Ok(out)
}

/// 更新自定义方法论（None 字段不动）
#[command(rename_all = "snake_case")]
pub async fn update_custom_methodology(
    id: String,
    name: Option<String>,
    description: Option<String>,
    steps: Option<Vec<MethodologyStep>>,
    enabled: Option<bool>,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    CustomMethodologyRepository::new(pool)
        .update(
            &id,
            name.as_deref(),
            description.as_deref(),
            steps.as_deref(),
            enabled,
            None,
            None,
        )
        .map_err(AppError::from)
}

/// 删除自定义方法论：引用它的故事 methodology_id 置空
#[command(rename_all = "snake_case")]
pub async fn delete_custom_methodology(id: String, app_handle: AppHandle) -> Result<(), AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let repo = CustomMethodologyRepository::new(pool);
    repo.clear_story_references(&id).map_err(AppError::from)?;
    repo.delete(&id).map_err(AppError::from)
}
