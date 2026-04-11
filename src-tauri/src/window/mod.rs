use tauri::{AppHandle, Manager, WebviewWindow, Emitter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub frontstage_visible: bool,
    pub backstage_visible: bool,
}

/// 窗口管理器 - 管理幕前和幕后窗口
pub struct WindowManager;

impl WindowManager {
    /// 获取幕前窗口
    pub fn get_frontstage(app: &AppHandle) -> Option<WebviewWindow> {
        app.get_webview_window("frontstage")
    }

    /// 获取幕后窗口
    pub fn get_backstage(app: &AppHandle) -> Option<WebviewWindow> {
        app.get_webview_window("main")
    }

    /// 显示幕前窗口
    pub fn show_frontstage(app: &AppHandle) -> Result<(), String> {
        if let Some(window) = Self::get_frontstage(app) {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Frontstage window not found".to_string())
        }
    }

    /// 隐藏幕前窗口
    pub fn hide_frontstage(app: &AppHandle) -> Result<(), String> {
        if let Some(window) = Self::get_frontstage(app) {
            window.hide().map_err(|e| e.to_string())
        } else {
            Err("Frontstage window not found".to_string())
        }
    }

    /// 切换幕前窗口显示状态
    pub fn toggle_frontstage(app: &AppHandle) -> Result<bool, String> {
        if let Some(window) = Self::get_frontstage(app) {
            let is_visible = window.is_visible().map_err(|e| e.to_string())?;
            if is_visible {
                window.hide().map_err(|e| e.to_string())?;
                Ok(false)
            } else {
                window.show().map_err(|e| e.to_string())?;
                window.set_focus().map_err(|e| e.to_string())?;
                Ok(true)
            }
        } else {
            Err("Frontstage window not found".to_string())
        }
    }

    /// 获取窗口状态
    pub fn get_window_state(app: &AppHandle) -> Result<WindowState, String> {
        let frontstage_visible = if let Some(window) = Self::get_frontstage(app) {
            window.is_visible().map_err(|e| e.to_string())?
        } else {
            false
        };

        let backstage_visible = if let Some(window) = Self::get_backstage(app) {
            window.is_visible().map_err(|e| e.to_string())?
        } else {
            false
        };

        Ok(WindowState {
            frontstage_visible,
            backstage_visible,
        })
    }

    /// 向幕前窗口发送内容更新
    pub fn send_to_frontstage(app: &AppHandle, event: FrontstageEvent) -> Result<(), String> {
        if let Some(window) = Self::get_frontstage(app) {
            window
                .emit("frontstage-update", event)
                .map_err(|e| e.to_string())
        } else {
            Err("Frontstage window not found".to_string())
        }
    }

    /// 向幕后窗口发送内容更新
    pub fn send_to_backstage(app: &AppHandle, event: BackstageEvent) -> Result<(), String> {
        if let Some(window) = Self::get_backstage(app) {
            window
                .emit("backstage-update", event)
                .map_err(|e| e.to_string())
        } else {
            Err("Backstage window not found".to_string())
        }
    }
}

/// 发送给幕前窗口的事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum FrontstageEvent {
    /// 更新正文内容
    ContentUpdate { text: String, chapter_id: String },
    /// AI 提示浮现
    AiHint {
        hint: String,
        position: HintPosition,
        duration_ms: u64,
    },
    /// AI 生成段落预览
    AiPreview { text: String, insert_position: usize },
    /// 章节切换
    ChapterSwitch { chapter_id: String, title: String },
    /// 保存状态更新
    SaveStatus { saved: bool, timestamp: Option<String> },
}

/// 发送给幕后窗口的事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum BackstageEvent {
    /// 幕前内容变更
    ContentChanged { text: String, chapter_id: String },
    /// 幕前请求生成
    GenerationRequested { chapter_id: String, context: String },
    /// 幕前窗口关闭
    FrontstageClosed,
    /// 幕前窗口获得焦点
    FrontstageFocused,
}

/// AI 提示位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HintPosition {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

/// 窗口相关 Tauri 命令
#[tauri::command]
pub fn show_frontstage(app: AppHandle) -> Result<(), String> {
    WindowManager::show_frontstage(&app)
}

#[tauri::command]
pub fn hide_frontstage(app: AppHandle) -> Result<(), String> {
    WindowManager::hide_frontstage(&app)
}

#[tauri::command]
pub fn toggle_frontstage(app: AppHandle) -> Result<bool, String> {
    WindowManager::toggle_frontstage(&app)
}

#[tauri::command]
pub fn get_window_state(app: AppHandle) -> Result<WindowState, String> {
    WindowManager::get_window_state(&app)
}

#[tauri::command]
pub fn send_ai_hint(
    app: AppHandle,
    hint: String,
    line: usize,
    column: usize,
) -> Result<(), String> {
    let event = FrontstageEvent::AiHint {
        hint,
        position: HintPosition {
            line,
            column,
            offset: 0,
        },
        duration_ms: 5000,
    };
    WindowManager::send_to_frontstage(&app, event)
}

#[tauri::command]
pub fn update_frontstage_content(app: AppHandle, text: String, chapter_id: String) -> Result<(), String> {
    let event = FrontstageEvent::ContentUpdate { text, chapter_id };
    WindowManager::send_to_frontstage(&app, event)
}