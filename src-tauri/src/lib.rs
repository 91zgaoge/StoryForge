#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod config;
mod llm;
mod agents;
mod memory;
mod vector;
mod workflow;
mod export;
mod prompts;
mod versions;
mod chat;
mod analytics;
mod skills;
mod mcp;
mod collab;
mod state;
mod router;
mod evolution;
mod embeddings;
mod utils;
mod window;
mod updater;

use tauri::{Manager, AppHandle};

use db::{DbPool, init_db, StoryRepository, CharacterRepository, ChapterRepository, CreateStoryRequest, CreateCharacterRequest, CreateChapterRequest};
use config::AppConfig;
use skills::{SkillManager, SkillCategory, SkillInfo};
use mcp::{McpClient, McpServerConfig};
use export::{StoryExporter, ExportConfig, ExportFormat, ExportResult};
use once_cell::sync::OnceCell;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;

use collab::websocket::WebSocketServer;


static DB_POOL: Lazy<Mutex<Option<DbPool>>> = Lazy::new(|| Mutex::new(None));
static APP_CONFIG: Lazy<Mutex<Option<AppConfig>>> = Lazy::new(|| Mutex::new(None));
static SKILL_MANAGER: OnceCell<Mutex<SkillManager>> = OnceCell::new();

fn get_pool() -> Option<DbPool> { DB_POOL.lock().unwrap().clone() }
fn get_config() -> Option<AppConfig> { APP_CONFIG.lock().unwrap().clone() }

#[derive(Serialize)]
struct DashboardState { current_story: Option<db::Story>, stories_count: usize, characters_count: usize, chapters_count: usize }

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_dir = app.path().app_data_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));
            std::fs::create_dir_all(&app_dir).ok();
            
            log::info!("App directory: {:?}", app_dir);

            match init_db(&app_dir) {
                Ok(pool) => {
                    log::info!("Database initialized successfully");
                    *DB_POOL.lock().unwrap() = Some(pool);
                }
                Err(e) => {
                    log::error!("Failed to initialize database: {}", e);
                }
            }
            let _ = SKILL_MANAGER.set(Mutex::new(SkillManager::new()));

            // Initialize embedding model
            let _ = embeddings::init_embedding_model();

            // Initialize LanceDB vector store
            let vector_db_path = app_dir.join("vector_db").to_string_lossy().to_string();
            std::fs::create_dir_all(&vector_db_path).ok();

            tauri::async_runtime::spawn(async move {
                let mut vector_store = LanceVectorStore::new(vector_db_path);
                if let Err(e) = vector_store.init().await {
                    log::error!("Failed to initialize vector store: {}", e);
                } else {
                    let _ = VECTOR_STORE.set(vector_store);
                    log::info!("Vector store initialized successfully");
                }
            });

            // Start WebSocket server for collaborative editing
            tauri::async_runtime::spawn(async move {
                // Try different ports if 8765 is taken
                let ports = [8765, 8766, 8767, 8768, 8769];
                for port in ports {
                    let ws_server = WebSocketServer::new();
                    match ws_server.start(port).await {
                        Ok(_) => {
                            log::info!("WebSocket server started on port {}", port);
                            break;
                        }
                        Err(e) => {
                            log::warn!("Failed to start WebSocket server on port {}: {}", port, e);
                        }
                    }
                }
            });

            // Configure window URLs
            if let Some(frontstage) = app.get_webview_window("frontstage") {
                if let Err(e) = frontstage.navigate(tauri::Url::parse("http://localhost:5174/frontstage.html").unwrap_or_else(|_| tauri::Url::parse("tauri://localhost/frontstage.html").unwrap())) {
                    log::warn!("Failed to navigate frontstage: {}", e);
                }
            }
            if let Some(backstage) = app.get_webview_window("backstage") {
                if let Err(e) = backstage.navigate(tauri::Url::parse("http://localhost:5174/index.html").unwrap_or_else(|_| tauri::Url::parse("tauri://localhost/index.html").unwrap())) {
                    log::warn!("Failed to navigate backstage: {}", e);
                }
                // Hide backstage initially
                let _ = backstage.hide();
            }
            log::info!("Windows initialized with custom URLs");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health_check, get_state, list_stories, create_story, update_story, delete_story,
            get_story_characters, create_character, update_character, delete_character,
            get_story_chapters, get_chapter, create_chapter, update_chapter, delete_chapter,
            get_skills, get_skills_by_category, import_skill, enable_skill, disable_skill, uninstall_skill, execute_skill,
            connect_mcp_server, call_mcp_tool, list_mcp_tools, execute_mcp_tool,
            search_similar, embed_chapter,
            export_story,
            // Window management commands
            window::show_frontstage,
            window::hide_frontstage,
            window::toggle_frontstage,
            window::get_window_state,
            window::send_ai_hint,
            window::update_frontstage_content,
            // Backstage communication commands
            notify_backstage_content_changed,
            notify_backstage_generation_requested,
            show_backstage,
            // Settings commands
            config::get_settings,
            config::save_settings,
            config::export_settings,
            config::import_settings,
            config::get_models,
            config::create_model,
            config::update_model,
            config::delete_model,
            config::set_active_model,
            config::get_agent_mappings,
            config::update_agent_mapping,
            config::test_model_connection,
            // LLM commands
            llm::commands::llm_generate,
            llm::commands::llm_generate_stream,
            llm::commands::llm_test_connection,
            llm::commands::llm_cancel_generation,
            // Agent commands
            agents::commands::agent_execute,
            agents::commands::agent_execute_stream,
            agents::commands::agent_cancel_task,
            agents::service::get_available_agents,
            // Updater commands
            updater::check_update,
            updater::install_update,
            updater::get_current_version,
            updater::open_update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}

