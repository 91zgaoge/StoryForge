//! Agent Commands
//!
//! Tauri commands for agent execution

use super::service::{AgentEvent, AgentService, AgentStage, AgentTask, AgentType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{command, AppHandle, Emitter, Manager, State};
use uuid::Uuid;

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
    
    tokio::spawn(async move {
        match service.execute_task(task).await {
            Ok(result) => {
                let _ = app_handle.emit(&format!("agent-complete-{}", task_id_clone), result);
            }
            Err(e) => {
                let _ = app_handle.emit(&format!("agent-error-{}", task_id_clone), e);
            }
        }
    });
    
    Ok(task_id)
}

/// 取消Agent任务
#[command]
pub async fn agent_cancel_task(task_id: String) -> Result<(), String> {
    // TODO: 实现任务取消机制
    log::info!("[Agent] Cancelling task: {}", task_id);
    Ok(())
}

/// 获取Agent执行状态
#[command]
pub fn agent_get_status(task_id: String) -> String {
    // TODO: 实现状态跟踪
    format!("running")
}

/// 构建Agent上下文
/// 
/// TODO: 实现完整的数据库访问来获取故事、角色和章节信息
/// 当前使用简化版本，从请求中构建基本上下文
async fn build_agent_context(
    _app_handle: &AppHandle,
    request: &ExecuteAgentRequest,
) -> Result<super::AgentContext, String> {
    use super::{ChapterSummary, CharacterInfo};
    
    // 构建最小化上下文
    // 后续可以通过Tauri state或全局变量访问数据库
    Ok(super::AgentContext {
        story_id: request.story_id.clone(),
        story_title: "未命名作品".to_string(),
        genre: "小说".to_string(),
        tone: "中性".to_string(),
        pacing: "正常".to_string(),
        chapter_number: request.chapter_number.unwrap_or(1),
        characters: vec![],
        previous_chapters: vec![],
    })
}
