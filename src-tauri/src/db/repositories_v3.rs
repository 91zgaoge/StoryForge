//! V3 架构 Repository 层

use super::{DbPool, Scene, ConflictType, CharacterConflict, WorldBuilding, WorldRule, Culture};
use super::{Setting, LocationType, SensoryDetails, WritingStyle, StudioConfig};
use super::{LlmStudioConfig, UiStudioConfig, AgentBotConfig, Entity, Relation};
use chrono::Local;
use rusqlite::{params, OptionalExtension};
use serde::{Serialize, Deserialize};
use serde_json;
use uuid::Uuid;

// ==================== Scene Repository ====================

pub struct SceneRepository {
    pool: DbPool,
}

impl SceneRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, story_id: &str, sequence_number: i32, title: Option<&str>) -> Result<Scene, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Local::now();
        
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "INSERT INTO scenes (id, story_id, sequence_number, title, characters_present, character_conflicts, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![&id, story_id, sequence_number, title, "[]", "[]", now.to_rfc3339(), now.to_rfc3339()],
        )?;
        
        Ok(Scene {
            id,
            story_id: story_id.to_string(),
            sequence_number,
            title: title.map(|s| s.to_string()),
            dramatic_goal: None,
            external_pressure: None,
            conflict_type: None,
            characters_present: vec![],
            character_conflicts: vec![],
            content: None,
            setting_location: None,
            setting_time: None,
            setting_atmosphere: None,
            previous_scene_id: None,
            next_scene_id: None,
            model_used: None,
            cost: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_by_story(&self, story_id: &str) -> Result<Vec<Scene>, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, sequence_number, title, dramatic_goal, external_pressure, conflict_type,
                    characters_present, character_conflicts, setting_location, setting_time, setting_atmosphere,
                    content, previous_scene_id, next_scene_id, model_used, cost, created_at, updated_at 
             FROM scenes WHERE story_id = ?1 ORDER BY sequence_number"
        )?;

        let scenes = stmt.query_map([story_id], |row| {
            let conflict_type_str: Option<String> = row.get(5)?;
            let conflict_type = conflict_type_str.and_then(|s| s.parse().ok());
            
            let chars_json: String = row.get(7)?;
            let characters_present: Vec<String> = serde_json::from_str(&chars_json).unwrap_or_default();
            
            let conflicts_json: String = row.get(8)?;
            let character_conflicts: Vec<CharacterConflict> = serde_json::from_str(&conflicts_json).unwrap_or_default();
            
            let created_str: String = row.get(17)?;
            let updated_str: String = row.get(18)?;
            
            Ok(Scene {
                id: row.get(0)?,
                story_id: row.get(1)?,
                sequence_number: row.get(2)?,
                title: row.get(3)?,
                dramatic_goal: row.get(4)?,
                external_pressure: row.get(5)?,
                conflict_type,
                characters_present,
                character_conflicts,
                setting_location: row.get(9)?,
                setting_time: row.get(10)?,
                setting_atmosphere: row.get(11)?,
                content: row.get(12)?,
                previous_scene_id: row.get(13)?,
                next_scene_id: row.get(14)?,
                model_used: row.get(15)?,
                cost: row.get(16)?,
                created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(scenes)
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Scene>, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, sequence_number, title, dramatic_goal, external_pressure, conflict_type,
                    characters_present, character_conflicts, setting_location, setting_time, setting_atmosphere,
                    content, previous_scene_id, next_scene_id, model_used, cost, created_at, updated_at 
             FROM scenes WHERE id = ?1"
        )?;

        let scene = stmt.query_row([id], |row| {
            let conflict_type_str: Option<String> = row.get(5)?;
            let conflict_type = conflict_type_str.and_then(|s| s.parse().ok());
            
            let chars_json: String = row.get(7)?;
            let characters_present: Vec<String> = serde_json::from_str(&chars_json).unwrap_or_default();
            
            let conflicts_json: String = row.get(8)?;
            let character_conflicts: Vec<CharacterConflict> = serde_json::from_str(&conflicts_json).unwrap_or_default();
            
            let created_str: String = row.get(17)?;
            let updated_str: String = row.get(18)?;
            
            Ok(Scene {
                id: row.get(0)?,
                story_id: row.get(1)?,
                sequence_number: row.get(2)?,
                title: row.get(3)?,
                dramatic_goal: row.get(4)?,
                external_pressure: row.get(5)?,
                conflict_type,
                characters_present,
                character_conflicts,
                setting_location: row.get(9)?,
                setting_time: row.get(10)?,
                setting_atmosphere: row.get(11)?,
                content: row.get(12)?,
                previous_scene_id: row.get(13)?,
                next_scene_id: row.get(14)?,
                model_used: row.get(15)?,
                cost: row.get(16)?,
                created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
            })
        }).optional()?;

        Ok(scene)
    }

    pub fn update(&self, id: &str, updates: &SceneUpdate) -> Result<usize, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = Local::now().to_rfc3339();
        
        let count = conn.execute(
            "UPDATE scenes SET 
                title = COALESCE(?2, title),
                dramatic_goal = COALESCE(?3, dramatic_goal),
                external_pressure = COALESCE(?4, external_pressure),
                conflict_type = COALESCE(?5, conflict_type),
                characters_present = COALESCE(?6, characters_present),
                character_conflicts = COALESCE(?7, character_conflicts),
                content = COALESCE(?8, content),
                setting_location = COALESCE(?9, setting_location),
                setting_time = COALESCE(?10, setting_time),
                setting_atmosphere = COALESCE(?11, setting_atmosphere),
                previous_scene_id = COALESCE(?12, previous_scene_id),
                next_scene_id = COALESCE(?13, next_scene_id),
                updated_at = ?14
             WHERE id = ?1",
            params![
                id,
                updates.title,
                updates.dramatic_goal,
                updates.external_pressure,
                updates.conflict_type.as_ref().map(|c| c.to_string()),
                updates.characters_present.as_ref().map(|c| serde_json::to_string(c).unwrap()),
                updates.character_conflicts.as_ref().map(|c| serde_json::to_string(c).unwrap()),
                updates.content,
                updates.setting_location,
                updates.setting_time,
                updates.setting_atmosphere,
                updates.previous_scene_id,
                updates.next_scene_id,
                now
            ],
        )?;
        Ok(count)
    }

    pub fn delete(&self, id: &str) -> Result<usize, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let count = conn.execute("DELETE FROM scenes WHERE id = ?1", [id])?;
        Ok(count)
    }

    pub fn update_sequence(&self, id: &str, new_sequence: i32) -> Result<usize, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = Local::now().to_rfc3339();
        let count = conn.execute(
            "UPDATE scenes SET sequence_number = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, new_sequence, now],
        )?;
        Ok(count)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SceneUpdate {
    pub title: Option<String>,
    pub dramatic_goal: Option<String>,
    pub external_pressure: Option<String>,
    pub conflict_type: Option<ConflictType>,
    pub characters_present: Option<Vec<String>>,
    pub character_conflicts: Option<Vec<CharacterConflict>>,
    pub content: Option<String>,
    pub setting_location: Option<String>,
    pub setting_time: Option<String>,
    pub setting_atmosphere: Option<String>,
    pub previous_scene_id: Option<String>,
    pub next_scene_id: Option<String>,
}

// ==================== WorldBuilding Repository ====================

pub struct WorldBuildingRepository {
    pool: DbPool,
}

impl WorldBuildingRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, story_id: &str, concept: &str) -> Result<WorldBuilding, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Local::now();
        
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "INSERT INTO world_buildings (id, story_id, concept, rules, history, cultures, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, ?7)",
            params![&id, story_id, concept, "[]", "[]", now.to_rfc3339(), now.to_rfc3339()],
        )?;
        
        Ok(WorldBuilding {
            id,
            story_id: story_id.to_string(),
            concept: concept.to_string(),
            rules: vec![],
            history: None,
            cultures: vec![],
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_by_story(&self, story_id: &str) -> Result<Option<WorldBuilding>, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, concept, rules, history, cultures, created_at, updated_at 
             FROM world_buildings WHERE story_id = ?1"
        )?;

        let result = stmt.query_row([story_id], |row| {
            let rules_json: String = row.get(3)?;
            let rules: Vec<WorldRule> = serde_json::from_str(&rules_json).unwrap_or_default();
            
            let cultures_json: String = row.get(5)?;
            let cultures: Vec<Culture> = serde_json::from_str(&cultures_json).unwrap_or_default();
            
            let created_str: String = row.get(6)?;
            let updated_str: String = row.get(7)?;
            
            Ok(WorldBuilding {
                id: row.get(0)?,
                story_id: row.get(1)?,
                concept: row.get(2)?,
                rules,
                history: row.get(4)?,
                cultures,
                created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
            })
        }).optional()?;

        Ok(result)
    }

    pub fn update(&self, id: &str, concept: Option<&str>, rules: Option<&[WorldRule]>, 
                  history: Option<&str>, cultures: Option<&[Culture]>) -> Result<usize, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = Local::now().to_rfc3339();
        
        let count = conn.execute(
            "UPDATE world_buildings SET 
                concept = COALESCE(?2, concept),
                rules = COALESCE(?3, rules),
                history = COALESCE(?4, history),
                cultures = COALESCE(?5, cultures),
                updated_at = ?6
             WHERE id = ?1",
            params![
                id,
                concept,
                rules.map(|r| serde_json::to_string(r).unwrap()),
                history,
                cultures.map(|c| serde_json::to_string(c).unwrap()),
                now
            ],
        )?;
        Ok(count)
    }
}

