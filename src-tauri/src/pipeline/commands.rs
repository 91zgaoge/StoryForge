use tauri::{command, AppHandle, Manager, Runtime, State};

use super::{types::*, PipelineOrchestrator, PostProcessRunWithSteps};
use crate::{
    db::{DbPool, DraftRepository, PipelineReviewRepository},
    error::AppError,
    subscription::{identity, SubscriptionService},
    task_system::{
        models::{CreateTaskRequest, TaskStatus, TaskType},
        service::TaskService,
    },
};

fn check_pipeline_feature_access(app_handle: &AppHandle, feature_id: &str) -> Result<(), AppError> {
    let pool = app_handle.state::<DbPool>();
    let user_id = identity::resolve_user_id(&app_handle, pool.inner());
    let subscription = SubscriptionService::new(pool.inner().clone());
    if !subscription.has_feature_access(&user_id, feature_id)? {
        return Err(AppError::subscription_required(
            feature_id,
            format!("{} 功能需要 Pro 订阅，请升级以继续使用", feature_id),
        ));
    }
    Ok(())
}

// ==================== Pipeline Task Delegation ====================

const PIPELINE_TASK_TIMEOUT_SECONDS: u64 = 600;
const PIPELINE_POLL_INTERVAL_MS: u64 = 200;

fn create_pipeline_review_task<R: Runtime>(
    task_service: &TaskService<R>,
    operation: &str,
    story_id: &str,
    draft_id: &str,
    payload: serde_json::Value,
) -> Result<String, AppError> {
    let req = CreateTaskRequest {
        name: format!("Pipeline {}", operation),
        description: Some(format!("story: {}, draft: {}", story_id, draft_id)),
        task_type: TaskType::PipelineReview.to_string(),
        schedule_type: "once".to_string(),
        cron_pattern: None,
        payload: Some(payload.to_string()),
        enabled: Some(false),
        max_retries: Some(0),
        heartbeat_timeout_seconds: Some(PIPELINE_TASK_TIMEOUT_SECONDS as i32),
    };

    let task = task_service.create_task(req)?;
    Ok(task.id)
}

async fn wait_for_pipeline_task<R: Runtime, T: serde::de::DeserializeOwned>(
    task_service: &TaskService<R>,
    task_id: &str,
) -> Result<T, AppError> {
    let task_service = task_service.clone();
    let timeout = std::time::Duration::from_secs(PIPELINE_TASK_TIMEOUT_SECONDS);
    let poll_interval = std::time::Duration::from_millis(PIPELINE_POLL_INTERVAL_MS);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() >= timeout {
            if let Err(e) = task_service.cancel_task(task_id) {
                log::warn!(
                    "Failed to cancel timed-out pipeline task {}: {}",
                    task_id,
                    e
                );
            }
            return Err(AppError::internal(format!(
                "Pipeline task {} timed out after {} seconds",
                task_id, PIPELINE_TASK_TIMEOUT_SECONDS
            )));
        }

        match task_service.get_task(task_id)? {
            Some(task) => match task.status {
                TaskStatus::Completed => {
                    let result_json = task.result.unwrap_or_else(|| "{}".to_string());
                    return serde_json::from_str(&result_json).map_err(|e| {
                        AppError::internal(format!(
                            "Failed to parse pipeline task {} result: {}",
                            task_id, e
                        ))
                    });
                }
                TaskStatus::Failed => {
                    let err = task
                        .error_message
                        .unwrap_or_else(|| "Pipeline task failed".to_string());
                    return Err(AppError::internal(err));
                }
                TaskStatus::Cancelled => {
                    return Err(AppError::internal(
                        "Pipeline task was cancelled".to_string(),
                    ));
                }
                _ => {}
            },
            None => {
                return Err(AppError::internal(format!(
                    "Pipeline task {} not found",
                    task_id
                )))
            }
        }

        tokio::time::sleep(poll_interval).await;
    }
}

#[derive(Debug, serde::Deserialize)]
struct FinalizeTaskResult {
    post_process_run_id: String,
}

fn build_finalize_pipeline_result(
    draft_id: String,
    chapter_number: i32,
    post_process_run_id: String,
    message: String,
) -> PipelineResult {
    PipelineResult {
        draft_id: draft_id.clone(),
        chapter_number,
        refined_draft_id: None,
        review_id: None,
        finalized_draft_id: Some(draft_id),
        post_process_run_id: if post_process_run_id.is_empty() {
            None
        } else {
            Some(post_process_run_id)
        },
        success: true,
        message,
    }
}

