#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

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
mod commands_v3;
mod intent;
mod creative_engine;
mod subscription;
mod book_deconstruction;
mod task_system;
mod canonical_state;
mod audit;

#[cfg(test)]
mod test_utils;

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
pub static SKILL_MANAGER: OnceCell<Mutex<SkillManager>> = OnceCell::new();

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
                    app.manage(pool.clone());
                    *DB_POOL.lock().unwrap() = Some(pool);
                }
                Err(e) => {
                    log::error!("Failed to initialize database: {}", e);
                }
            }
            let _ = SKILL_MANAGER.set(Mutex::new(SkillManager::new(Some(crate::llm::LlmService::new(app.handle().clone())))));

            // Seed built-in StyleDNAs
            if let Some(pool) = get_pool() {
                let style_repo = db::repositories_v3::StyleDnaRepository::new(pool);
                match style_repo.get_builtin() {
                    Ok(existing) if existing.is_empty() => {
                        log::info!("[StyleDNA] Seeding built-in styles...");
                        for style in creative_engine::style::classic_styles::get_builtin_styles() {
                            if let Ok(dna_json) = serde_json::to_string(&style) {
                                let _ = style_repo.create(
                                    &style.meta.name,
                                    style.meta.author.as_deref(),
                                    &dna_json,
                                    true,
                                );
                            }
                        }
                        log::info!("[StyleDNA] Built-in styles seeded successfully");
                    }
                    Ok(_) => log::info!("[StyleDNA] Built-in styles already exist, skipping seed"),
                    Err(e) => log::warn!("[StyleDNA] Failed to check existing styles: {}", e),
                }
            }

            // Initialize embedding model
            let _ = embeddings::init_embedding_model();

            // Bootstrap task system
            if let Some(pool) = get_pool() {
                let app_handle = app.handle().clone();
                let task_service = task_system::service::TaskService::new(pool.clone(), app_handle.clone());
                let llm_service = llm::LlmService::new(app_handle.clone());
                let executor = std::sync::Arc::new(book_deconstruction::executor::BookDeconstructionExecutor::new(
                    pool.clone(),
                    llm_service,
                    app_handle.clone(),
                ));
                task_service.register_executor(executor);
                if let Err(e) = task_service.bootstrap() {
                    log::error!("Failed to bootstrap task system: {}", e);
                } else {
                    log::info!("Task system bootstrapped successfully");
                }
                app.manage(task_service);
            }

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
            if let Some(pool) = get_pool() {
                tauri::async_runtime::spawn(async move {
                    // Try different ports if 8765 is taken
                    let ports = [8765, 8766, 8767, 8768, 8769];
                    for port in ports {
                        let ws_server = WebSocketServer::with_pool(pool.clone());
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
            }

            // Ensure backstage is hidden on startup
            if let Some(backstage) = app.get_webview_window("backstage") {
                let _ = backstage.hide();
            }
            // Focus frontstage
            if let Some(frontstage) = app.get_webview_window("frontstage") {
                let _ = frontstage.set_focus();
            }

            // Disable default webview context menus on Windows
            #[cfg(target_os = "windows")]
            {
                for label in ["frontstage", "backstage"] {
                    if let Some(window) = app.get_webview_window(label) {
                        let _ = window.with_webview(|webview| {
                            let controller = webview.controller();
                            unsafe {
                                if let Ok(core) = controller.CoreWebView2() {
                                    if let Ok(settings) = core.Settings() {
                                        let _ = settings.SetAreDefaultContextMenusEnabled(false);
                                    }
                                }
                            }
                        });
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health_check, check_model_status, chat_completion, get_state, list_stories, create_story, update_story, delete_story,
            get_story_characters, create_character, update_character, delete_character,
            get_story_chapters, get_chapter, create_chapter, update_chapter, delete_chapter,
            get_skills, get_skill, get_skills_by_category, import_skill, enable_skill, disable_skill, uninstall_skill, execute_skill, update_skill, format_text,
            connect_mcp_server, call_mcp_tool, disconnect_mcp_server, get_mcp_connections, list_mcp_tools, execute_mcp_tool,
            search_similar, text_search_vectors, hybrid_search_vectors, embed_chapter,
            export_story,
            // Window management commands
            window::show_frontstage,
            window::hide_frontstage,
            window::toggle_frontstage,
            window::get_window_state,
            window::update_frontstage_content,
            // Backstage communication commands
            notify_backstage_content_changed,
            notify_backstage_generation_requested,
            notify_frontstage_content_changed,
            notify_frontstage_data_refresh,
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
            config::fetch_models,
            // LLM commands
            llm::commands::llm_generate,
            llm::commands::llm_generate_stream,
            llm::commands::llm_test_connection,
            llm::commands::llm_cancel_generation,
            // Intent commands
            parse_intent,
            execute_intent,
            record_feedback,
            // Agent commands
            agents::commands::agent_execute,
            agents::commands::agent_execute_stream,
            agents::commands::agent_cancel_task,
            agents::commands::writer_agent_execute,
            agents::commands::auto_write,
            agents::commands::auto_write_cancel,
            agents::commands::auto_revise,
            agents::commands::auto_revise_cancel,
            agents::service::get_available_agents,
            // Subscription commands
            subscription::commands::get_subscription_status,
            subscription::commands::get_quota_detail,
            subscription::commands::check_auto_write_quota,
            subscription::commands::check_auto_revise_quota,
            subscription::commands::dev_upgrade_subscription,
            subscription::commands::dev_downgrade_subscription,
            // Updater commands
            updater::check_update,
            updater::install_update,
            updater::get_current_version,
            updater::open_update_settings,
            // V3 Architecture commands
            commands_v3::create_scene,
            commands_v3::get_story_scenes,
            commands_v3::get_scene,
            commands_v3::update_scene,
            commands_v3::delete_scene,
            commands_v3::reorder_scenes,
            commands_v3::create_world_building,
            commands_v3::get_world_building,
            commands_v3::update_world_building,
            commands_v3::create_writing_style,
            commands_v3::get_writing_style,
            commands_v3::update_writing_style,
            commands_v3::create_studio_config,
            commands_v3::get_studio_config,
            commands_v3::update_studio_config,
            commands_v3::export_studio,
            commands_v3::import_studio,
            commands_v3::create_entity,
            commands_v3::update_entity,
            commands_v3::get_story_entities,
            commands_v3::create_relation,
            commands_v3::get_entity_relations,
            commands_v3::get_story_graph,
            commands_v3::get_retention_report,
            commands_v3::archive_forgotten_entities,
            commands_v3::restore_archived_entity,
            commands_v3::get_archived_entities,
            // Scene annotations
            commands_v3::create_scene_annotation,
            commands_v3::get_scene_annotations,
            commands_v3::get_story_unresolved_annotations,
            commands_v3::update_scene_annotation,
            commands_v3::resolve_scene_annotation,
            commands_v3::unresolve_scene_annotation,
            commands_v3::delete_scene_annotation,
            // Text inline annotations
            commands_v3::create_text_annotation,
            commands_v3::get_text_annotations_by_chapter,
            commands_v3::get_text_annotations_by_scene,
            commands_v3::update_text_annotation,
            commands_v3::resolve_text_annotation,
            commands_v3::unresolve_text_annotation,
            commands_v3::delete_text_annotation,
            // Commentator agent
            commands_v3::generate_paragraph_commentaries,
            // Memory compressor
            commands_v3::compress_content,
            commands_v3::compress_scene,
            // Knowledge distiller
            commands_v3::distill_story_knowledge,
            commands_v3::get_story_summaries,
            commands_v3::update_story_summary,
            commands_v3::delete_story_summary,
            // Novel creation wizard commands
            commands_v3::generate_world_building_options,
            commands_v3::generate_character_profiles,
            commands_v3::generate_writing_styles,
            commands_v3::generate_first_scene,
            commands_v3::create_story_with_wizard,
            // Scene version commands
            commands_v3::get_scene_versions,
            commands_v3::get_scene_version,
            commands_v3::create_scene_version,
            commands_v3::compare_scene_versions,
            commands_v3::restore_scene_version,
            commands_v3::get_scene_version_stats,
            commands_v3::delete_scene_version,
            commands_v3::get_scene_version_chain,
            commands_v3::get_version_change_tracks,
            // Change tracking (revision mode)
            commands_v3::track_change,
            commands_v3::accept_change,
            commands_v3::reject_change,
            commands_v3::get_pending_changes,
            commands_v3::accept_all_changes,
            commands_v3::reject_all_changes,
            // Comment threads (revision mode)
            commands_v3::create_comment_thread,
            commands_v3::add_comment_message,
            commands_v3::get_comment_threads,
            commands_v3::resolve_comment_thread,
            commands_v3::reopen_comment_thread,
            commands_v3::delete_comment_thread,
            commands_v3::run_creation_workflow,
            commands_v3::list_style_dnas,
            commands_v3::set_story_style_dna,
            commands_v3::analyze_style_sample,
            // Book deconstruction commands
            book_deconstruction::commands::upload_book,
            book_deconstruction::commands::get_analysis_status,
            book_deconstruction::commands::get_book_analysis,
            book_deconstruction::commands::list_reference_books,
            book_deconstruction::commands::delete_reference_book,
            book_deconstruction::commands::convert_book_to_story,
            book_deconstruction::commands::cancel_book_analysis,
            // Task system commands
            task_system::commands::create_task,
            task_system::commands::update_task,
            task_system::commands::delete_task,
            task_system::commands::list_tasks,
            task_system::commands::get_task,
            task_system::commands::trigger_task,
            task_system::commands::cancel_task,
            task_system::commands::get_task_logs,
            // Foreshadowing tracker commands
            commands_v3::get_story_foreshadowings,
            commands_v3::create_foreshadowing,
            commands_v3::update_foreshadowing_status,
            // Payoff Ledger commands
            commands_v3::get_payoff_ledger,
            commands_v3::detect_overdue_payoffs,
            commands_v3::recommend_payoff_timing,
            commands_v3::update_payoff_ledger_fields,
            // Canonical state commands
            get_canonical_state,
            // Structured outline commands
            commands_v3::generate_scene_outline,
            commands_v3::generate_scene_draft,
            // Audit commands
            audit::commands::audit_scene,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessageItem {
    pub role: String,
    pub content: String,
}

#[tauri::command]
async fn chat_completion(
    base_url: String,
    api_key: Option<String>,
    model: String,
    messages: Vec<ChatMessageItem>,
    max_tokens: i32,
    temperature: f32,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let mut request = client
        .post(format!("{}/chat/completions", base_url))
        .header("Content-Type", "application/json");

    if let Some(key) = api_key {
        if !key.is_empty() {
            request = request.header("Authorization", format!("Bearer {}", key));
        }
    }

    let body = serde_json::json!({
        "model": model,
        "messages": messages.iter().map(|m| serde_json::json!({
            "role": m.role,
            "content": m.content
        })).collect::<Vec<_>>(),
        "max_tokens": max_tokens,
        "temperature": temperature,
        "stream": false,
    });

    let response = request.json(&body).send().await.map_err(|e| e.to_string())?;
    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status, text));
    }

    let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(data)
}

#[tauri::command]
async fn check_model_status(app_handle: AppHandle) -> Result<String, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app dir: {}", e))?;
    let config = config::AppConfig::load(&app_dir).map_err(|e| e.to_string())?;
    let active_profile_id = config.active_llm_profile.as_deref()
        .or(config.llm_profiles.values().find(|p| p.is_default).map(|p| p.id.as_str()))
        .or(config.llm_profiles.keys().next().map(|s| s.as_str()))
        .ok_or("No LLM profile configured")?;

    let profile = config.llm_profiles.get(active_profile_id)
        .ok_or("Active LLM profile not found")?;

    let base_url = profile.api_base.clone()
        .or(config.llm.api_base.clone())
        .unwrap_or_else(|| match profile.provider {
            config::settings::LlmProvider::OpenAI => "https://api.openai.com/v1".to_string(),
            config::settings::LlmProvider::Anthropic => "https://api.anthropic.com".to_string(),
            config::settings::LlmProvider::Ollama => "http://localhost:11434".to_string(),
            config::settings::LlmProvider::DeepSeek => "https://api.deepseek.com".to_string(),
            _ => "http://localhost:11434".to_string(),
        });

    let api_key = if profile.api_key.is_empty() {
        config.llm.api_key.clone()
    } else {
        profile.api_key.clone()
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let api_key_ref = if api_key.is_empty() { None } else { Some(api_key.as_str()) };

    // 探测策略：只要收到任何 HTTP 响应（不论状态码）即视为网络可通
    // 1. GET base_url（根路径，最宽容）
    if client.get(&base_url).send().await.is_ok() {
        return Ok("connected".to_string());
    }

    // 2. GET /models（OpenAI 标准）
    let mut req = client.get(format!("{}/models", base_url));
    if let Some(key) = api_key_ref {
        req = req.header("Authorization", format!("Bearer {}", key));
    }
    if req.send().await.is_ok() {
        return Ok("connected".to_string());
    }

    // 3. POST /chat/completions
    let mut req = client.post(format!("{}/chat/completions", base_url));
    if let Some(key) = api_key_ref {
        req = req.header("Authorization", format!("Bearer {}", key));
    }
    req = req.header("Content-Type", "application/json");
    if req.body(r#"{"model":"test","messages":[{"role":"user","content":"hi"}],"max_tokens":1}"#).send().await.is_ok() {
        return Ok("connected".to_string());
    }

    // 4. POST /v1/chat/completions（部分服务 base_url 不含 /v1）
    let mut req = client.post(format!("{}/v1/chat/completions", base_url));
    if let Some(key) = api_key_ref {
        req = req.header("Authorization", format!("Bearer {}", key));
    }
    req = req.header("Content-Type", "application/json");
    if req.body(r#"{"model":"test","messages":[{"role":"user","content":"hi"}],"max_tokens":1}"#).send().await.is_ok() {
        return Ok("connected".to_string());
    }

    Ok("disconnected".to_string())
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
    StoryRepository::new(get_pool().ok_or("DB not initialized")?).create(CreateStoryRequest { title, description, genre, style_dna_id: None }).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_story(
    id: String,
    title: Option<String>,
    description: Option<String>,
    tone: Option<String>,
    pacing: Option<String>,
    style_dna_id: Option<String>,
    methodology_id: Option<String>,
    methodology_step: Option<i32>,
) -> Result<(), String> {
    let req = db::UpdateStoryRequest { title, description, tone, pacing, style_dna_id, methodology_id, methodology_step };
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
    let character = CharacterRepository::new(get_pool().ok_or("DB not initialized")?).create(CreateCharacterRequest { story_id, name, background }).map_err(|e| e.to_string())?;

    // OnCharacterCreate hook
    if let Some(manager) = SKILL_MANAGER.get() {
        if let Ok(skill_manager) = manager.lock() {
            let story_id = character.story_id.clone();
            let character_id = character.id.clone();
            let character_name = character.name.clone();
            let skill_manager = skill_manager.clone();
            tauri::async_runtime::spawn(async move {
                let context = crate::agents::AgentContext::minimal(story_id, String::new());
                let data = serde_json::json!({ "character_id": character_id, "character_name": character_name });
                let _ = skill_manager.execute_hooks(crate::skills::HookEvent::OnCharacterCreate, &context, data).await;
                log::info!("Hook executed: {:?}", crate::skills::HookEvent::OnCharacterCreate);
            });
        }
    }

    Ok(character)
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
fn update_chapter(id: String, title: Option<String>, outline: Option<String>, content: Option<String>, word_count: Option<i32>, app: AppHandle) -> Result<(), String> {
    let result = db::ChapterRepository::new(get_pool().ok_or("DB not initialized")?).update(&id, title, outline, content, word_count).map_err(|e| e.to_string());
    if result.is_ok() {
        let _ = window::WindowManager::send_to_frontstage(&app, window::FrontstageEvent::SaveStatus { saved: true, timestamp: Some(chrono::Local::now().to_rfc3339()) });
    }
    result.map(|_| ())
}

#[tauri::command]
fn delete_chapter(id: String) -> Result<(), String> {
    db::ChapterRepository::new(get_pool().ok_or("DB not initialized")?).delete(&id).map_err(|e| e.to_string()).map(|_| ())
}

#[tauri::command]
fn create_chapter(story_id: String, chapter_number: i32, title: Option<String>, outline: Option<String>, content: Option<String>) -> Result<db::Chapter, String> {
    let req = CreateChapterRequest { story_id, chapter_number, title, outline, content };
    let chapter = ChapterRepository::new(get_pool().ok_or("DB not initialized")?).create(req).map_err(|e| e.to_string())?;

    // AfterChapterSave hook
    if let Some(manager) = SKILL_MANAGER.get() {
        if let Ok(skill_manager) = manager.lock() {
            let story_id = chapter.story_id.clone();
            let chapter_id = chapter.id.clone();
            let chapter_number = chapter.chapter_number;
            let skill_manager = skill_manager.clone();
            tauri::async_runtime::spawn(async move {
                let context = crate::agents::AgentContext::minimal(story_id, String::new());
                let data = serde_json::json!({ "chapter_id": chapter_id, "chapter_number": chapter_number });
                let _ = skill_manager.execute_hooks(crate::skills::HookEvent::AfterChapterSave, &context, data).await;
                log::info!("Hook executed: {:?}", crate::skills::HookEvent::AfterChapterSave);
            });
        }
    }

    Ok(chapter)
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
fn get_skill(skill_id: String) -> Result<SkillInfo, String> {
    let skill = SKILL_MANAGER.get().ok_or("Skills not initialized")?.lock().map_err(|e| e.to_string())?.get_skill(&skill_id);
    skill.map(SkillInfo::from).ok_or_else(|| "Skill not found".to_string())
}

#[tauri::command]
fn update_skill(skill_id: String, manifest: skills::SkillManifest) -> Result<(), String> {
    SKILL_MANAGER.get().ok_or("Skills not initialized")?.lock().map_err(|e| e.to_string())?.update_skill(&skill_id, manifest)
}

#[tauri::command]
async fn execute_skill(
    skill_id: String,
    params: HashMap<String, serde_json::Value>,
    app_handle: AppHandle,
) -> Result<serde_json::Value, String> {
    let mut params = params;
    let story_id = params.remove("story_id").and_then(|v| v.as_str().map(|s| s.to_string()));

    // Build context from database if story_id is provided
    let context = if let Some(story_id) = story_id {
        match app_handle.try_state::<DbPool>() {
            Some(pool_state) => {
                let pool = pool_state.inner().clone();
                let builder = creative_engine::StoryContextBuilder::new(pool);
                match builder.build_quick(&story_id) {
                    Ok(ctx) => ctx,
                    Err(e) => {
                        log::warn!("[execute_skill] StoryContextBuilder failed: {}, using minimal context", e);
                        agents::AgentContext::minimal(story_id, String::new())
                    }
                }
            }
            None => {
                log::warn!("[execute_skill] DbPool not available, using minimal context");
                agents::AgentContext::minimal(story_id, String::new())
            }
        }
    } else {
        agents::AgentContext {
            story_id: String::new(),
            story_title: String::new(),
            genre: String::new(),
            tone: String::new(),
            pacing: String::new(),
            chapter_number: 0,
            characters: vec![],
            previous_chapters: vec![],
            current_content: None,
            selected_text: None,
            world_rules: None,
            scene_structure: None,
            methodology_id: None,
            methodology_step: None,
            style_dna_id: None,
        }
    };

    // Execute skill
    let manager = {
        let guard = SKILL_MANAGER.get().ok_or("Skills not initialized")?.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };
    
    let result = manager.execute_skill(&skill_id, &context, params).await?;
    
    if !result.success {
        return Err(result.error.unwrap_or("Skill execution failed".to_string()));
    }
    
    // If LLM was already called (PromptRuntime with llm_service), return content directly
    if let Some(content) = result.data.get("content").and_then(|v| v.as_str()) {
        return Ok(serde_json::json!({
            "success": true,
            "content": content,
            "model": result.data.get("model").and_then(|v| v.as_str()).unwrap_or(""),
            "tokens_used": result.data.get("tokens_used").and_then(|v| v.as_i64()).unwrap_or(0),
            "execution_time_ms": result.execution_time_ms,
        }));
    }
    
    // Fallback: skill returned prompts but no LLM result, call LLM manually
    let system_prompt = result.data.get("system_prompt").and_then(|v| v.as_str()).unwrap_or("");
    let user_prompt = result.data.get("user_prompt").and_then(|v| v.as_str()).unwrap_or("");
    
    if system_prompt.is_empty() && user_prompt.is_empty() {
        return Err("Skill did not produce a valid prompt".to_string());
    }

    let llm_service = crate::llm::LlmService::new(app_handle);
    let full_prompt = if system_prompt.is_empty() {
        user_prompt.to_string()
    } else {
        format!("[系统指令]\n{}\n\n[用户请求]\n{}", system_prompt, user_prompt)
    };
    
    let response = llm_service.generate(full_prompt, Some(2000), Some(0.7)).await?;
    
    Ok(serde_json::json!({
        "success": true,
        "content": response.content,
        "model": response.model,
        "tokens_used": response.tokens_used,
        "execution_time_ms": result.execution_time_ms,
    }))
}

/// 使用 text_formatter skill 对文本进行智能排版
#[tauri::command]
async fn format_text(content: String, app: AppHandle) -> Result<String, String> {
    let result = execute_skill(
        "builtin.text_formatter".to_string(),
        {
            let mut p = HashMap::new();
            p.insert("content".to_string(), serde_json::Value::String(content));
            p
        },
        app,
    ).await?;
    
    result.get("content")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "LLM returned empty content".to_string())
}

use tokio::sync::Mutex as TokioMutex;

static MCP_CONNECTIONS: Lazy<TokioMutex<HashMap<String, mcp::McpClient>>> =
    Lazy::new(|| TokioMutex::new(HashMap::new()));

#[tauri::command]
async fn connect_mcp_server(config: McpServerConfig) -> Result<Vec<mcp::McpTool>, String> {
    let mut client = McpClient::new(config.clone());
    client.connect().await.map_err(|e| e.to_string())?;
    let tools = client.get_tools().clone();
    let mut connections = MCP_CONNECTIONS.lock().await;
    connections.insert(config.id.clone(), client);
    log::info!("[MCP] Connected to server {} ({}), {} tools available", config.name, config.id, tools.len());
    Ok(tools)
}

#[tauri::command]
async fn call_mcp_tool(server_id: String, tool_name: String, arguments: serde_json::Value) -> Result<serde_json::Value, String> {
    let mut connections = MCP_CONNECTIONS.lock().await;
    let client = connections.get_mut(&server_id)
        .ok_or_else(|| format!("MCP server {} not connected", server_id))?;
    client.call_tool(&tool_name, arguments).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn disconnect_mcp_server(server_id: String) -> Result<(), String> {
    let mut connections = MCP_CONNECTIONS.lock().await;
    if let Some(mut client) = connections.remove(&server_id) {
        client.disconnect().await;
        log::info!("[MCP] Disconnected from server {}", server_id);
    }
    Ok(())
}

#[tauri::command]
async fn get_mcp_connections() -> Result<Vec<serde_json::Value>, String> {
    let connections = MCP_CONNECTIONS.lock().await;
    let result: Vec<serde_json::Value> = connections.iter()
        .map(|(id, client)| {
            serde_json::json!({
                "id": id,
                "tools": client.get_tools().len(),
                "resources": client.get_resources().len(),
            })
        })
        .collect();
    Ok(result)
}

// Vector Search Commands (LanceDB)
use vector::{LanceVectorStore, SearchResult};

static VECTOR_STORE: OnceCell<LanceVectorStore> = OnceCell::new();

#[tauri::command]
async fn search_similar(story_id: String, query: String, top_k: Option<usize>) -> Result<Vec<SearchResult>, String> {
    use embeddings::embed_text;
    
    let store = VECTOR_STORE.get().ok_or("Vector store not initialized")?;
    
    // 生成查询向量
    let query_embedding = embed_text(&query).map_err(|e| e.to_string())?;
    
    store.search(&story_id, query_embedding, top_k.unwrap_or(5))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn text_search_vectors(story_id: String, query: String, top_k: Option<usize>) -> Result<Vec<SearchResult>, String> {
    let store = VECTOR_STORE.get().ok_or("Vector store not initialized")?;
    store.text_search(&story_id, &query, top_k.unwrap_or(5))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn hybrid_search_vectors(story_id: String, query: String, top_k: Option<usize>) -> Result<Vec<SearchResult>, String> {
    use embeddings::embed_text;
    
    let store = VECTOR_STORE.get().ok_or("Vector store not initialized")?;
    let query_embedding = embed_text(&query).map_err(|e| e.to_string())?;
    
    store.hybrid_search(&story_id, &query, query_embedding, top_k.unwrap_or(5))
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

// Intent Parser Command
#[tauri::command]
async fn parse_intent(user_input: String, app_handle: AppHandle) -> Result<intent::Intent, String> {
    let parser = intent::IntentParser::new(app_handle);
    parser.parse(&user_input).await
}

// Intent Executor Command
#[tauri::command]
async fn execute_intent(
    intent: intent::Intent,
    story_id: String,
    app_handle: AppHandle,
) -> Result<intent::IntentExecutionResult, String> {
    let executor = intent::IntentExecutor::new(app_handle);
    executor.execute(intent, story_id).await
}

#[derive(Debug, Deserialize)]
struct RecordFeedbackRequest {
    story_id: String,
    scene_id: Option<String>,
    chapter_id: Option<String>,
    feedback_type: String,
    agent_type: Option<String>,
    original_ai_text: String,
    final_text: Option<String>,
}

#[tauri::command]
async fn record_feedback(request: RecordFeedbackRequest) -> Result<(), String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let recorder = creative_engine::adaptive::FeedbackRecorder::new(pool.clone());
    let result = match request.feedback_type.as_str() {
        "accept" => recorder.record_accept(&request.story_id, &request.original_ai_text, request.agent_type.as_deref()),
        "reject" => recorder.record_reject(&request.story_id, &request.original_ai_text, request.agent_type.as_deref()),
        "modify" => recorder.record_modify(
            &request.story_id,
            &request.original_ai_text,
            request.final_text.as_deref().unwrap_or(""),
            request.agent_type.as_deref(),
        ),
        _ => Err("Unknown feedback type".to_string()),
    };
    
    // 异步触发偏好挖掘，让自适应学习系统形成闭环
    if result.is_ok() {
        let story_id = request.story_id.clone();
        tauri::async_runtime::spawn(async move {
            let engine = creative_engine::adaptive::AdaptiveLearningEngine::new(pool);
            match engine.mine_preferences(&story_id) {
                Ok(prefs) if !prefs.is_empty() => {
                    log::info!("[Adaptive] Mined {} preferences for story {}", prefs.len(), story_id);
                }
                Ok(_) => {}
                Err(e) => log::warn!("[Adaptive] Preference mining failed: {}", e),
            }
        });
    }
    
    result
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

/// 通知 frontstage 内容已变更
#[tauri::command]
fn notify_frontstage_content_changed(text: String, chapter_id: String, app: AppHandle) -> Result<(), String> {
    let event = window::FrontstageEvent::ContentUpdate { text, chapter_id };
    window::WindowManager::send_to_frontstage(&app, event)
}

/// 通知 frontstage 数据已刷新（幕后创建/修改了故事、章节等）
#[tauri::command]
fn notify_frontstage_data_refresh(entity: String, app: AppHandle) -> Result<(), String> {
    let event = window::FrontstageEvent::DataRefresh { entity };
    window::WindowManager::send_to_frontstage(&app, event)
}

/// 显示 backstage 窗口
#[tauri::command]
fn show_backstage(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("backstage") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        // 窗口可能被关闭，重新创建
        let window = tauri::WebviewWindowBuilder::new(
            &app,
            "backstage",
            tauri::WebviewUrl::App("index.html".into())
        )
        .title("草苔 - 幕后工作室")
        .inner_size(1200.0, 800.0)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// 获取故事的规范状态快照
#[tauri::command]
async fn get_canonical_state(story_id: String) -> Result<canonical_state::CanonicalStateSnapshot, String> {
    let pool = get_pool().ok_or("Database not initialized")?;
    let manager = canonical_state::CanonicalStateManager::new(pool);
    manager.get_snapshot(&story_id).await
}