// ==================== WritingStyle Repository ====================

pub struct WritingStyleRepository {
    pool: DbPool,
}

impl WritingStyleRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, story_id: &str) -> Result<WritingStyle, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Local::now();
        
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "INSERT INTO writing_styles (id, story_id, custom_rules, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&id, story_id, "[]", now.to_rfc3339(), now.to_rfc3339()],
        )?;
        
        Ok(WritingStyle {
            id,
            story_id: story_id.to_string(),
            name: None,
            description: None,
            tone: None,
            pacing: None,
            vocabulary_level: None,
            sentence_structure: None,
            custom_rules: vec![],
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_by_story(&self, story_id: &str) -> Result<Option<WritingStyle>, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, name, description, tone, pacing, vocabulary_level, 
                    sentence_structure, custom_rules, created_at, updated_at 
             FROM writing_styles WHERE story_id = ?1"
        )?;

        let result = stmt.query_row([story_id], |row| {
            let rules_json: String = row.get(8)?;
            let custom_rules: Vec<String> = serde_json::from_str(&rules_json).unwrap_or_default();
            
            let created_str: String = row.get(9)?;
            let updated_str: String = row.get(10)?;
            
            Ok(WritingStyle {
                id: row.get(0)?,
                story_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                tone: row.get(4)?,
                pacing: row.get(5)?,
                vocabulary_level: row.get(6)?,
                sentence_structure: row.get(7)?,
                custom_rules,
                created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
            })
        }).optional()?;

        Ok(result)
    }

    pub fn update(&self, id: &str, updates: &WritingStyleUpdate) -> Result<usize, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = Local::now().to_rfc3339();
        
        let count = conn.execute(
            "UPDATE writing_styles SET 
                name = COALESCE(?2, name),
                description = COALESCE(?3, description),
                tone = COALESCE(?4, tone),
                pacing = COALESCE(?5, pacing),
                vocabulary_level = COALESCE(?6, vocabulary_level),
                sentence_structure = COALESCE(?7, sentence_structure),
                custom_rules = COALESCE(?8, custom_rules),
                updated_at = ?9
             WHERE id = ?1",
            params![
                id,
                updates.name,
                updates.description,
                updates.tone,
                updates.pacing,
                updates.vocabulary_level,
                updates.sentence_structure,
                updates.custom_rules.as_ref().map(|r| serde_json::to_string(r).unwrap()),
                now
            ],
        )?;
        Ok(count)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WritingStyleUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tone: Option<String>,
    pub pacing: Option<String>,
    pub vocabulary_level: Option<String>,
    pub sentence_structure: Option<String>,
    pub custom_rules: Option<Vec<String>>,
}

