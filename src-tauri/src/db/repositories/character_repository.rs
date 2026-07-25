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
        let id = Uuid::new_v4().to_string();
        let now = Local::now();
        let traits_json = "[]";

        let source = req.source.as_deref().unwrap_or("user_created");
        let is_auto_generated = req.is_auto_generated.unwrap_or(false) as i32;

        tx.execute(
            "INSERT INTO characters (id, story_id, name, background, personality, goals, \
             appearance, gender, age, dynamic_traits, source, is_auto_generated, created_at, \
             updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                &id,
                &req.story_id,
                &req.name,
                req.background,
                req.personality,
                req.goals,
                req.appearance,
                req.gender,
                req.age,
                traits_json,
                source,
                is_auto_generated,
                now.to_rfc3339(),
                now.to_rfc3339()
            ],
        )?;

        Ok(Character {
            id,
            story_id: req.story_id,
            name: req.name,
            background: req.background,
            personality: req.personality,
            goals: req.goals,
            appearance: req.appearance,
            gender: req.gender,
            age: req.age,
            dynamic_traits: vec![],
            source: Some(source.to_string()),
            is_auto_generated: Some(is_auto_generated != 0),
            created_at: now,
            updated_at: now,
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
        let mut stmt = conn.prepare(
            "SELECT id, story_id, name, background, personality, goals, appearance, gender, age, \
             dynamic_traits, source, is_auto_generated, created_at, updated_at FROM characters \
             WHERE story_id = ?1",
        )?;

        let characters = stmt
            .query_map([story_id], |row| {
                // dynamic_traits 列无 NOT NULL/DEFAULT 约束，旧数据（列迁移
                // 前的行）可能为 NULL -> 读为 Option 兜底 "[]"，避免
                // "Invalid column type Null at index: 9" 致获取角色失败。
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

    pub fn get_by_id(&self, id: &str) -> Result<Option<Character>, rusqlite::Error> {
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, story_id, name, background, personality, goals, appearance, gender, age, \
             dynamic_traits, source, is_auto_generated, created_at, updated_at FROM characters \
             WHERE id = ?1",
        )?;

        let character = stmt
            .query_row([id], |row| {
                // dynamic_traits 列无 NOT NULL/DEFAULT 约束，旧数据（列迁移
                // 前的行）可能为 NULL -> 读为 Option 兜底 "[]"，避免
                // "Invalid column type Null at index: 9" 致获取角色失败。
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
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = Local::now().to_rfc3339();

        let count = conn.execute(
            "UPDATE characters SET name = COALESCE(?2, name), background = COALESCE(?3, \
             background),
             personality = COALESCE(?4, personality), goals = COALESCE(?5, goals), appearance = \
             COALESCE(?6, appearance),
             gender = COALESCE(?7, gender), age = COALESCE(?8, age), updated_at = ?9 WHERE id = ?1",
            params![
                id,
                name,
                background,
                personality,
                goals,
                appearance,
                gender,
                age,
                now
            ],
        )?;
        Ok(count)
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

        if count == 0 {
            conn.execute(
                "INSERT INTO character_states (
                    id, story_id, character_id, location, power_level, physical_state,
                    mental_state, key_items, recent_events, updated_at_chapter, cs_json,
                    last_updated
                )
                SELECT ?1, c.story_id, c.id, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11
                FROM characters c
                WHERE c.id = ?2",
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
            )?;
        }

        Ok(count.max(1))
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

        // 验证角色是否存在
        let exists: bool = tx
            .query_row("SELECT 1 FROM characters WHERE id = ?1", [id], |_| Ok(true))
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
        let count = tx.execute("DELETE FROM characters WHERE id = ?1", [id])?;

        tx.commit()?;
        Ok(count)
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

    /// 回归：dynamic_traits 列为 NULL（旧数据 / StoryForge 迁移导入的行）
    /// 时，get_by_story / get_by_id 不得报 "Invalid column type Null at
    /// index: 9"，应兜底为空数组。
    #[test]
    fn test_get_by_story_tolerates_null_dynamic_traits() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let story = story_repo.create(story_req("角色测试")).unwrap();
        let repo = CharacterRepository::new(pool.clone());
        let ch = repo.create(req(&story.id, "李明")).unwrap();

        // 模拟旧数据：手动置 NULL（覆盖 V111 回填未触及的路径）
        let conn = pool.get().unwrap();
        conn.execute(
            "UPDATE characters SET dynamic_traits = NULL WHERE id = ?1",
            [&ch.id],
        )
        .unwrap();
        drop(conn);

        // get_by_story 不得 panic/Err
        let fetched = repo.get_by_story(&story.id).unwrap();
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].name, "李明");
        assert!(
            fetched[0].dynamic_traits.is_empty(),
            "NULL dynamic_traits 应兜底为空数组"
        );

        // get_by_id 同理
        let by_id = repo.get_by_id(&ch.id).unwrap().unwrap();
        assert!(by_id.dynamic_traits.is_empty());
    }

    /// 正常路径：dynamic_traits 为合法 JSON 数组时正确解析。
    #[test]
    fn test_get_by_story_parses_dynamic_traits_json() {
        let pool = create_test_pool().unwrap();
        let story_repo = StoryRepository::new(pool.clone());
        let story = story_repo.create(story_req("角色测试2")).unwrap();
        let repo = CharacterRepository::new(pool.clone());
        let ch = repo.create(req(&story.id, "韩雪")).unwrap();

        // 写入合法 JSON 数组（模拟有动态特征的角色）
        let conn = pool.get().unwrap();
        conn.execute(
            "UPDATE characters SET dynamic_traits = ?1 WHERE id = ?2",
            params![r#"[{"trait":"坚定","confidence":0.9}]"#, &ch.id],
        )
        .unwrap();
        drop(conn);

        let fetched = repo.get_by_story(&story.id).unwrap();
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].dynamic_traits.len(), 1);
        assert_eq!(fetched[0].dynamic_traits[0].trait_name, "坚定");
    }
}
