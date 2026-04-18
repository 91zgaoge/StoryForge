//! Agent Commands
//!
//! Tauri commands for agent execution
#![allow(dead_code)]
#![allow(unused_imports)]

use super::service::{AgentService, AgentTask, AgentType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use tauri::{command, AppHandle, Emitter, Manager};
use uuid::Uuid;
use crate::db::{DbPool, CreateStoryRequest, CreateChapterRequest};
use crate::db::repositories::{StoryRepository, ChapterRepository};
use crate::subscription::{SubscriptionService, SubscriptionTier};

/// 获取当前用户订阅层级（同步）
fn get_user_tier_sync(app_handle: &AppHandle) -> SubscriptionTier {
    let app_dir = match app_handle.path().app_data_dir() {
        Ok(d) => d,
        Err(_) => return SubscriptionTier::Free,
    };
    let machine_id_path = app_dir.join(".machine_id");
    let user_id = if machine_id_path.exists() {
        std::fs::read_to_string(&machine_id_path).unwrap_or_default().trim().to_string()
    } else {
        return SubscriptionTier::Free;
    };
    if user_id.is_empty() {
        return SubscriptionTier::Free;
    }
    if let Some(pool) = app_handle.try_state::<DbPool>() {
        let service = SubscriptionService::new(pool.inner().clone());
        if let Ok(status) = service.get_or_create_subscription(&user_id) {
            return status.tier.parse().unwrap_or(SubscriptionTier::Free);
        }
    }
    SubscriptionTier::Free
}

/// 检查 AI 配额并在不足时返回错误
fn check_ai_quota_sync(app_handle: &AppHandle) -> Result<(), String> {
    let pool = app_handle.state::<DbPool>();
    let service = SubscriptionService::new(pool.inner().clone());
    let user_id = get_user_id(app_handle);
    let result = service.check_ai_quota(&user_id)?;
    if !result.allowed {
        return Err(result.message.unwrap_or_else(|| "今日 AI 创作次数已用完".to_string()));
    }
    Ok(())
}

/// 消费一次 AI 配额
fn consume_ai_quota_sync(app_handle: &AppHandle) -> Result<(), String> {
    let pool = app_handle.state::<DbPool>();
    let service = SubscriptionService::new(pool.inner().clone());
    let user_id = get_user_id(app_handle);
    let result = service.consume_ai_quota(&user_id)?;
    if !result.allowed {
        return Err(result.message.unwrap_or_else(|| "今日 AI 创作次数已用完".to_string()));
    }
    Ok(())
}

/// 获取用户 ID
fn get_user_id(app_handle: &AppHandle) -> String {
    let app_dir = app_handle.path().app_data_dir().unwrap_or_default();
    let machine_id_path = app_dir.join(".machine_id");
    if machine_id_path.exists() {
        std::fs::read_to_string(&machine_id_path).unwrap_or_default().trim().to_string()
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        let _ = std::fs::create_dir_all(&app_dir);
        let _ = std::fs::write(&machine_id_path, &id);
        id
    }
}

static TASK_HANDLES: Lazy<Mutex<HashMap<String, tokio::task::AbortHandle>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 执行Agent请求
#[derive(Debug, Deserialize)]
pub struct ExecuteAgentRequest {
    pub agent_type: AgentType,
    pub story_id: String,
    pub chapter_number: Option<u32>,
    pub input: String,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

/// Agent执行响应
#[derive(Debug, Serialize)]
pub struct ExecuteAgentResponse {
    pub task_id: String,
    pub result: Option<super::AgentResult>,
    pub error: Option<String>,
}

/// 同步执行Agent
#[command]
pub async fn agent_execute(
    request: ExecuteAgentRequest,
    app_handle: AppHandle,
) -> Result<ExecuteAgentResponse, String> {
    check_ai_quota_sync(&app_handle)?;
    let task_id = Uuid::new_v4().to_string();
    
    // 构建上下文
    let context = build_agent_context(&app_handle, &request).await?;
    
    let tier = get_user_tier_sync(&app_handle);
    let task = AgentTask {
        id: task_id.clone(),
        agent_type: request.agent_type,
        context,
        input: request.input,
        parameters: request.parameters.unwrap_or_default(),
        tier: Some(tier),
    };
    
    let service = AgentService::new(app_handle.clone());
    
    match service.execute_task(task).await {
        Ok(result) => {
            // 执行成功后才扣费，避免用户为失败请求买单
            if let Err(e) = consume_ai_quota_sync(&app_handle) {
                log::warn!("[agent_execute] Quota consume failed after success: {}", e);
            }
            Ok(ExecuteAgentResponse {
                task_id,
                result: Some(result),
                error: None,
            })
        }
        Err(e) => Ok(ExecuteAgentResponse {
            task_id,
            result: None,
            error: Some(e),
        }),
    }
}

/// 开始流式Agent执行（通过事件推送进度）
#[command]
pub async fn agent_execute_stream(
    request: ExecuteAgentRequest,
    app_handle: AppHandle,
) -> Result<String, String> {
    check_ai_quota_sync(&app_handle)?;
    let task_id = Uuid::new_v4().to_string();

    // 构建上下文
    let context = build_agent_context(&app_handle, &request).await?;

    let tier = get_user_tier_sync(&app_handle);
    let task = AgentTask {
        id: task_id.clone(),
        agent_type: request.agent_type.clone(),
        context,
        input: request.input.clone(),
        parameters: request.parameters.unwrap_or_default(),
        tier: Some(tier),
    };

    // 在后台执行
    let service = AgentService::new(app_handle.clone());
    let task_id_clone = task_id.clone();
    let app_handle_for_consume = app_handle.clone();

    let handle = tokio::spawn(async move {
        match service.execute_task(task).await {
            Ok(result) => {
                // 执行成功后才扣费
                if let Err(e) = consume_ai_quota_sync(&app_handle_for_consume) {
                    log::warn!("[agent_execute_stream] Quota consume failed after success: {}", e);
                }
                let _ = app_handle.emit(&format!("agent-complete-{}", task_id_clone), result);
            }
            Err(e) => {
                let _ = app_handle.emit(&format!("agent-error-{}", task_id_clone), e);
            }
        }
        // 完成后清理句柄
        let _ = TASK_HANDLES.lock().unwrap().remove(&task_id_clone);
    });

    TASK_HANDLES.lock().unwrap().insert(task_id.clone(), handle.abort_handle());

    Ok(task_id)
}

/// 取消Agent任务
#[command]
pub async fn agent_cancel_task(task_id: String) -> Result<(), String> {
    let mut handles = TASK_HANDLES.lock().unwrap();
    if let Some(handle) = handles.remove(&task_id) {
        handle.abort();
        log::info!("[Agent] Task {} aborted", task_id);
    } else {
        log::info!("[Agent] No active task found for {} to cancel", task_id);
    }
    Ok(())
}

/// 获取Agent执行状态
#[command]
pub fn agent_get_status(_task_id: String) -> String {
    // TODO: 实现状态跟踪
    format!("running")
}

/// 正文助手(WriterAgent)专用请求
#[derive(Debug, Deserialize)]
pub struct WriterAgentRequest {
    pub story_id: String,
    pub chapter_number: Option<u32>,
    pub current_content: String,
    pub selected_text: Option<String>,
    pub instruction: String,
}

/// 正文助手执行响应
#[derive(Debug, Serialize)]
pub struct WriterAgentResponse {
    pub content: String,
    pub story_id: Option<String>,
    pub chapter_id: Option<String>,
}

/// 执行正文助手任务（直接操作编辑器内容）
#[command]
pub async fn writer_agent_execute(
    request: WriterAgentRequest,
    app_handle: AppHandle,
) -> Result<WriterAgentResponse, String> {
    check_ai_quota_sync(&app_handle)?;
    let mut story_id = request.story_id.clone();
    let mut chapter_number = request.chapter_number.unwrap_or(1);
    let mut created_chapter_id: Option<String> = None;

    // 如果没有 story_id，自动创建新作品和第一章
    if story_id.is_empty() {
        let pool = app_handle.state::<DbPool>();
        let story_repo = StoryRepository::new(pool.inner().clone());
        let chapter_repo = ChapterRepository::new(pool.inner().clone());

        let story = story_repo.create(CreateStoryRequest {
            title: "未命名作品".to_string(),
            description: Some(request.instruction.clone()),
            genre: Some("小说".to_string()),
        }).map_err(|e| e.to_string())?;

        let chapter = chapter_repo.create(CreateChapterRequest {
            story_id: story.id.clone(),
            chapter_number: 1,
            title: Some("第一章".to_string()),
            outline: None,
            content: None,
        }).map_err(|e| e.to_string())?;

        story_id = story.id;
        chapter_number = 1;
        created_chapter_id = Some(chapter.id.clone());

        // 通知幕前切换到新章节
        let event = crate::window::FrontstageEvent::ChapterSwitch {
            story_id: story_id.clone(),
            chapter_id: chapter.id,
            title: "第一章".to_string(),
        };
        let _ = crate::window::WindowManager::send_to_frontstage(&app_handle, event);
    }

    let mut context = build_agent_context(
        &app_handle,
        &ExecuteAgentRequest {
            agent_type: AgentType::Writer,
            story_id: story_id.clone(),
            chapter_number: Some(chapter_number),
            input: request.instruction.clone(),
            parameters: None,
        },
    ).await?;

    context.current_content = Some(request.current_content);
    context.selected_text = request.selected_text;

    let tier = get_user_tier_sync(&app_handle);
    let task = AgentTask {
        id: Uuid::new_v4().to_string(),
        agent_type: AgentType::Writer,
        context,
        input: request.instruction,
        parameters: std::collections::HashMap::new(),
        tier: Some(tier),
    };

    let service = AgentService::new(app_handle.clone());

    match service.execute_task(task).await {
        Ok(result) => {
            // 执行成功后才扣费
            if let Err(e) = consume_ai_quota_sync(&app_handle) {
                log::warn!("[writer_agent_execute] Quota consume failed after success: {}", e);
            }
            // 如果创建了新区间，把生成的内容保存到数据库
            if let Some(ref chapter_id) = created_chapter_id {
                let pool = app_handle.state::<DbPool>();
                let chapter_repo = ChapterRepository::new(pool.inner().clone());
                let _ = chapter_repo.update(
                    chapter_id,
                    Some("第一章".to_string()),
                    None,
                    Some(result.content.clone()),
                );

                // 同时推送内容更新事件到幕前
                let event = crate::window::FrontstageEvent::ContentUpdate {
                    text: result.content.clone(),
                    chapter_id: chapter_id.clone(),
                };
                let _ = crate::window::WindowManager::send_to_frontstage(&app_handle, event);
            }

            Ok(WriterAgentResponse {
                content: result.content,
                story_id: Some(story_id),
                chapter_id: created_chapter_id,
            })
        },
        Err(e) => Err(e),
    }
}

/// 构建Agent上下文
/// 
/// 使用 StoryContextBuilder 从数据库读取真实故事数据，
/// 为Agent提供完整的创作上下文（包含世界观规则、场景结构等）。
pub(crate) async fn build_agent_context(
    app_handle: &AppHandle,
    request: &ExecuteAgentRequest,
) -> Result<super::AgentContext, String> {
    use crate::db::DbPool;
    use crate::creative_engine::StoryContextBuilder;
    use tauri::Manager;

    let pool = app_handle.state::<DbPool>();
    let story_id = request.story_id.clone();
    let chapter_number = request.chapter_number.unwrap_or(1);

    let builder = StoryContextBuilder::new(pool.inner().clone());
    let mut context = match builder.build_with_query(&story_id, Some(chapter_number as i32), None, None).await {
        Ok(ctx) => ctx,
        Err(e) => {
            log::warn!("[build_agent_context] StoryContextBuilder failed: {}, falling back to minimal", e);
            return Ok(super::AgentContext::minimal(story_id, String::new()));
        }
    };

    // 注入当前内容和选中文本（来自请求）
    context.current_content = None; // 由调用方填充
    context.selected_text = None;   // 由调用方填充

    Ok(context)
}