// ==================== StudioConfig Repository ====================

pub struct StudioConfigRepository {
    pool: DbPool,
}

impl StudioConfigRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, story_id: &str) -> Result<StudioConfig, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Local::now();
        
        let llm_config = LlmStudioConfig::default();
        let ui_config = UiStudioConfig::default();
        
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "INSERT INTO studio_configs (id, story_id, llm_config, ui_config, agent_bots, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &id, 
                story_id, 
                serde_json::to_string(&llm_config).unwrap(),
                serde_json::to_string(&ui_config).unwrap(),
                "[]",
                now.to_rfc3339(), 
                now.to_rfc3339()
            ],
        )?;
        
        Ok(StudioConfig {
            id,
            story_id: story_id.to_string(),
            pen_name: None,
            llm_config,
            ui_config,
            agent_bots: vec![],
            frontstage_theme: None,
            backstage_theme: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_by_story(&self, story_id: &str) -> Result<Option<StudioConfig>, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, pen_name, llm_config, ui_config, agent_bots, 
                    frontstage_theme, backstage_theme, created_at, updated_at 
             FROM studio_configs WHERE story_id = ?1"
        )?;

        let result = stmt.query_row([story_id], |row| {
            let llm_json: String = row.get(3)?;
            let llm_config: LlmStudioConfig = serde_json::from_str(&llm_json).unwrap_or_default();
            
            let ui_json: String = row.get(4)?;
            let ui_config: UiStudioConfig = serde_json::from_str(&ui_json).unwrap_or_default();
            
            let bots_json: String = row.get(5)?;
            let agent_bots: Vec<AgentBotConfig> = serde_json::from_str(&bots_json).unwrap_or_default();
            
            let created_str: String = row.get(8)?;
            let updated_str: String = row.get(9)?;
            
            Ok(StudioConfig {
                id: row.get(0)?,
                story_id: row.get(1)?,
                pen_name: row.get(2)?,
                llm_config,
                ui_config,
                agent_bots,
                frontstage_theme: row.get(6)?,
                backstage_theme: row.get(7)?,
                created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
            })
        }).optional()?;

        Ok(result)
    }

    pub fn update(&self, id: &str, pen_name: Option<&str>, llm_config: Option<&LlmStudioConfig>,
                  ui_config: Option<&UiStudioConfig>, agent_bots: Option<&[AgentBotConfig]>) -> Result<usize, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = Local::now().to_rfc3339();
        
        let count = conn.execute(
            "UPDATE studio_configs SET 
                pen_name = COALESCE(?2, pen_name),
                llm_config = COALESCE(?3, llm_config),
                ui_config = COALESCE(?4, ui_config),
                agent_bots = COALESCE(?5, agent_bots),
                updated_at = ?6
             WHERE id = ?1",
            params![
                id,
                pen_name,
                llm_config.map(|c| serde_json::to_string(c).unwrap()),
                ui_config.map(|c| serde_json::to_string(c).unwrap()),
                agent_bots.map(|b| serde_json::to_string(b).unwrap()),
                now
            ],
        )?;
        Ok(count)
    }

    pub fn update_themes(&self, id: &str, frontstage: Option<&str>, backstage: Option<&str>) -> Result<usize, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = Local::now().to_rfc3339();
        
        let count = conn.execute(
            "UPDATE studio_configs SET 
                frontstage_theme = COALESCE(?2, frontstage_theme),
                backstage_theme = COALESCE(?3, backstage_theme),
                updated_at = ?4
             WHERE id = ?1",
            params![id, frontstage, backstage, now],
        )?;
        Ok(count)
    }
}