// ==================== Commands ====================

/// 执行 AI 修稿
#[command(rename_all = "snake_case")]
pub async fn run_refine(
    story_id: String,
    draft_id: String,
    user_prompt: Option<String>,
    _pool: State<'_, DbPool>,
    app_handle: AppHandle,
    task_service: State<'_, TaskService>,
) -> Result<RefineResult, AppError> {
    check_pipeline_feature_access(&app_handle, "pipeline_refine")?;

    let payload = serde_json::json!({
        "operation": "refine",
        "story_id": &story_id,
        "draft_id": &draft_id,
        "user_prompt": user_prompt,
    });
    let task_id =
        create_pipeline_review_task(&task_service, "refine", &story_id, &draft_id, payload)?;
    task_service.trigger_task(&task_id)?;

    let result: RefineResult = wait_for_pipeline_task(&task_service, &task_id).await?;
    Ok(result)
}

/// 执行 AI 审稿
#[command(rename_all = "snake_case")]
pub async fn run_review(
    story_id: String,
    draft_id: String,
    review_focus: Option<String>,
    _pool: State<'_, DbPool>,
    app_handle: AppHandle,
    task_service: State<'_, TaskService>,
) -> Result<ReviewResult, AppError> {
    check_pipeline_feature_access(&app_handle, "pipeline_review")?;

    let payload = serde_json::json!({
        "operation": "review",
        "story_id": &story_id,
        "draft_id": &draft_id,
        "review_focus": review_focus,
    });
    let task_id =
        create_pipeline_review_task(&task_service, "review", &story_id, &draft_id, payload)?;
    task_service.trigger_task(&task_id)?;

    let result: ReviewResult = wait_for_pipeline_task(&task_service, &task_id).await?;
    Ok(result)
}

/// 执行定稿与后处理
#[command(rename_all = "snake_case")]
pub async fn run_finalize(
    story_id: String,
    draft_id: String,
    chapter_number: i32,
    chapter_title: Option<String>,
    scene_id: Option<String>,
    _pool: State<'_, DbPool>,
    app_handle: AppHandle,
    task_service: State<'_, TaskService>,
    _vector_store: State<'_, std::sync::Arc<dyn crate::ports::VectorStore>>,
) -> Result<PipelineResult, AppError> {
    check_pipeline_feature_access(&app_handle, "pipeline_finalize")?;

    let payload = serde_json::json!({
        "operation": "finalize",
        "story_id": &story_id,
        "draft_id": &draft_id,
        "chapter_number": chapter_number,
        "chapter_title": chapter_title,
        "scene_id": scene_id,
    });
    let task_id =
        create_pipeline_review_task(&task_service, "finalize", &story_id, &draft_id, payload)?;
    task_service.trigger_task(&task_id)?;

    let inner: FinalizeTaskResult = wait_for_pipeline_task(&task_service, &task_id).await?;
    Ok(build_finalize_pipeline_result(
        draft_id,
        chapter_number,
        inner.post_process_run_id,
        "定稿完成".to_string(),
    ))
}

/// 修复定稿后处理 — 当后处理失败时重跑
#[command(rename_all = "snake_case")]
pub async fn repair_finalize(
    story_id: String,
    chapter_number: i32,
    scene_id: Option<String>,
    _pool: State<'_, DbPool>,
    app_handle: AppHandle,
    task_service: State<'_, TaskService>,
    _vector_store: State<'_, std::sync::Arc<dyn crate::ports::VectorStore>>,
) -> Result<PipelineResult, AppError> {
    let orchestrator = PipelineOrchestrator::new(app_handle.state::<DbPool>().inner().clone());

    let draft = match scene_id.as_deref() {
        Some(sid) if !sid.is_empty() => orchestrator
            .get_finalized_draft_by_scene(&story_id, sid)?
            .or(orchestrator.get_finalized_draft(&story_id, chapter_number)?),
        _ => orchestrator.get_finalized_draft(&story_id, chapter_number)?,
    }
    .ok_or_else(|| AppError::internal("未找到已定稿的草稿"))?;

    let effective_scene_id = scene_id
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| draft.scene_id.clone());

    let payload = serde_json::json!({
        "operation": "finalize",
        "story_id": &story_id,
        "draft_id": &draft.id,
        "chapter_number": chapter_number,
        "chapter_title": serde_json::Value::Null,
        "scene_id": effective_scene_id,
    });
    let task_id = create_pipeline_review_task(
        &task_service,
        "repair_finalize",
        &story_id,
        &draft.id,
        payload,
    )?;
    task_service.trigger_task(&task_id)?;

    let inner: FinalizeTaskResult = wait_for_pipeline_task(&task_service, &task_id).await?;
    Ok(build_finalize_pipeline_result(
        draft.id,
        chapter_number,
        inner.post_process_run_id,
        "后处理修复完成".to_string(),
    ))
}