#[tauri::command]
fn health_check() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[tauri::command]
fn get_state() -> Result<DashboardState, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let stories = StoryRepository::new(pool.clone()).get_all().map_err(|e| e.to_string())?;
    let chars_count: usize = stories.iter().map(|s| CharacterRepository::new(pool.clone()).get_by_story(&s.id).map(|c| c.len()).unwrap_or(0)).sum();
    Ok(DashboardState { current_story: stories.first().cloned(), stories_count: stories.len(), characters_count: chars_count, chapters_count: 0 })
}

#[tauri::command]
fn list_stories() -> Result<Vec<db::Story>, String> {
    StoryRepository::new(get_pool().ok_or("DB not initialized")?).get_all().map_err(|e| e.to_string())
}

#[tauri::command]
fn create_story(title: String, description: Option<String>, genre: Option<String>) -> Result<db::Story, String> {
    StoryRepository::new(get_pool().ok_or("DB not initialized")?).create(CreateStoryRequest { title, description, genre }).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_story(id: String, title: Option<String>, description: Option<String>, tone: Option<String>, pacing: Option<String>) -> Result<(), String> {
    let req = db::UpdateStoryRequest { title, description, tone, pacing };
    StoryRepository::new(get_pool().ok_or("DB not initialized")?).update(&id, &req).map_err(|e| e.to_string()).map(|_| ())
}

#[tauri::command]
fn delete_story(id: String) -> Result<(), String> {
    StoryRepository::new(get_pool().ok_or("DB not initialized")?).delete(&id).map_err(|e| e.to_string()).map(|_| ())
}

#[tauri::command]
fn get_story_characters(story_id: String) -> Result<Vec<db::Character>, String> {
    CharacterRepository::new(get_pool().ok_or("DB not initialized")?).get_by_story(&story_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_character(story_id: String, name: String, background: Option<String>) -> Result<db::Character, String> {
    CharacterRepository::new(get_pool().ok_or("DB not initialized")?).create(CreateCharacterRequest { story_id, name, background }).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_character(id: String, name: Option<String>, background: Option<String>, personality: Option<String>, goals: Option<String>) -> Result<(), String> {
    CharacterRepository::new(get_pool().ok_or("DB not initialized")?).update(&id, name, background, personality, goals).map_err(|e| e.to_string()).map(|_| ())
}

#[tauri::command]
fn delete_character(id: String) -> Result<(), String> {
    CharacterRepository::new(get_pool().ok_or("DB not initialized")?).delete(&id).map_err(|e| e.to_string()).map(|_| ())
}

#[tauri::command]
fn get_story_chapters(story_id: String) -> Result<Vec<db::Chapter>, String> {
    db::ChapterRepository::new(get_pool().ok_or("DB not initialized")?).get_by_story(&story_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_chapter(id: String) -> Result<Option<db::Chapter>, String> {
    db::ChapterRepository::new(get_pool().ok_or("DB not initialized")?).get_by_id(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_chapter(id: String, title: Option<String>, outline: Option<String>, content: Option<String>) -> Result<(), String> {
    db::ChapterRepository::new(get_pool().ok_or("DB not initialized")?).update(&id, title, outline, content).map_err(|e| e.to_string()).map(|_| ())
}

#[tauri::command]
fn delete_chapter(id: String) -> Result<(), String> {
    db::ChapterRepository::new(get_pool().ok_or("DB not initialized")?).delete(&id).map_err(|e| e.to_string()).map(|_| ())
}

#[tauri::command]
fn create_chapter(story_id: String, chapter_number: i32, title: Option<String>, outline: Option<String>, content: Option<String>) -> Result<db::Chapter, String> {
    let req = CreateChapterRequest { story_id, chapter_number, title, outline, content };
    ChapterRepository::new(get_pool().ok_or("DB not initialized")?).create(req).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_skills() -> Result<Vec<SkillInfo>, String> {
    let skills = SKILL_MANAGER.get().ok_or("Skills not initialized")?.lock().map_err(|e| e.to_string())?.get_all_skills();
    Ok(skills.into_iter().map(SkillInfo::from).collect())
}

#[tauri::command]
fn get_skills_by_category(category: String) -> Result<Vec<SkillInfo>, String> {
    let cat = match category.as_str() {
        "writing" => SkillCategory::Writing, "analysis" => SkillCategory::Analysis,
        "character" => SkillCategory::Character, "world_building" => SkillCategory::WorldBuilding,
        "style" => SkillCategory::Style, "plot" => SkillCategory::Plot,
        "export" => SkillCategory::Export, "integration" => SkillCategory::Integration,
        _ => SkillCategory::Custom,
    };
    let skills = SKILL_MANAGER.get().ok_or("Skills not initialized")?.lock().map_err(|e| e.to_string())?.get_skills_by_category(cat);
    Ok(skills.into_iter().map(SkillInfo::from).collect())
}

#[tauri::command]
fn import_skill(path: String) -> Result<SkillInfo, String> {
    let skill = SKILL_MANAGER.get().ok_or("Skills not initialized")?.lock().map_err(|e| e.to_string())?.import_skill(std::path::Path::new(&path))?;
    Ok(SkillInfo::from(skill))
}

#[tauri::command]
fn enable_skill(skill_id: String) -> Result<(), String> {
    SKILL_MANAGER.get().ok_or("Skills not initialized")?.lock().map_err(|e| e.to_string())?.enable_skill(&skill_id)
}

#[tauri::command]
fn disable_skill(skill_id: String) -> Result<(), String> {
    SKILL_MANAGER.get().ok_or("Skills not initialized")?.lock().map_err(|e| e.to_string())?.disable_skill(&skill_id)
}

#[tauri::command]
fn uninstall_skill(skill_id: String) -> Result<(), String> {
    SKILL_MANAGER.get().ok_or("Skills not initialized")?.lock().map_err(|e| e.to_string())?.uninstall_skill(&skill_id)
}

#[tauri::command]
fn execute_skill(skill_id: String, params: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
    let manager = SKILL_MANAGER.get().ok_or("Skills not initialized")?.lock().map_err(|e| e.to_string())?;
    let context = agents::AgentContext {
        story_id: String::new(),
        story_title: String::new(),
        genre: String::new(),
        tone: String::new(),
        pacing: String::new(),
        chapter_number: 0,
        characters: vec![],
        previous_chapters: vec![],
    };
    let result = manager.execute_skill(&skill_id, &context, params)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn connect_mcp_server(config: McpServerConfig) -> Result<Vec<mcp::McpTool>, String> {
    let mut client = McpClient::new(config);
    client.connect().await.map_err(|e| e.to_string())?;
    Ok(client.get_tools().clone())
}

#[tauri::command]
async fn call_mcp_tool(config: McpServerConfig, tool_name: String, arguments: serde_json::Value) -> Result<serde_json::Value, String> {
    let mut client = McpClient::new(config);
    client.connect().await.map_err(|e| e.to_string())?;
    client.call_tool(&tool_name, arguments).await.map_err(|e| e.to_string())
}

// Vector Search Commands (LanceDB)
use vector::{LanceVectorStore, SearchResult};

static VECTOR_STORE: OnceCell<LanceVectorStore> = OnceCell::new();

#[tauri::command]
async fn search_similar(story_id: String, query: String, top_k: Option<usize>) -> Result<Vec<SearchResult>, String> {
    let store = VECTOR_STORE.get().ok_or("Vector store not initialized")?;
    store.search(&story_id, &query, top_k.unwrap_or(5))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn embed_chapter(chapter_id: String, content: String) -> Result<(), String> {
    use embeddings::embed_text;
    use vector::VectorRecord;

    let store = VECTOR_STORE.get().ok_or("Vector store not initialized")?;

    // 生成嵌入向量
    let embedding = embed_text(&content).map_err(|e| e.to_string())?;

    let record = VectorRecord {
        id: format!("{}", uuid::Uuid::new_v4()),
        story_id: String::new(), // 需要从chapter_id查询
        chapter_id,
        chapter_number: 0,
        text: content.chars().take(500).collect(),
        record_type: "chapter".to_string(),
        embedding,
    };

    store.add_record(record).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_mcp_tools() -> Result<Vec<mcp::McpTool>, String> {
    let config = mcp::McpServerConfig {
        id: "builtin".to_string(),
        name: "Built-in Tools".to_string(),
        command: String::new(),
        args: vec![],
        env: HashMap::new(),
        timeout_seconds: 30,
    };

    let server = mcp::McpServer::new(config);
    Ok(server.get_tools())
}

#[tauri::command]
async fn execute_mcp_tool(tool_name: String, arguments: serde_json::Value) -> Result<serde_json::Value, String> {
    let config = mcp::McpServerConfig {
        id: "builtin".to_string(),
        name: "Built-in Tools".to_string(),
        command: String::new(),
        args: vec![],
        env: HashMap::new(),
        timeout_seconds: 30,
    };

    let server = mcp::McpServer::new(config);
    server.start().await.map_err(|e| e.to_string())?;

    let result = server.execute_tool(&tool_name, arguments).await
        .map_err(|e| e.to_string())?;

    Ok(result)
}

#[derive(Debug, Deserialize)]
struct ExportOptions {
    story_id: String,
    format: String,
    include_metadata: Option<bool>,
    include_outline: Option<bool>,
    include_characters: Option<bool>,
}
#[tauri::command]
async fn export_story(options: ExportOptions, app_handle: tauri::AppHandle) -> Result<ExportResult, String> {
    let pool = get_pool().ok_or("Database not initialized")?;

    let story = StoryRepository::new(pool.clone())
        .get_by_id(&options.story_id)
        .map_err(|e| e.to_string())?
        .ok_or("Story not found")?;

    let chapters = ChapterRepository::new(pool.clone())
        .get_by_story(&options.story_id)
        .map_err(|e| e.to_string())?;

    let characters = CharacterRepository::new(pool.clone())
        .get_by_story(&options.story_id)
        .map_err(|e| e.to_string())?;

    let format = match options.format.as_str() {
        "markdown" => ExportFormat::Markdown,
        "pdf" => ExportFormat::Pdf,
        "epub" => ExportFormat::Epub,
        "html" => ExportFormat::Html,
        "txt" => ExportFormat::PlainText,
        "json" => ExportFormat::Json,
        _ => ExportFormat::Markdown,
    };

    let extension = match format {
        ExportFormat::Markdown => "md",
        ExportFormat::Pdf => "pdf",
        ExportFormat::Epub => "epub",
        ExportFormat::Html => "html",
        ExportFormat::PlainText => "txt",
        ExportFormat::Json => "json",
    };

    let safe_title = story.title.replace(|c: char| !c.is_alphanumeric(), "_");
    let filename = format!("{}_{}.{}", safe_title, chrono::Local::now().format("%Y%m%d"), extension);

    let export_dir = app_handle.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default())
        .join("exports");

    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;
    let output_path = export_dir.join(&filename);

    let config = ExportConfig {
        format,
        include_outline: options.include_outline.unwrap_or(true),
        include_metadata: options.include_metadata.unwrap_or(true),
        chapter_separator: "\n\n---\n\n".to_string(),
    };

    let exporter = StoryExporter::new();
    exporter.export_to_file(&story, &chapters, &characters, &config, &output_path)
        .map_err(|e| e.to_string())?;

    Ok(ExportResult {
        file_path: output_path.to_string_lossy().to_string(),
        content: std::fs::read_to_string(&output_path).unwrap_or_default(),
        format: options.format,
    })
}

// ===== 幕前/幕后通信命令 =====

/// 通知 backstage 内容已变更
#[tauri::command]
fn notify_backstage_content_changed(text: String, chapter_id: String, app: AppHandle) -> Result<(), String> {
    let event = window::BackstageEvent::ContentChanged { text, chapter_id };
    window::WindowManager::send_to_backstage(&app, event)
}

/// 通知 backstage 请求生成内容
#[tauri::command]
fn notify_backstage_generation_requested(chapter_id: String, context: String, app: AppHandle) -> Result<(), String> {
    let event = window::BackstageEvent::GenerationRequested { chapter_id, context };
    window::WindowManager::send_to_backstage(&app, event)
}

/// 显示 backstage 窗口
#[tauri::command]
fn show_backstage(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Backstage window not found".to_string())
    }
}