// ==================== Knowledge Graph Repository ====================

pub struct KnowledgeGraphRepository {
    pool: DbPool,
}

impl KnowledgeGraphRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create_entity(&self, story_id: &str, name: &str, entity_type: &str, 
                         attributes: &serde_json::Value) -> Result<Entity, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Local::now();
        
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "INSERT INTO kg_entities (id, story_id, name, entity_type, attributes, first_seen, last_updated) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![&id, story_id, name, entity_type, attributes.to_string(), now.to_rfc3339(), now.to_rfc3339()],
        )?;
        
        Ok(Entity {
            id,
            story_id: story_id.to_string(),
            name: name.to_string(),
            entity_type: entity_type.parse().map_err(|_| rusqlite::Error::InvalidParameterName("Invalid entity type".to_string()))?,
            attributes: attributes.clone(),
            embedding: None,
            first_seen: now,
            last_updated: now,
        })
    }

    pub fn get_entities_by_story(&self, story_id: &str) -> Result<Vec<Entity>, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, name, entity_type, attributes, embedding, first_seen, last_updated 
             FROM kg_entities WHERE story_id = ?1"
        )?;

        let entities = stmt.query_map([story_id], |row| {
            let type_str: String = row.get(3)?;
            let entity_type = type_str.parse().map_err(|_| rusqlite::Error::InvalidParameterName("Invalid entity type".to_string()))?;
            
            let attrs_json: String = row.get(4)?;
            let attributes: serde_json::Value = serde_json::from_str(&attrs_json).unwrap_or_default();
            
            let first_str: String = row.get(6)?;
            let updated_str: String = row.get(7)?;
            
            Ok(Entity {
                id: row.get(0)?,
                story_id: row.get(1)?,
                name: row.get(2)?,
                entity_type,
                attributes,
                embedding: None, // TODO: 处理BLOB
                first_seen: first_str.parse().unwrap_or_else(|_| Local::now()),
                last_updated: updated_str.parse().unwrap_or_else(|_| Local::now()),
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(entities)
    }

    pub fn create_relation(&self, story_id: &str, source_id: &str, target_id: &str, 
                           relation_type: &str, strength: f32) -> Result<Relation, rusqlite::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Local::now();
        
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "INSERT INTO kg_relations (id, story_id, source_id, target_id, relation_type, strength, evidence, first_seen) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![&id, story_id, source_id, target_id, relation_type, strength, "[]", now.to_rfc3339()],
        )?;
        
        Ok(Relation {
            id,
            story_id: story_id.to_string(),
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            relation_type: relation_type.parse().map_err(|_| rusqlite::Error::InvalidParameterName("Invalid relation type".to_string()))?,
            strength,
            evidence: vec![],
            first_seen: now,
        })
    }

    pub fn get_relations_by_entity(&self, entity_id: &str) -> Result<Vec<Relation>, rusqlite::Error> {
        let conn = self.pool.get().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, source_id, target_id, relation_type, strength, evidence, first_seen 
             FROM kg_relations WHERE source_id = ?1 OR target_id = ?1"
        )?;

        let relations = stmt.query_map([entity_id], |row| {
            let type_str: String = row.get(4)?;
            let relation_type = type_str.parse().map_err(|_| rusqlite::Error::InvalidParameterName("Invalid relation type".to_string()))?;
            
            let evidence_json: String = row.get(6)?;
            let evidence: Vec<String> = serde_json::from_str(&evidence_json).unwrap_or_default();
            
            let first_str: String = row.get(7)?;
            
            Ok(Relation {
                id: row.get(0)?,
                story_id: row.get(1)?,
                source_id: row.get(2)?,
                target_id: row.get(3)?,
                relation_type,
                strength: row.get(5)?,
                evidence,
                first_seen: first_str.parse().unwrap_or_else(|_| Local::now()),
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(relations)
    }
}