/// 获取后处理运行状态（含步骤详情）
#[command(rename_all = "snake_case")]
pub async fn get_post_process_status(
    run_id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<PostProcessRunWithSteps>, AppError> {
    let orchestrator = PipelineOrchestrator::new(pool.inner().clone());
    orchestrator.get_post_process_status(&run_id)
}

/// 获取管线编排器状态 — 指定章节当前活跃草稿
#[command(rename_all = "snake_case")]
pub async fn get_pipeline_active_draft(
    story_id: String,
    chapter_number: i32,
    scene_id: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<Option<crate::db::Draft>, AppError> {
    let orchestrator = PipelineOrchestrator::new(pool.inner().clone());
    if let Some(sid) = scene_id.as_deref().filter(|s| !s.is_empty()) {
        if let Some(draft) = orchestrator.get_active_draft_by_scene(&story_id, sid)? {
            return Ok(Some(draft));
        }
    }
    orchestrator.get_active_draft(&story_id, chapter_number)
}

/// 合并修稿（用户接受修稿结果）
#[command(rename_all = "snake_case")]
pub async fn merge_revision(
    revision_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, AppError> {
    let orchestrator = PipelineOrchestrator::new(pool.inner().clone());
    orchestrator.merge_revision(&revision_id)
}

/// 获取草稿的修稿历史
#[command(rename_all = "snake_case")]
pub async fn get_draft_revision_history(
    draft_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<crate::db::Revision>, AppError> {
    let orchestrator = PipelineOrchestrator::new(pool.inner().clone());
    orchestrator.get_draft_revision_history(&draft_id)
}

/// 获取草稿的审稿历史
#[command(rename_all = "snake_case")]
pub async fn get_draft_review_history(
    draft_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<crate::db::PipelineReview>, AppError> {
    let orchestrator = PipelineOrchestrator::new(pool.inner().clone());
    orchestrator.get_draft_review_history(&draft_id)
}

/// 获取故事章节的草稿列表
#[command(rename_all = "snake_case")]
pub async fn get_story_chapter_drafts(
    story_id: String,
    chapter_number: i32,
    scene_id: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<Vec<crate::db::Draft>, AppError> {
    let repo = DraftRepository::new(pool.inner().clone());
    match scene_id.as_deref() {
        Some(sid) if !sid.is_empty() => repo
            .get_by_story_and_scene(&story_id, sid)
            .map_err(AppError::from),
        _ => repo
            .get_by_story_chapter(&story_id, chapter_number)
            .map_err(AppError::from),
    }
}

/// 获取草稿的最新审稿报告
#[command(rename_all = "snake_case")]
pub async fn get_latest_pipeline_review(
    draft_id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<crate::db::PipelineReview>, AppError> {
    let repo = PipelineReviewRepository::new(pool.inner().clone());
    repo.get_latest_by_draft(&draft_id).map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finalize_task_result_parses_empty_run_id() {
        let json = r#"{"post_process_run_id":""}"#;
        let result: FinalizeTaskResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.post_process_run_id, "");
    }

    #[test]
    fn test_finalize_task_result_parses_run_id() {
        let json = r#"{"post_process_run_id":"run-123"}"#;
        let result: FinalizeTaskResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.post_process_run_id, "run-123");
    }

    #[test]
    fn test_build_finalize_pipeline_result() {
        let result = build_finalize_pipeline_result(
            "draft-1".to_string(),
            3,
            "run-123".to_string(),
            "定稿完成".to_string(),
        );
        assert_eq!(result.draft_id, "draft-1");
        assert_eq!(result.chapter_number, 3);
        assert_eq!(result.finalized_draft_id, Some("draft-1".to_string()));
        assert_eq!(result.post_process_run_id, Some("run-123".to_string()));
        assert!(result.success);
    }

    #[test]
    fn test_build_finalize_pipeline_result_ignores_empty_run_id() {
        let result = build_finalize_pipeline_result(
            "draft-1".to_string(),
            3,
            "".to_string(),
            "定稿完成".to_string(),
        );
        assert_eq!(result.post_process_run_id, None);
    }
}
