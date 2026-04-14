//! V3 架构 Tauri 命令

use crate::db::*;
use crate::db::repositories_v3::*;
use crate::config::StudioManager;
use crate::memory::retention::RetentionManager;
use tauri::{command, AppHandle, Manager, State};
use chrono::Local;
use std::sync::Arc;
use tokio::sync::Mutex;

// ==================== 场景命令 ====================

#[command]
pub async fn create_scene(
    story_id: String,
    sequence_number: i32,
    title: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<Scene, String> {
    let repo = SceneRepository::new(pool.inner().clone());
    repo.create(&story_id, sequence_number, title.as_deref())
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_story_scenes(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<Scene>, String> {
    let repo = SceneRepository::new(pool.inner().clone());
    repo.get_by_story(&story_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_scene(
    scene_id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<Scene>, String> {
    let repo = SceneRepository::new(pool.inner().clone());
    repo.get_by_id(&scene_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn update_scene(
    scene_id: String,
    updates: SceneUpdate,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = SceneRepository::new(pool.inner().clone());
    repo.update(&scene_id, &updates)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn delete_scene(
    scene_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = SceneRepository::new(pool.inner().clone());
    repo.delete(&scene_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn reorder_scenes(
    story_id: String,
    scene_ids: Vec<String>,
    pool: State<'_, DbPool>,
) -> Result<(), String> {
    let repo = SceneRepository::new(pool.inner().clone());
    
    for (index, scene_id) in scene_ids.iter().enumerate() {
        repo.update_sequence(scene_id, (index + 1) as i32)
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

// ==================== 世界观命令 ====================

#[command]
pub async fn create_world_building(
    story_id: String,
    concept: String,
    pool: State<'_, DbPool>,
) -> Result<WorldBuilding, String> {
    let repo = WorldBuildingRepository::new(pool.inner().clone());
    repo.create(&story_id, &concept)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_world_building(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<WorldBuilding>, String> {
    let repo = WorldBuildingRepository::new(pool.inner().clone());
    repo.get_by_story(&story_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn update_world_building(
    id: String,
    concept: Option<String>,
    rules: Option<Vec<WorldRule>>,
    history: Option<String>,
    cultures: Option<Vec<Culture>>,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = WorldBuildingRepository::new(pool.inner().clone());
    repo.update(&id, concept.as_deref(), rules.as_deref(), history.as_deref(), cultures.as_deref())
        .map_err(|e| e.to_string())
}

// ==================== 文字风格命令 ====================

#[command]
pub async fn create_writing_style(
    story_id: String,
    name: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<WritingStyle, String> {
    let repo = WritingStyleRepository::new(pool.inner().clone());
    repo.create(&story_id, name.as_deref())
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_writing_style(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<WritingStyle>, String> {
    let repo = WritingStyleRepository::new(pool.inner().clone());
    repo.get_by_story(&story_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn update_writing_style(
    id: String,
    updates: WritingStyleUpdate,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = WritingStyleRepository::new(pool.inner().clone());
    repo.update(&id, &updates)
        .map_err(|e| e.to_string())
}

// ==================== 工作室配置命令 ====================

#[command]
pub async fn create_studio_config(
    story_id: String,
    app_handle: AppHandle,
    pool: State<'_, DbPool>,
) -> Result<StudioConfig, String> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    let manager = StudioManager::new(pool.inner().clone(), &app_dir);
    manager.create_default_studio(&story_id, "")
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_studio_config(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<StudioConfig>, String> {
    let repo = StudioConfigRepository::new(pool.inner().clone());
    repo.get_by_story(&story_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn update_studio_config(
    id: String,
    pen_name: Option<String>,
    llm_config: Option<LlmStudioConfig>,
    ui_config: Option<UiStudioConfig>,
    agent_bots: Option<Vec<AgentBotConfig>>,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = StudioConfigRepository::new(pool.inner().clone());
    repo.update(&id, pen_name.as_deref(), llm_config.as_ref(), ui_config.as_ref(), agent_bots.as_deref())
        .map_err(|e| e.to_string())
}

// ==================== 导入/导出命令 ====================

#[command]
pub async fn export_studio(
    request: StudioExportRequest,
    app_handle: AppHandle,
    pool: State<'_, DbPool>,
) -> Result<Vec<u8>, String> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    let manager = StudioManager::new(pool.inner().clone(), &app_dir);
    manager.export_studio(&request)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn import_studio(
    data: Vec<u8>,
    options: crate::config::studio_manager::ImportOptions,
    app_handle: AppHandle,
    pool: State<'_, DbPool>,
) -> Result<Story, String> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    let manager = StudioManager::new(pool.inner().clone(), &app_dir);
    manager.import_studio(&data, &options)
        .map_err(|e| e.to_string())
}

// ==================== 知识图谱命令 ====================

#[command]
pub async fn create_entity(
    story_id: String,
    name: String,
    entity_type: String,
    attributes: serde_json::Value,
    pool: State<'_, DbPool>,
) -> Result<Entity, String> {
    let repo = KnowledgeGraphRepository::new(pool.inner().clone());
    repo.create_entity(&story_id, &name, &entity_type, &attributes)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_story_entities(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<Entity>, String> {
    let repo = KnowledgeGraphRepository::new(pool.inner().clone());
    repo.get_entities_by_story(&story_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn create_relation(
    story_id: String,
    source_id: String,
    target_id: String,
    relation_type: String,
    strength: f32,
    pool: State<'_, DbPool>,
) -> Result<Relation, String> {
    let repo = KnowledgeGraphRepository::new(pool.inner().clone());
    repo.create_relation(&story_id, &source_id, &target_id, &relation_type, strength)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_entity_relations(
    entity_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<Relation>, String> {
    let repo = KnowledgeGraphRepository::new(pool.inner().clone());
    repo.get_relations_by_entity(&entity_id)
        .map_err(|e| e.to_string())
}

#[derive(Debug, serde::Serialize)]
pub struct StoryGraph {
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
}

#[command]
pub async fn get_story_graph(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<StoryGraph, String> {
    let repo = KnowledgeGraphRepository::new(pool.inner().clone());
    let entities = repo.get_entities_by_story(&story_id)
        .map_err(|e| e.to_string())?;
    let relations = repo.get_relations_by_story(&story_id)
        .map_err(|e| e.to_string())?;
    Ok(StoryGraph { entities, relations })
}

#[command]
pub async fn get_retention_report(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<crate::memory::retention::RetentionReport, String> {
    let repo = KnowledgeGraphRepository::new(pool.inner().clone());
    let entities = repo.get_entities_by_story(&story_id)
        .map_err(|e| e.to_string())?;
    
    let manager = RetentionManager::new();
    Ok(manager.generate_retention_report(&entities))
}

#[command]
pub async fn archive_forgotten_entities(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<crate::memory::retention::ArchiveResult, String> {
    let repo = KnowledgeGraphRepository::new(pool.inner().clone());
    let entities = repo.get_entities_by_story(&story_id)
        .map_err(|e| e.to_string())?;
    
    let manager = RetentionManager::new();
    let forgotten = manager.get_forgotten_entities(&entities);
    
    let mut archived = Vec::new();
    for (entity, _) in &forgotten {
        repo.archive_entity(&entity.id)
            .map_err(|e| e.to_string())?;
        archived.push(entity.name.clone());
    }
    
    Ok(crate::memory::retention::ArchiveResult {
        archived_count: archived.len(),
        archived_entities: archived,
        story_id,
    })
}

#[command]
pub async fn restore_archived_entity(
    entity_id: String,
    pool: State<'_, DbPool>,
) -> Result<Entity, String> {
    let repo = KnowledgeGraphRepository::new(pool.inner().clone());
    repo.restore_entity(&entity_id)
        .map_err(|e| e.to_string())?;
    
    repo.get_entity_by_id(&entity_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Entity not found".to_string())
}

#[command]
pub async fn get_archived_entities(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<Entity>, String> {
    let repo = KnowledgeGraphRepository::new(pool.inner().clone());
    repo.get_archived_entities(&story_id)
        .map_err(|e| e.to_string())
}

// ==================== 场景版本命令 ====================

use crate::db::models_v3::{SceneVersion, CreatorType};
use crate::db::repositories_v3::SceneVersionRepository;
use crate::versions::service::{SceneVersionService, VersionDiff, VersionStats, RestoreResult};

#[command]
pub async fn get_scene_versions(
    scene_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<SceneVersion>, String> {
    let repo = SceneVersionRepository::new(pool.inner().clone());
    repo.get_versions(&scene_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_scene_version(
    version_id: String,
    pool: State<'_, DbPool>,
) -> Result<Option<SceneVersion>, String> {
    let repo = SceneVersionRepository::new(pool.inner().clone());
    repo.get_version(&version_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn create_scene_version(
    scene_id: String,
    change_summary: String,
    created_by: String,
    confidence_score: Option<f32>,
    pool: State<'_, DbPool>,
) -> Result<SceneVersion, String> {
    let scene_repo = crate::db::repositories_v3::SceneRepository::new(pool.inner().clone());
    let version_repo = SceneVersionRepository::new(pool.inner().clone());
    
    let scene = scene_repo.get_by_id(&scene_id)
        .map_err(|e| e.to_string())?
        .ok_or("Scene not found")?;
    
    let creator = match created_by.as_str() {
        "user" => CreatorType::User,
        "ai" => CreatorType::Ai,
        _ => CreatorType::System,
    };
    
    version_repo.create_version(&scene, &change_summary, creator, None, confidence_score)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn compare_scene_versions(
    from_version_id: String,
    to_version_id: String,
    pool: State<'_, DbPool>,
) -> Result<VersionDiff, String> {
    let service = SceneVersionService::new(pool.inner().clone());
    service.compare_versions(&from_version_id, &to_version_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn restore_scene_version(
    scene_id: String,
    version_id: String,
    restored_by: String,
    pool: State<'_, DbPool>,
) -> Result<SceneVersion, String> {
    let service = SceneVersionService::new(pool.inner().clone());
    let result = service.restore_version(&scene_id, &version_id, &restored_by)
        .map_err(|e| e.to_string())?;
    Ok(result.new_version)
}

#[command]
pub async fn get_scene_version_stats(
    scene_id: String,
    pool: State<'_, DbPool>,
) -> Result<VersionStats, String> {
    let service = SceneVersionService::new(pool.inner().clone());
    service.get_version_stats(&scene_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn delete_scene_version(
    version_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = SceneVersionRepository::new(pool.inner().clone());
    repo.delete_version(&version_id)
        .map_err(|e| e.to_string())
}
