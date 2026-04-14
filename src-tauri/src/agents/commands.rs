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
    let task_id = Uuid::new_v4().to_string();
    
    // 构建上下文
    let context = build_agent_context(&app_handle, &request).await?;
    
    let task = AgentTask {
        id: task_id.clone(),
        agent_type: request.agent_type,
        context,
        input: request.input,
        parameters: request.parameters.unwrap_or_default(),
    };
    
    let service = AgentService::new(app_handle);
    
    match service.execute_task(task).await {
        Ok(result) => Ok(ExecuteAgentResponse {
            task_id,
            result: Some(result),
            error: None,
        }),
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
    let task_id = Uuid::new_v4().to_string();

    // 构建上下文
    let context = build_agent_context(&app_handle, &request).await?;

    let task = AgentTask {
        id: task_id.clone(),
        agent_type: request.agent_type.clone(),
        context,
        input: request.input.clone(),
        parameters: request.parameters.unwrap_or_default(),
    };

    // 在后台执行
    let service = AgentService::new(app_handle.clone());
    let task_id_clone = task_id.clone();

    let handle = tokio::spawn(async move {
        match service.execute_task(task).await {
            Ok(result) => {
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

/// 构建Agent上下文
/// 
/// 从数据库中获取故事、角色、文风以及前场景信息，为Agent提供完整上下文
pub(crate) async fn build_agent_context(
    app_handle: &AppHandle,
    request: &ExecuteAgentRequest,
) -> Result<super::AgentContext, String> {
    use super::{ChapterSummary, CharacterInfo};
    use crate::db::DbPool;
    use crate::db::repositories::{StoryRepository, CharacterRepository};
    use crate::db::repositories_v3::{WritingStyleRepository, SceneRepository};
    use tauri::Manager;

    let pool = app_handle.state::<DbPool>();
    let story_id = request.story_id.clone();
    let chapter_number = request.chapter_number.unwrap_or(1);

    // 获取故事信息
    let story_repo = StoryRepository::new(pool.inner().clone());
    let story = match story_repo.get_by_id(&story_id) {
        Ok(Some(s)) => s,
        _ => {
            return Ok(super::AgentContext {
                story_id,
                story_title: "未命名作品".to_string(),
                genre: "小说".to_string(),
                tone: "中性".to_string(),
                pacing: "正常".to_string(),
                chapter_number,
                characters: vec![],
                previous_chapters: vec![],
            });
        }
    };

    // 获取角色信息
    let char_repo = CharacterRepository::new(pool.inner().clone());
    let characters = match char_repo.get_by_story(&story_id) {
        Ok(chars) => chars.into_iter().map(|c| {
            let role = if let Some(first_trait) = c.dynamic_traits.first() {
                first_trait.trait_name.clone()
            } else {
                c.background.clone().unwrap_or_else(|| "主要角色".to_string())
            };
            CharacterInfo {
                name: c.name,
                personality: c.personality.unwrap_or_else(|| "性格未定".to_string()),
                role,
            }
        }).collect(),
        Err(_) => vec![],
    };

    // 获取文风信息（用于 tone / pacing 回退）
    let style_repo = WritingStyleRepository::new(pool.inner().clone());
    let style = style_repo.get_by_story(&story_id).ok().flatten();

    let tone = story.tone.clone()
        .or_else(|| style.as_ref().and_then(|s| s.tone.clone()))
        .unwrap_or_else(|| "中性".to_string());
    let pacing = story.pacing.clone()
        .or_else(|| style.as_ref().and_then(|s| s.pacing.clone()))
        .unwrap_or_else(|| "正常".to_string());

    // 获取前场景摘要（V3 使用 scene 替代 chapter）
    let scene_repo = SceneRepository::new(pool.inner().clone());
    let previous_chapters = match scene_repo.get_by_story(&story_id) {
        Ok(scenes) => {
            let mut prev = scenes.into_iter()
                .filter(|s| s.sequence_number < chapter_number as i32)
                .collect::<Vec<_>>();
            prev.sort_by_key(|s| s.sequence_number);
            prev.into_iter().map(|s| {
                let summary = s.content.clone()
                    .or(s.dramatic_goal.clone())
                    .unwrap_or_else(|| "无内容".to_string());
                let preview = if summary.chars().count() > 200 {
                    format!("{}...", summary.chars().take(200).collect::<String>())
                } else {
                    summary
                };
                ChapterSummary {
                    title: s.title.unwrap_or_else(|| format!("场景 {}", s.sequence_number)),
                    number: s.sequence_number.max(0) as u32,
                    summary: preview,
                }
            }).collect()
        }
        Err(_) => vec![],
    };

    Ok(super::AgentContext {
        story_id,
        story_title: story.title.clone(),
        genre: story.genre.clone().unwrap_or_else(|| "小说".to_string()),
        tone,
        pacing,
        chapter_number,
        characters,
        previous_chapters,
    })
}
