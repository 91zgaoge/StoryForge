//! V3 架构 Tauri 命令

use crate::db::*;
use crate::db::repositories_v3::*;
use crate::config::StudioManager;
use crate::memory::retention::RetentionManager;
use crate::memory::ingest::{IngestPipeline, IngestContent};
use crate::agents::novel_creation::{NovelCreationAgent, WorldBuildingOption, CharacterProfileOption, WritingStyleOption, SceneProposal, GenerationOptions};
use crate::llm::LlmService;
use serde::{Serialize, Deserialize};
use tauri::{command, AppHandle, Manager, State};


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
    repo.create_entity(&story_id, &name, &entity_type, &attributes, None)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn update_entity(
    entity_id: String,
    name: Option<String>,
    attributes: Option<serde_json::Value>,
    pool: State<'_, DbPool>,
) -> Result<Entity, String> {
    use crate::embeddings::{embed_entity, EntityEmbeddingRequest};
    use std::collections::HashMap;

    let repo = KnowledgeGraphRepository::new(pool.inner().clone());
    let existing = repo.get_entity_by_id(&entity_id)
        .map_err(|e| e.to_string())?
        .ok_or("Entity not found")?;

    let new_name = name.as_deref().unwrap_or(&existing.name);
    let new_attrs = attributes.as_ref().unwrap_or(&existing.attributes);

    // Auto-regenerate embedding when attributes or name changes
    let embedding = if name.is_some() || attributes.is_some() {
        let attrs_map: HashMap<String, serde_json::Value> = match new_attrs {
            serde_json::Value::Object(map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            _ => HashMap::new(),
        };
        let request = EntityEmbeddingRequest {
            entity_id: entity_id.clone(),
            name: new_name.to_string(),
            description: new_attrs.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            entity_type: existing.entity_type.to_string(),
            attributes: attrs_map,
        };
        embed_entity(&request).ok()
    } else {
        existing.embedding
    };

    repo.update_entity(&entity_id, Some(new_name), Some(new_attrs), embedding)
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

// ==================== 场景批注命令 ====================

#[command]
pub async fn create_scene_annotation(
    scene_id: String,
    story_id: String,
    content: String,
    annotation_type: String,
    pool: State<'_, DbPool>,
) -> Result<SceneAnnotation, String> {
    let repo = SceneAnnotationRepository::new(pool.inner().clone());
    repo.create_annotation(&scene_id, &story_id, &content, &annotation_type)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_scene_annotations(
    scene_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<SceneAnnotation>, String> {
    let repo = SceneAnnotationRepository::new(pool.inner().clone());
    repo.get_annotations_by_scene(&scene_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_story_unresolved_annotations(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<SceneAnnotation>, String> {
    let repo = SceneAnnotationRepository::new(pool.inner().clone());
    repo.get_unresolved_annotations_by_story(&story_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn update_scene_annotation(
    annotation_id: String,
    content: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = SceneAnnotationRepository::new(pool.inner().clone());
    repo.update_annotation(&annotation_id, &content)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn resolve_scene_annotation(
    annotation_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = SceneAnnotationRepository::new(pool.inner().clone());
    repo.resolve_annotation(&annotation_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn unresolve_scene_annotation(
    annotation_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = SceneAnnotationRepository::new(pool.inner().clone());
    repo.unresolve_annotation(&annotation_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn delete_scene_annotation(
    annotation_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = SceneAnnotationRepository::new(pool.inner().clone());
    repo.delete_annotation(&annotation_id)
        .map_err(|e| e.to_string())
}

// ==================== 文本内联批注命令 ====================

#[command]
pub async fn create_text_annotation(
    story_id: String,
    scene_id: Option<String>,
    chapter_id: Option<String>,
    content: String,
    annotation_type: String,
    from_pos: i32,
    to_pos: i32,
    pool: State<'_, DbPool>,
) -> Result<TextAnnotation, String> {
    let repo = TextAnnotationRepository::new(pool.inner().clone());
    repo.create_annotation(&story_id, scene_id.as_deref(), chapter_id.as_deref(), &content, &annotation_type, from_pos, to_pos)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_text_annotations_by_chapter(
    chapter_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<TextAnnotation>, String> {
    let repo = TextAnnotationRepository::new(pool.inner().clone());
    repo.get_annotations_by_chapter(&chapter_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_text_annotations_by_scene(
    scene_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<TextAnnotation>, String> {
    let repo = TextAnnotationRepository::new(pool.inner().clone());
    repo.get_annotations_by_scene(&scene_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn update_text_annotation(
    annotation_id: String,
    content: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = TextAnnotationRepository::new(pool.inner().clone());
    repo.update_annotation(&annotation_id, &content)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn resolve_text_annotation(
    annotation_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = TextAnnotationRepository::new(pool.inner().clone());
    repo.resolve_annotation(&annotation_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn unresolve_text_annotation(
    annotation_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = TextAnnotationRepository::new(pool.inner().clone());
    repo.unresolve_annotation(&annotation_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn delete_text_annotation(
    annotation_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = TextAnnotationRepository::new(pool.inner().clone());
    repo.delete_annotation(&annotation_id)
        .map_err(|e| e.to_string())
}

// ==================== 古典评点家命令 ====================

#[command]
pub async fn generate_paragraph_commentaries(
    story_id: String,
    story_title: String,
    genre: String,
    text: String,
    app_handle: AppHandle,
) -> Result<String, String> {
    use crate::agents::AgentContext;
    use crate::agents::commentator::CommentatorAgent;

    let context = AgentContext {
        story_id,
        story_title,
        genre,
        tone: "中性".to_string(),
        pacing: "正常".to_string(),
        chapter_number: 1,
        characters: vec![],
        previous_chapters: vec![],
    };

    let llm_service = LlmService::new(app_handle);
    let agent = CommentatorAgent::new(llm_service);
    let commentaries = agent.comment_on_text(&context, &text).await
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&commentaries).map_err(|e| e.to_string())
}

// ==================== 记忆压缩命令 ====================

#[command]
pub async fn compress_content(
    story_id: String,
    content: String,
    target_ratio: Option<f32>,
    app_handle: AppHandle,
) -> Result<crate::agents::AgentResult, String> {
    use crate::agents::service::{AgentService, AgentTask, AgentType};
    use crate::agents::commands::ExecuteAgentRequest;
    use std::collections::HashMap;

    let parameters = target_ratio.map(|r| {
        let mut map = HashMap::new();
        map.insert("target_ratio".to_string(), serde_json::json!(r));
        map
    });

    let request = ExecuteAgentRequest {
        agent_type: AgentType::MemoryCompressor,
        story_id: story_id.clone(),
        chapter_number: None,
        input: content.clone(),
        parameters: parameters.clone(),
    };

    let context = crate::agents::commands::build_agent_context(&app_handle, &request).await?;
    let task = AgentTask {
        id: uuid::Uuid::new_v4().to_string(),
        agent_type: AgentType::MemoryCompressor,
        context,
        input: content,
        parameters: parameters.unwrap_or_default(),
    };

    let service = AgentService::new(app_handle);
    service.execute_task(task).await
}

#[command]
pub async fn compress_scene(
    scene_id: String,
    target_ratio: Option<f32>,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<crate::agents::AgentResult, String> {
    use crate::agents::service::{AgentService, AgentTask, AgentType};
    use crate::agents::commands::ExecuteAgentRequest;
    use crate::db::repositories_v3::SceneRepository;
    use std::collections::HashMap;

    let scene_repo = SceneRepository::new(pool.inner().clone());
    let scene = scene_repo.get_by_id(&scene_id)
        .map_err(|e| e.to_string())?
        .ok_or("Scene not found")?;

    let content = scene.content.unwrap_or_default();
    if content.trim().is_empty() {
        return Err("Scene has no content to compress".to_string());
    }

    let parameters = target_ratio.map(|r| {
        let mut map = HashMap::new();
        map.insert("target_ratio".to_string(), serde_json::json!(r));
        map
    });

    let request = ExecuteAgentRequest {
        agent_type: AgentType::MemoryCompressor,
        story_id: scene.story_id.clone(),
        chapter_number: Some(scene.sequence_number.max(0) as u32),
        input: content.clone(),
        parameters: parameters.clone(),
    };

    let context = crate::agents::commands::build_agent_context(&app_handle, &request).await?;
    let task = AgentTask {
        id: uuid::Uuid::new_v4().to_string(),
        agent_type: AgentType::MemoryCompressor,
        context,
        input: content,
        parameters: parameters.unwrap_or_default(),
    };

    let service = AgentService::new(app_handle);
    service.execute_task(task).await
}

// ==================== 知识蒸馏命令 ====================

#[command]
pub async fn distill_story_knowledge(
    story_id: String,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<StorySummary, String> {
    use crate::agents::service::{AgentService, AgentTask, AgentType};
    use crate::agents::commands::ExecuteAgentRequest;
    use crate::db::repositories_v3::{KnowledgeGraphRepository, StorySummaryRepository};

    let kg_repo = KnowledgeGraphRepository::new(pool.inner().clone());
    let entities = kg_repo.get_entities_by_story(&story_id)
        .map_err(|e| e.to_string())?;
    let relations = kg_repo.get_relations_by_story(&story_id)
        .map_err(|e| e.to_string())?;

    use std::collections::HashMap;
    let entity_names: HashMap<&str, &str> = entities.iter()
        .map(|e| (e.id.as_str(), e.name.as_str()))
        .collect();

    let kg_input = serde_json::json!({
        "entities": entities.iter().map(|e| {
            serde_json::json!({
                "name": e.name,
                "type": e.entity_type,
                "attributes": e.attributes,
            })
        }).collect::<Vec<_>>(),
        "relations": relations.iter().map(|r| {
            serde_json::json!({
                "source": entity_names.get(r.source_id.as_str()).unwrap_or(&r.source_id.as_str()),
                "target": entity_names.get(r.target_id.as_str()).unwrap_or(&r.target_id.as_str()),
                "type": r.relation_type,
                "strength": r.strength,
            })
        }).collect::<Vec<_>>(),
    });

    let request = ExecuteAgentRequest {
        agent_type: AgentType::KnowledgeDistiller,
        story_id: story_id.clone(),
        chapter_number: None,
        input: kg_input.to_string(),
        parameters: None,
    };

    let context = crate::agents::commands::build_agent_context(&app_handle, &request).await?;
    let task = AgentTask {
        id: uuid::Uuid::new_v4().to_string(),
        agent_type: AgentType::KnowledgeDistiller,
        context,
        input: kg_input.to_string(),
        parameters: std::collections::HashMap::new(),
    };

    let service = AgentService::new(app_handle);
    let result = service.execute_task(task).await?;

    let summary_repo = StorySummaryRepository::new(pool.inner().clone());
    // 如果已存在同类型摘要，则更新；否则创建
    let summary = match summary_repo.get_summary_by_type(&story_id, "knowledge_distillation") {
        Ok(Some(existing)) => {
            summary_repo.update_summary(&existing.id, &result.content)
                .map_err(|e| e.to_string())?;
            StorySummary {
                content: result.content,
                updated_at: chrono::Local::now(),
                ..existing
            }
        }
        _ => {
            summary_repo.create_summary(&story_id, "knowledge_distillation", &result.content)
                .map_err(|e| e.to_string())?
        }
    };

    Ok(summary)
}

#[command]
pub async fn get_story_summaries(
    story_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<StorySummary>, String> {
    let repo = StorySummaryRepository::new(pool.inner().clone());
    repo.get_summaries_by_story(&story_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn update_story_summary(
    summary_id: String,
    content: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = StorySummaryRepository::new(pool.inner().clone());
    repo.update_summary(&summary_id, &content)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn delete_story_summary(
    summary_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    let repo = StorySummaryRepository::new(pool.inner().clone());
    repo.delete_summary(&summary_id)
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

// ==================== 小说创建向导命令 ====================

#[command]
pub async fn generate_world_building_options(
    user_input: String,
    app_handle: AppHandle,
) -> Result<Vec<WorldBuildingOption>, String> {
    let llm_service = LlmService::new(app_handle);
    let agent = NovelCreationAgent::new(llm_service);
    let options = GenerationOptions::default();
    
    agent.generate_world_building_options(&user_input, &options)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn generate_character_profiles(
    world_building: WorldBuildingOption,
    app_handle: AppHandle,
) -> Result<Vec<Vec<CharacterProfileOption>>, String> {
    let llm_service = LlmService::new(app_handle);
    let agent = NovelCreationAgent::new(llm_service);
    let options = GenerationOptions::default();
    
    agent.generate_character_profiles(&world_building, &options)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn generate_writing_styles(
    genre: String,
    world_building: WorldBuildingOption,
    app_handle: AppHandle,
) -> Result<Vec<WritingStyleOption>, String> {
    let llm_service = LlmService::new(app_handle);
    let agent = NovelCreationAgent::new(llm_service);
    let options = GenerationOptions::default();
    
    agent.generate_writing_styles(&genre, &world_building, &options)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn generate_first_scene(
    world_building: WorldBuildingOption,
    characters: Vec<CharacterProfileOption>,
    writing_style: WritingStyleOption,
    app_handle: AppHandle,
) -> Result<SceneProposal, String> {
    let llm_service = LlmService::new(app_handle);
    let agent = NovelCreationAgent::new(llm_service);
    
    agent.generate_first_scene(&world_building, &characters, &writing_style)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardCreationResult {
    pub story: Story,
    pub world_building: WorldBuilding,
    pub writing_style: WritingStyle,
    pub first_scene: Scene,
    pub characters: Vec<Character>,
    pub ingested_entities: usize,
    pub ingested_relations: usize,
}

#[command]
pub async fn create_story_with_wizard(
    title: String,
    description: Option<String>,
    genre: Option<String>,
    world_building: WorldBuildingOption,
    characters: Vec<CharacterProfileOption>,
    writing_style: WritingStyleOption,
    first_scene: SceneProposal,
    pool: State<'_, DbPool>,
    app_handle: AppHandle,
) -> Result<WizardCreationResult, String> {
    // 1. 创建故事
    let story_repo = StoryRepository::new(pool.inner().clone());
    let story = story_repo.create(CreateStoryRequest { title, description, genre })
        .map_err(|e| e.to_string())?;
    let story_id = story.id.clone();
    
    // 2. 创建世界观
    let wb_repo = WorldBuildingRepository::new(pool.inner().clone());
    let wb = wb_repo.create(&story_id, &world_building.concept)
        .map_err(|e| e.to_string())?;
    
    wb_repo.update(&wb.id, Some(&world_building.concept), 
        Some(&world_building.rules),
        Some(&world_building.history),
        Some(&world_building.cultures)
    ).map_err(|e| e.to_string())?;
    
    // 3. 创建角色
    let char_repo = CharacterRepository::new(pool.inner().clone());
    let mut created_chars = Vec::new();
    for char_opt in &characters {
        let background = format!("{}", char_opt.background);
        let char = char_repo.create(CreateCharacterRequest {
            story_id: story_id.clone(),
            name: char_opt.name.clone(),
            background: Some(background),
        }).map_err(|e| e.to_string())?;
        
        char_repo.update(&char.id, None, None, Some(char_opt.personality.clone()), Some(char_opt.goals.clone()))
            .map_err(|e| e.to_string())?;
        
        created_chars.push(char);
    }
    
    // 4. 创建文字风格
    let ws_repo = WritingStyleRepository::new(pool.inner().clone());
    let ws = ws_repo.create(&story_id, Some(&writing_style.name))
        .map_err(|e| e.to_string())?;
    
    let ws_update = WritingStyleUpdate {
        name: Some(writing_style.name.clone()),
        description: Some(writing_style.description.clone()),
        tone: Some(writing_style.tone.clone()),
        pacing: Some(writing_style.pacing.clone()),
        vocabulary_level: Some(writing_style.vocabulary_level.clone()),
        sentence_structure: Some(writing_style.sentence_structure.clone()),
        custom_rules: Some(vec![]),
    };
    ws_repo.update(&ws.id, &ws_update).map_err(|e| e.to_string())?;
    
    // 5. 创建首个场景
    let scene_repo = SceneRepository::new(pool.inner().clone());
    let scene = scene_repo.create(&story_id, 1, Some(&first_scene.title))
        .map_err(|e| e.to_string())?;
    
    let conflict_type = first_scene.conflict_type.parse().ok();
    let char_ids: Vec<String> = created_chars.iter().map(|c| c.id.clone()).collect();
    let scene_update = SceneUpdate {
        title: Some(first_scene.title.clone()),
        dramatic_goal: Some(first_scene.dramatic_goal.clone()),
        external_pressure: Some(first_scene.external_pressure.clone()),
        conflict_type,
        characters_present: Some(char_ids),
        character_conflicts: Some(vec![]),
        content: Some(first_scene.content.clone()),
        setting_location: Some(first_scene.setting_location.clone()),
        setting_time: Some(first_scene.setting_time.clone()),
        setting_atmosphere: Some(first_scene.setting_atmosphere.clone()),
        previous_scene_id: None,
        next_scene_id: None,
        confidence_score: Some(0.8),
    };
    scene_repo.update(&scene.id, &scene_update).map_err(|e| e.to_string())?;
    
    // 6. 自动 Ingest
    let ingest_text = format!(
        "世界观：{}\n\n历史背景：{}\n\n角色设定：\n{}\n\n文字风格：{}\n\n首个场景：{}\n\n{}",
        world_building.concept,
        &world_building.history,
        characters.iter().map(|c| format!("- {}：{}，目标：{}", c.name, c.personality, c.goals)).collect::<Vec<_>>().join("\n"),
        writing_style.name,
        first_scene.title,
        first_scene.content
    );
    
    let llm_service = LlmService::new(app_handle);
    let pipeline = IngestPipeline::new(llm_service);
    let ingest_content = IngestContent {
        text: ingest_text,
        source: format!("novel_creation_wizard:{}" , story_id),
        story_id: story_id.clone(),
        scene_id: Some(scene.id.clone()),
    };
    
    let ingest_result = pipeline.ingest(&ingest_content).await
        .map_err(|e| e.to_string())?;
    
    // 保存 Ingest 结果到知识图谱
    let kg_repo = KnowledgeGraphRepository::new(pool.inner().clone());
    let mut saved_entities = 0usize;
    let mut saved_relations = 0usize;
    
    for entity in &ingest_result.entities {
        kg_repo.create_entity(&story_id, &entity.name, &entity.entity_type.to_string(), &entity.attributes, entity.embedding.clone())
            .map_err(|e| e.to_string())?;
        saved_entities += 1;
    }
    
    // 为关系建立映射（按实体名称查找ID）
    let entity_name_to_id: std::collections::HashMap<String, String> = ingest_result.entities
        .iter()
        .map(|e| (e.name.clone(), e.id.clone()))
        .collect();
    
    for relation in &ingest_result.relations {
        if let (Some(source_id), Some(target_id)) = (entity_name_to_id.get(&relation.source_id), entity_name_to_id.get(&relation.target_id)) {
            kg_repo.create_relation(&story_id, source_id, target_id, &relation.relation_type.to_string(), relation.strength)
                .map_err(|e| e.to_string())?;
            saved_relations += 1;
        }
    }
    
    // 重新获取完整的世界观（因为 update 返回的是 usize）
    let final_wb = wb_repo.get_by_story(&story_id)
        .map_err(|e| e.to_string())?
        .ok_or("World building not found")?;
    
    let final_ws = ws_repo.get_by_story(&story_id)
        .map_err(|e| e.to_string())?
        .ok_or("Writing style not found")?;
    
    Ok(WizardCreationResult {
        story,
        world_building: final_wb,
        writing_style: final_ws,
        first_scene: scene_repo.get_by_id(&scene.id).map_err(|e| e.to_string())?.ok_or("Scene not found")?,
        characters: created_chars,
        ingested_entities: saved_entities,
        ingested_relations: saved_relations,
    })
}

// ==================== 场景版本命令 ====================

use crate::db::models_v3::{SceneVersion, CreatorType};
use crate::db::repositories_v3::SceneVersionRepository;
use crate::versions::service::{SceneVersionService, VersionDiff, VersionStats};

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

/// 为指定场景创建版本快照，并自动生成 ChangeTrack diff
fn create_version_snapshot(
    pool: &DbPool,
    scene_id: &str,
    change_summary: &str,
    created_by: &str,
) -> Result<Option<SceneVersion>, String> {
    use crate::db::repositories_v3::ChangeTrackRepository;

    let scene_repo = crate::db::repositories_v3::SceneRepository::new(pool.clone());
    let version_repo = SceneVersionRepository::new(pool.clone());
    let track_repo = ChangeTrackRepository::new(pool.clone());
    
    let scene = match scene_repo.get_by_id(scene_id) {
        Ok(Some(s)) => s,
        Ok(None) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    
    // 获取上一版本内容用于 diff
    let prev_content = version_repo.get_versions(scene_id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .next()
        .and_then(|v| v.content);
    
    let creator = match created_by {
        "user" => CreatorType::User,
        "ai" => CreatorType::Ai,
        _ => CreatorType::System,
    };
    
    let version = version_repo.create_version(&scene, change_summary, creator, None, None)
        .map_err(|e| e.to_string())?;
    
    // 基于 diff 生成 ChangeTrack
    let current_content = scene.content.as_deref().unwrap_or("");
    if let Some(old) = prev_content {
        let tracks = diff_to_change_tracks(scene_id, created_by, &old, current_content);
        for mut track in tracks {
            track.version_id = Some(version.id.clone());
            let _ = track_repo.create(&track);
        }
    }
    
    Ok(Some(version))
}

#[command]
pub async fn create_scene_version(
    scene_id: String,
    change_summary: String,
    created_by: String,
    confidence_score: Option<f32>,
    pool: State<'_, DbPool>,
) -> Result<SceneVersion, String> {
    use crate::db::repositories_v3::ChangeTrackRepository;

    let scene_repo = crate::db::repositories_v3::SceneRepository::new(pool.inner().clone());
    let version_repo = SceneVersionRepository::new(pool.inner().clone());
    let track_repo = ChangeTrackRepository::new(pool.inner().clone());
    
    let scene = scene_repo.get_by_id(&scene_id)
        .map_err(|e| e.to_string())?
        .ok_or("Scene not found")?;
    
    // 获取上一版本内容用于 diff
    let prev_content = version_repo.get_versions(&scene_id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .next()
        .and_then(|v| v.content);
    
    let creator = match created_by.as_str() {
        "user" => CreatorType::User,
        "ai" => CreatorType::Ai,
        _ => CreatorType::System,
    };
    
    let version = version_repo.create_version(&scene, &change_summary, creator, None, confidence_score)
        .map_err(|e| e.to_string())?;
    
    // 基于 diff 生成 ChangeTrack
    let current_content = scene.content.as_deref().unwrap_or("");
    if let Some(old) = prev_content {
        let tracks = diff_to_change_tracks(&scene_id, &created_by, &old, current_content);
        for mut track in tracks {
            track.version_id = Some(version.id.clone());
            let _ = track_repo.create(&track);
        }
    }
    
    Ok(version)
}

/// 将两段文本的差异转换为 ChangeTrack 列表（简单字符级 diff）
fn diff_to_change_tracks(
    scene_id: &str,
    author_id: &str,
    old: &str,
    new: &str,
) -> Vec<crate::db::models_v3::ChangeTrack> {
    use crate::db::models_v3::{ChangeTrack, ChangeType};
    
    if old == new {
        return vec![];
    }
    
    // 找公共前缀
    let mut prefix = 0;
    let old_chars: Vec<char> = old.chars().collect();
    let new_chars: Vec<char> = new.chars().collect();
    while prefix < old_chars.len() && prefix < new_chars.len() && old_chars[prefix] == new_chars[prefix] {
        prefix += 1;
    }
    
    // 找公共后缀
    let mut suffix = 0;
    while suffix < old_chars.len() - prefix && suffix < new_chars.len() - prefix
        && old_chars[old_chars.len() - 1 - suffix] == new_chars[new_chars.len() - 1 - suffix]
    {
        suffix += 1;
    }
    
    let old_mid_start = prefix;
    let old_mid_end = old_chars.len() - suffix;
    let new_mid_start = prefix;
    let new_mid_end = new_chars.len() - suffix;
    
    let mut tracks = Vec::new();
    
    // 删除的部分
    if old_mid_start < old_mid_end {
        let deleted: String = old_chars[old_mid_start..old_mid_end].iter().collect();
        tracks.push(ChangeTrack::new(
            Some(scene_id.to_string()),
            None,
            author_id.to_string(),
            ChangeType::Delete,
            old_mid_start as i32,
            old_mid_end as i32,
            Some(deleted),
        ));
    }
    
    // 插入的部分
    if new_mid_start < new_mid_end {
        let inserted: String = new_chars[new_mid_start..new_mid_end].iter().collect();
        tracks.push(ChangeTrack::new(
            Some(scene_id.to_string()),
            None,
            author_id.to_string(),
            ChangeType::Insert,
            new_mid_start as i32,
            new_mid_end as i32,
            Some(inserted),
        ));
    }
    
    tracks
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
pub async fn get_version_change_tracks(
    version_id: String,
    pool: State<'_, DbPool>,
) -> Result<Vec<crate::db::models_v3::ChangeTrack>, String> {
    let repo = crate::db::repositories_v3::ChangeTrackRepository::new(pool.inner().clone());
    repo.get_by_version(&version_id)
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


// ==================== 变更追踪命令 (修订模式) ====================

#[command]
pub async fn track_change(
    scene_id: Option<String>,
    chapter_id: Option<String>,
    change_type: String,
    from_pos: i32,
    to_pos: i32,
    content: Option<String>,
    author_id: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<crate::db::models_v3::ChangeTrack, String> {
    use crate::db::models_v3::{ChangeTrack, ChangeType};
    use crate::db::repositories_v3::ChangeTrackRepository;

    let ct = match change_type.as_str() {
        "Delete" => ChangeType::Delete,
        "Format" => ChangeType::Format,
        _ => ChangeType::Insert,
    };

    let track = ChangeTrack::new(
        scene_id,
        chapter_id,
        author_id.unwrap_or_else(|| "user".to_string()),
        ct,
        from_pos,
        to_pos,
        content,
    );

    let repo = ChangeTrackRepository::new(pool.inner().clone());
    repo.create(&track)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn accept_change(
    change_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    use crate::db::models_v3::ChangeStatus;
    use crate::db::repositories_v3::ChangeTrackRepository;

    let repo = ChangeTrackRepository::new(pool.inner().clone());
    let result = repo.update_status(&change_id, ChangeStatus::Accepted)
        .map_err(|e| e.to_string())?;
    
    // 自动创建版本快照
    if let Ok(Some(track)) = repo.get_by_id(&change_id) {
        if let Some(scene_id) = track.scene_id {
            let _ = create_version_snapshot(pool.inner(), &scene_id, "接受变更", "system");
        }
    }
    
    Ok(result)
}

#[command]
pub async fn reject_change(
    change_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    use crate::db::models_v3::ChangeStatus;
    use crate::db::repositories_v3::ChangeTrackRepository;

    let repo = ChangeTrackRepository::new(pool.inner().clone());
    let result = repo.update_status(&change_id, ChangeStatus::Rejected)
        .map_err(|e| e.to_string())?;
    
    // 自动创建版本快照
    if let Ok(Some(track)) = repo.get_by_id(&change_id) {
        if let Some(scene_id) = track.scene_id {
            let _ = create_version_snapshot(pool.inner(), &scene_id, "拒绝变更", "system");
        }
    }
    
    Ok(result)
}

#[command]
pub async fn get_pending_changes(
    scene_id: Option<String>,
    chapter_id: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<Vec<crate::db::models_v3::ChangeTrack>, String> {
    use crate::db::repositories_v3::ChangeTrackRepository;

    let repo = ChangeTrackRepository::new(pool.inner().clone());
    if let Some(sid) = scene_id {
        repo.get_pending_by_scene(&sid)
            .map_err(|e| e.to_string())
    } else if let Some(cid) = chapter_id {
        repo.get_pending_by_chapter(&cid)
            .map_err(|e| e.to_string())
    } else {
        Err("Either scene_id or chapter_id must be provided".to_string())
    }
}

#[command]
pub async fn accept_all_changes(
    scene_id: Option<String>,
    chapter_id: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    use crate::db::repositories_v3::ChangeTrackRepository;

    let repo = ChangeTrackRepository::new(pool.inner().clone());
    let result = if let Some(sid) = scene_id.clone() {
        repo.accept_all_by_scene(&sid)
            .map_err(|e| e.to_string())?
    } else if let Some(cid) = chapter_id {
        repo.accept_all_by_chapter(&cid)
            .map_err(|e| e.to_string())?
    } else {
        return Err("Either scene_id or chapter_id must be provided".to_string());
    };
    
    // 自动创建版本快照（仅场景级变更）
    if let Some(sid) = scene_id {
        let _ = create_version_snapshot(pool.inner(), &sid, "全部接受变更", "system");
    }
    
    Ok(result)
}

#[command]
pub async fn reject_all_changes(
    scene_id: Option<String>,
    chapter_id: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    use crate::db::repositories_v3::ChangeTrackRepository;

    let repo = ChangeTrackRepository::new(pool.inner().clone());
    let result = if let Some(sid) = scene_id.clone() {
        repo.reject_all_by_scene(&sid)
            .map_err(|e| e.to_string())?
    } else if let Some(cid) = chapter_id {
        repo.reject_all_by_chapter(&cid)
            .map_err(|e| e.to_string())?
    } else {
        return Err("Either scene_id or chapter_id must be provided".to_string());
    };
    
    // 自动创建版本快照（仅场景级变更）
    if let Some(sid) = scene_id {
        let _ = create_version_snapshot(pool.inner(), &sid, "全部拒绝变更", "system");
    }
    
    Ok(result)
}


// ==================== 评论线程命令 (修订模式) ====================

#[command]
pub async fn create_comment_thread(
    version_id: Option<String>,
    anchor_type: String,
    scene_id: Option<String>,
    chapter_id: Option<String>,
    from_pos: Option<i32>,
    to_pos: Option<i32>,
    selected_text: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<crate::db::models_v3::CommentThread, String> {
    use crate::db::models_v3::{CommentThread, AnchorType};
    use crate::db::repositories_v3::CommentThreadRepository;

    let at = match anchor_type.as_str() {
        "SceneLevel" => AnchorType::SceneLevel,
        _ => AnchorType::TextRange,
    };

    let thread = CommentThread::new(
        version_id,
        at,
        scene_id,
        chapter_id,
        from_pos,
        to_pos,
        selected_text,
    );

    let repo = CommentThreadRepository::new(pool.inner().clone());
    repo.create_thread(&thread)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn add_comment_message(
    thread_id: String,
    content: String,
    author_id: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<crate::db::models_v3::CommentMessage, String> {
    use crate::db::models_v3::CommentMessage;
    use crate::db::repositories_v3::CommentThreadRepository;
    use chrono::Local;
    use uuid::Uuid;

    let message = CommentMessage {
        id: Uuid::new_v4().to_string(),
        thread_id,
        author_id: author_id.unwrap_or_else(|| "user".to_string()),
        author_name: None,
        content,
        created_at: Local::now(),
    };

    let repo = CommentThreadRepository::new(pool.inner().clone());
    repo.add_message(&message)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_comment_threads(
    scene_id: Option<String>,
    chapter_id: Option<String>,
    pool: State<'_, DbPool>,
) -> Result<Vec<crate::db::models_v3::CommentThreadWithMessages>, String> {
    use crate::db::repositories_v3::CommentThreadRepository;

    let repo = CommentThreadRepository::new(pool.inner().clone());
    if let Some(sid) = scene_id {
        repo.get_threads_by_scene(&sid)
            .map_err(|e| e.to_string())
    } else if let Some(cid) = chapter_id {
        repo.get_threads_by_chapter(&cid)
            .map_err(|e| e.to_string())
    } else {
        Err("Either scene_id or chapter_id must be provided".to_string())
    }
}

#[command]
pub async fn resolve_comment_thread(
    thread_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    use crate::db::repositories_v3::CommentThreadRepository;

    let repo = CommentThreadRepository::new(pool.inner().clone());
    repo.resolve_thread(&thread_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn reopen_comment_thread(
    thread_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    use crate::db::repositories_v3::CommentThreadRepository;

    let repo = CommentThreadRepository::new(pool.inner().clone());
    repo.reopen_thread(&thread_id)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn delete_comment_thread(
    thread_id: String,
    pool: State<'_, DbPool>,
) -> Result<usize, String> {
    use crate::db::repositories_v3::CommentThreadRepository;

    let repo = CommentThreadRepository::new(pool.inner().clone());
    repo.delete_thread(&thread_id)
        .map_err(|e| e.to_string())
}
