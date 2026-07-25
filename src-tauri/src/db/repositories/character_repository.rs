use std::collections::HashMap;

use super::*;

pub struct CharacterRepository {
    pool: DbPool,
}

impl CharacterRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create_in_tx(
        &self,
        tx: &rusqlite::Transaction,
        req: CreateCharacterRequest,
    ) -> Result<Character, rusqlite::Error> {
        let kg_repo = KnowledgeGraphRepository::new(self.pool.clone());

        let source = req.source.as_deref().unwrap_or("user_created");
        let is_auto_generated = req.is_auto_generated.unwrap_or(false);

        let attributes = serde_json::json!({
            "background": req.background,
            "personality": req.personality,
            "goals": req.goals,
            "appearance": req.appearance,
            "gender": req.gender,
            "age": req.age,
            "dynamic_traits": serde_json::json!([]),
        });

        let entity = kg_repo.create_entity_in_tx_with_source(
            tx,
            &req.story_id,
            &req.name,
            "Character",
            &attributes,
            None,
            Some(source),
            Some(is_auto_generated),
        )?;

        // Keep the legacy `characters` table in sync so that existing direct
        // SQL consumers and foreign-key constraints continue to work during the
        // transition.  `kg_entities` remains the canonical store.
        tx.execute(
            "INSERT INTO characters (id, story_id, name, background, personality, goals, \
             appearance, gender, age, dynamic_traits, source, is_auto_generated, created_at, \
             updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                &entity.id,
                &req.story_id,
                &req.name,
                req.background,
                req.personality,
                req.goals,
                req.appearance,
                req.gender,
                req.age,
                "[]",
                source,
                is_auto_generated as i32,
                entity.first_seen.to_rfc3339(),
                entity.last_updated.to_rfc3339(),
            ],
        )?;

        Character::from_entity(&entity).ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("Created entity is not a Character".to_string())
        })
    }

    pub fn create(&self, req: CreateCharacterRequest) -> Result<Character, rusqlite::Error> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let tx = conn.transaction()?;
        let character = self.create_in_tx(&tx, req)?;
        tx.commit()?;
        Ok(character)
    }

    pub fn get_by_story(&self, story_id: &str) -> Result<Vec<Character>, rusqlite::Error> {
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

        // Canonical source: kg_entities.
        let mut stmt = conn.prepare(
            "SELECT id, story_id, name, entity_type, attributes, first_seen, last_updated, \
             source, is_auto_generated
             FROM kg_entities
             WHERE story_id = ?1 AND entity_type = 'Character' AND is_archived = 0",
        )?;

        let entities = stmt
            .query_map([story_id], |row| {
                let attrs_json: String = row
                    .get::<_, Option<String>>(4)?
                    .unwrap_or_else(|| "{}".to_string());
                let attributes: serde_json::Value =
                    serde_json::from_str(&attrs_json).unwrap_or_default();

                Ok(Entity {
                    id: row.get(0)?,
                    story_id: row.get(1)?,
                    name: row.get(2)?,
                    entity_type: EntityType::Character,
                    attributes,
                    embedding: None,
                    first_seen: row
                        .get::<_, String>(5)?
                        .parse()
                        .unwrap_or_else(|_| Local::now()),
                    last_updated: row
                        .get::<_, String>(6)?
                        .parse()
                        .unwrap_or_else(|_| Local::now()),
                    confidence_score: None,
                    access_count: 0,
                    last_accessed: None,
                    is_archived: false,
                    archived_at: None,
                    source: row.get(7).ok(),
                    is_auto_generated: row.get::<_, Option<i32>>(8).ok().flatten().map(|v| v != 0),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut result: Vec<Character> = entities
            .into_iter()
            .filter_map(|e| Character::from_entity(&e))
            .collect();

        // Compatibility fallback: rows that exist only in the legacy `characters`
        // table (e.g. older code paths or tests that insert directly).
        let mut stmt = conn.prepare(
            "SELECT id, story_id, name, background, personality, goals, appearance, gender, age, \
             dynamic_traits, source, is_auto_generated, created_at, updated_at
             FROM characters
             WHERE story_id = ?1 AND id NOT IN (
                 SELECT id FROM kg_entities WHERE story_id = ?1 AND entity_type = 'Character'
             )",
        )?;
        let legacy_rows = stmt
            .query_map([story_id], |row| {
                let traits_json: String = row
                    .get::<_, Option<String>>(9)?
                    .unwrap_or_else(|| "[]".to_string());
                let dynamic_traits: Vec<DynamicTrait> =
                    serde_json::from_str(&traits_json).unwrap_or_default();
                let created_str: String = row.get(12)?;
                let updated_str: String = row.get(13)?;
                let is_auto_generated: Option<i32> = row.get(11).ok();

                Ok(Character {
                    id: row.get(0)?,
                    story_id: row.get(1)?,
                    name: row.get(2)?,
                    background: row.get(3)?,
                    personality: row.get(4)?,
                    goals: row.get(5)?,
                    appearance: row.get(6)?,
                    gender: row.get(7)?,
                    age: row.get(8)?,
                    dynamic_traits,
                    source: row.get(10).ok(),
                    is_auto_generated: is_auto_generated.map(|v| v != 0),
                    created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                    updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        result.extend(legacy_rows);

        Ok(result)
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Character>, rusqlite::Error> {
        let kg_repo = KnowledgeGraphRepository::new(self.pool.clone());
        if let Some(entity) = kg_repo.get_entity_by_id(id)? {
            return Ok(Character::from_entity(&entity));
        }

        // Compatibility fallback: legacy `characters` table.
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, name, background, personality, goals, appearance, gender, age, \
             dynamic_traits, source, is_auto_generated, created_at, updated_at
             FROM characters WHERE id = ?1",
        )?;
        let character = stmt
            .query_row([id], |row| {
                let traits_json: String = row
                    .get::<_, Option<String>>(9)?
                    .unwrap_or_else(|| "[]".to_string());
                let dynamic_traits: Vec<DynamicTrait> =
                    serde_json::from_str(&traits_json).unwrap_or_default();
                let created_str: String = row.get(12)?;
                let updated_str: String = row.get(13)?;
                let is_auto_generated: Option<i32> = row.get(11).ok();

                Ok(Character {
                    id: row.get(0)?,
                    story_id: row.get(1)?,
                    name: row.get(2)?,
                    background: row.get(3)?,
                    personality: row.get(4)?,
                    goals: row.get(5)?,
                    appearance: row.get(6)?,
                    gender: row.get(7)?,
                    age: row.get(8)?,
                    dynamic_traits,
                    source: row.get(10).ok(),
                    is_auto_generated: is_auto_generated.map(|v| v != 0),
                    created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                    updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
                })
            })
            .optional()?;
        Ok(character)
    }

    pub fn update(
        &self,
        id: &str,
        name: Option<String>,
        background: Option<String>,
        personality: Option<String>,
        goals: Option<String>,
        appearance: Option<String>,
        gender: Option<String>,
        age: Option<i32>,
    ) -> Result<usize, rusqlite::Error> {
        let kg_repo = KnowledgeGraphRepository::new(self.pool.clone());
        let entity = match kg_repo.get_entity_by_id(id)? {
            Some(e) => e,
            None => return Ok(0),
        };

        if entity.entity_type != EntityType::Character {
            return Ok(0);
        }

        // Keep the legacy `characters` table in sync first, using references so
        // the Option<String> values remain available for the canonical update.
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = Local::now().to_rfc3339();
        conn.execute(
            "UPDATE characters SET name = COALESCE(?2, name),
                 background = COALESCE(?3, background),
                 personality = COALESCE(?4, personality),
                 goals = COALESCE(?5, goals),
                 appearance = COALESCE(?6, appearance),
                 gender = COALESCE(?7, gender),
                 age = COALESCE(?8, age),
                 updated_at = ?9
             WHERE id = ?1",
            params![
                id,
                name.as_deref(),
                background.as_deref(),
                personality.as_deref(),
                goals.as_deref(),
                appearance.as_deref(),
                gender.as_deref(),
                age,
                now
            ],
        )?;
        drop(conn);

        let mut attrs = entity.attributes.clone();
        if let Some(map) = attrs.as_object_mut() {
            if let Some(background) = background {
                map.insert("background".to_string(), serde_json::json!(background));
            }
            if let Some(personality) = personality {
                map.insert("personality".to_string(), serde_json::json!(personality));
            }
            if let Some(goals) = goals {
                map.insert("goals".to_string(), serde_json::json!(goals));
            }
            if let Some(appearance) = appearance {
                map.insert("appearance".to_string(), serde_json::json!(appearance));
            }
            if let Some(gender) = gender {
                map.insert("gender".to_string(), serde_json::json!(gender));
            }
            if let Some(age) = age {
                map.insert("age".to_string(), serde_json::json!(age));
            }
        }

        let new_name = name.as_deref().unwrap_or(&entity.name);
        kg_repo.update_entity(id, Some(new_name), Some(&attrs), None)?;

        Ok(1)
    }

    /// 将角色动态状态写入 `character_states` 表（而非 `characters` 的 `cs_*`
    /// 列）。
    pub fn update_character_state(
        &self,
        character_id: &str,
        state: &CharacterState,
    ) -> Result<usize, rusqlite::Error> {
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = Local::now().to_rfc3339();

        // 先尝试更新已有行，保留其他列（如 current_location / secrets_known 等）。
        let count = conn.execute(
            "UPDATE character_states SET
                location = COALESCE(?2, location),
                power_level = COALESCE(?3, power_level),
                physical_state = COALESCE(?4, physical_state),
                mental_state = COALESCE(?5, mental_state),
                key_items = COALESCE(?6, key_items),
                recent_events = COALESCE(?7, recent_events),
                updated_at_chapter = COALESCE(?8, updated_at_chapter),
                cs_json = COALESCE(?9, cs_json),
                last_updated = ?10
            WHERE character_id = ?1",
            params![
                character_id,
                state.location,
                state.power_level,
                state.physical_state,
                state.mental_state,
                state.key_items,
                state.recent_events,
                state.updated_at_chapter,
                state.cs_json,
                now,
            ],
        )?;

        let insert_count = if count == 0 {
            conn.execute(
                "INSERT INTO character_states (
                    id, story_id, character_id, location, power_level, physical_state,
                    mental_state, key_items, recent_events, updated_at_chapter, cs_json,
                    last_updated
                )
                SELECT ?1, e.story_id, e.id, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11
                FROM kg_entities e
                WHERE e.id = ?2 AND e.entity_type = 'Character'",
                params![
                    Uuid::new_v4().to_string(),
                    character_id,
                    state.location,
                    state.power_level,
                    state.physical_state,
                    state.mental_state,
                    state.key_items,
                    state.recent_events,
                    state.updated_at_chapter,
                    state.cs_json,
                    now,
                ],
            )?
        } else {
            0
        };

        Ok(count + insert_count)
    }

    pub fn get_character_state(
        &self,
        character_id: &str,
    ) -> Result<Option<CharacterState>, rusqlite::Error> {
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT location, power_level, physical_state, mental_state, key_items, recent_events, \
             updated_at_chapter, cs_json, state_transitions_json, arc_type \
             FROM character_states WHERE character_id = ?1 LIMIT 1",
        )?;

        let state = stmt
            .query_row([character_id], |row| {
                Ok(CharacterState {
                    location: row.get(0).ok(),
                    power_level: row.get(1).ok(),
                    physical_state: row.get(2).ok(),
                    mental_state: row.get(3).ok(),
                    key_items: row.get(4).ok(),
                    recent_events: row.get(5).ok(),
                    updated_at_chapter: row.get(6).ok(),
                    cs_json: row.get(7).ok(),
                    state_transitions_json: row.get(8).ok(),
                    arc_type: row.get(9).ok(),
                })
            })
            .optional()?;

        Ok(state)
    }

    /// 一次性加载故事下所有角色的动态状态，按 `character_id` 索引。
    pub fn get_character_states_by_story(
        &self,
        story_id: &str,
    ) -> Result<HashMap<String, CharacterState>, rusqlite::Error> {
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT character_id, location, power_level, physical_state, mental_state, key_items, \
             recent_events, updated_at_chapter, cs_json, state_transitions_json, arc_type \
             FROM character_states WHERE story_id = ?1",
        )?;

        let rows = stmt.query_map([story_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                CharacterState {
                    location: row.get(1).ok(),
                    power_level: row.get(2).ok(),
                    physical_state: row.get(3).ok(),
                    mental_state: row.get(4).ok(),
                    key_items: row.get(5).ok(),
                    recent_events: row.get(6).ok(),
                    updated_at_chapter: row.get(7).ok(),
                    cs_json: row.get(8).ok(),
                    state_transitions_json: row.get(9).ok(),
                    arc_type: row.get(10).ok(),
                },
            ))
        })?;

        let mut map = HashMap::new();
        for row in rows {
            let (character_id, state) = row?;
            map.insert(character_id, state);
        }
        Ok(map)
    }

    pub fn delete(&self, id: &str) -> Result<usize, rusqlite::Error> {
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

        // 在事务中执行删除操作
        let tx = conn.unchecked_transaction()?;

        // 验证角色是否存在（限定 Character 类型）
        let exists: bool = tx
            .query_row(
                "SELECT 1 FROM kg_entities WHERE id = ?1 AND entity_type = 'Character'",
                [id],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !exists {
            tx.rollback()?;
            return Ok(0);
        }
        let _ = tx.execute("DELETE FROM scene_characters WHERE character_id = ?1", [id]);
        let _ = tx.execute(
            "DELETE FROM scene_character_actions WHERE character_id = ?1",
            [id],
        );
        let _ = tx.execute(
            "DELETE FROM character_relationships WHERE source_character_id = ?1 OR \
             target_character_id = ?1",
            [id],
        );
        let _ = tx.execute("DELETE FROM character_states WHERE character_id = ?1", [id]);

        // 执行删除操作 - 外键约束会自动级联剩余关联数据
        let count = tx.execute(
            "DELETE FROM kg_entities WHERE id = ?1 AND entity_type = 'Character'",
            [id],
        )?;

        // Keep the legacy `characters` table in sync.
        let _ = tx.execute("DELETE FROM characters WHERE id = ?1", [id]);

        tx.commit()?;
        Ok(count)
    }

    /// 通过兼容视图 `v_characters` 读取旧列布局的行。
    ///
    /// 主要用于验证迁移 / 视图行为；新增代码应优先使用 `get_by_story`。
    pub fn get_from_view(&self, story_id: &str) -> Result<Vec<Character>, rusqlite::Error> {
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, name, background, personality, goals, appearance, gender, age, \
             dynamic_traits, source, is_auto_generated, created_at, updated_at
             FROM v_characters WHERE story_id = ?1",
        )?;

        let characters = stmt
            .query_map([story_id], |row| {
                let traits_json: String = row
                    .get::<_, Option<String>>(9)?
                    .unwrap_or_else(|| "[]".to_string());
                let dynamic_traits: Vec<DynamicTrait> =
                    serde_json::from_str(&traits_json).unwrap_or_default();
                let created_str: String = row.get(12)?;
                let updated_str: String = row.get(13)?;
                let is_auto_generated: Option<i32> = row.get(11).ok();

                Ok(Character {
                    id: row.get(0)?,
                    story_id: row.get(1)?,
                    name: row.get(2)?,
                    background: row.get(3)?,
                    personality: row.get(4)?,
                    goals: row.get(5)?,
                    appearance: row.get(6)?,
                    gender: row.get(7)?,
                    age: row.get(8)?,
                    dynamic_traits,
                    source: row.get(10).ok(),
                    is_auto_generated: is_auto_generated.map(|v| v != 0),
                    created_at: created_str.parse().unwrap_or_else(|_| Local::now()),
                    updated_at: updated_str.parse().unwrap_or_else(|_| Local::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(characters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::create_test_pool;

    fn req(story_id: &str, name: &str) -> CreateCharacterRequest {
        CreateCharacterRequest {
            story_id: story_id.to_string(),
            name: name.to_string(),
            background: None,
            personality: None,
            goals: None,
            appearance: None,
            gender: None,
            age: None,
            source: None,
            is_auto_generated: None,
        }
    }

    fn story_req(title: &str) -> CreateStoryRequest {
        CreateStoryRequest {
            title: title.to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
            genre_profile_id: None,
            methodology_id: None,
            reference_book_id: None,
        }
    }

    #[test]
    fn test_create_character_writes_to_kg_entities() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let story = story_repo.create(story_req("角色测试")).unwrap();
        let repo = CharacterRepository::new(pool.clone());

        let ch = repo
            .create(CreateCharacterRequest {
                story_id: story.id.clone(),
                name: "李明".to_string(),
                background: Some("背景".to_string()),
                personality: Some("性格".to_string()),
                goals: Some("目标".to_string()),
                appearance: Some("外貌".to_string()),
                gender: Some("male".to_string()),
                age: Some(25),
                source: None,
                is_auto_generated: None,
            })
            .unwrap();

        // 直接查询 kg_entities 验证
        let kg_repo = KnowledgeGraphRepository::new(pool.clone());
        let entity = kg_repo.get_entity_by_id(&ch.id).unwrap().unwrap();
        assert_eq!(entity.entity_type, EntityType::Character);
        assert_eq!(entity.name, "李明");
        assert_eq!(entity.attributes["background"].as_str(), Some("背景"));
        assert_eq!(entity.attributes["age"].as_i64(), Some(25));

        let by_story = repo.get_by_story(&story.id).unwrap();
        assert_eq!(by_story.len(), 1);
        assert_eq!(by_story[0].name, "李明");
        assert_eq!(by_story[0].background.as_deref(), Some("背景"));
    }

    #[test]
    fn test_update_character_mutates_kg_entities() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let story = story_repo.create(story_req("角色测试2")).unwrap();
        let repo = CharacterRepository::new(pool.clone());
        let ch = repo.create(req(&story.id, "韩雪")).unwrap();

        let count = repo
            .update(
                &ch.id,
                Some("韩雪（改名）".to_string()),
                Some("新背景".to_string()),
                None,
                None,
                Some("新外貌".to_string()),
                None,
                None,
            )
            .unwrap();
        assert_eq!(count, 1);

        let kg_repo = KnowledgeGraphRepository::new(pool.clone());
        let entity = kg_repo.get_entity_by_id(&ch.id).unwrap().unwrap();
        assert_eq!(entity.name, "韩雪（改名）");
        assert_eq!(entity.attributes["background"].as_str(), Some("新背景"));
        assert_eq!(entity.attributes["appearance"].as_str(), Some("新外貌"));
        // 未提供的字段保持不变
        assert!(entity.attributes["personality"].is_null());
    }

    #[test]
    fn test_delete_character_removes_kg_entity() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let story = story_repo.create(story_req("角色测试3")).unwrap();
        let repo = CharacterRepository::new(pool.clone());
        let ch = repo.create(req(&story.id, "张伟")).unwrap();

        let count = repo.delete(&ch.id).unwrap();
        assert_eq!(count, 1);

        let kg_repo = KnowledgeGraphRepository::new(pool.clone());
        assert!(kg_repo.get_entity_by_id(&ch.id).unwrap().is_none());
        assert!(repo.get_by_id(&ch.id).unwrap().is_none());
    }

    /// 回归：通过兼容视图读取角色，dynamic_traits 应正确解析。
    #[test]
    fn test_read_character_from_compatibility_view() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let story = story_repo.create(story_req("角色测试4")).unwrap();
        let repo = CharacterRepository::new(pool.clone());
        let ch = repo.create(req(&story.id, "王芳")).unwrap();

        // 手动写入 dynamic_traits（模拟有动态特征的角色）
        let conn = pool.get().unwrap();
        conn.execute(
            "UPDATE kg_entities SET attributes = json_set(attributes, '$.dynamic_traits', \
             json_array(json_object('trait', '坚定', 'confidence', 0.9))) WHERE id = ?1",
            [&ch.id],
        )
        .unwrap();
        drop(conn);

        let from_view = repo.get_from_view(&story.id).unwrap();
        assert_eq!(from_view.len(), 1);
        assert_eq!(from_view[0].name, "王芳");
        assert_eq!(from_view[0].dynamic_traits.len(), 1);
        assert_eq!(from_view[0].dynamic_traits[0].trait_name, "坚定");
    }
}
