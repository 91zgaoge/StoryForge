use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Result;
use std::path::Path;

pub type DbPool = Pool<SqliteConnectionManager>;

#[cfg(test)]
pub fn create_test_pool() -> Result<DbPool, Box<dyn std::error::Error>> {
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::builder()
        .max_size(1)
        .build(manager)?;
    
    let mut conn = pool.get()?;
    create_tables(&mut conn)?;
    create_v3_tables(&mut conn)?;
    run_migrations(&mut conn)?;
    
    Ok(pool)
}

pub fn init_db(app_dir: &Path) -> Result<DbPool, Box<dyn std::error::Error>> {
    let db_path = app_dir.join("cinema_ai.db");
    let manager = SqliteConnectionManager::file(&db_path);
    let pool = Pool::builder()
        .max_size(5)
        .build(manager)?;
    
    // Initialize tables
    let mut conn = pool.get()?;
    create_tables(&mut conn)?;
    create_v3_tables(&mut conn)?;
    run_migrations(&mut conn)?;
    
    Ok(pool)
}

fn create_tables(conn: &mut rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        -- Stories table
        CREATE TABLE IF NOT EXISTS stories (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            genre TEXT,
            tone TEXT,
            pacing TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- Characters table
        CREATE TABLE IF NOT EXISTS characters (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL,
            name TEXT NOT NULL,
            background TEXT,
            personality TEXT,
            goals TEXT,
            dynamic_traits TEXT, -- JSON array
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
        );

        -- Chapters table (保留用于向后兼容，新功能使用scenes表)
        CREATE TABLE IF NOT EXISTS chapters (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL,
            chapter_number INTEGER NOT NULL,
            title TEXT,
            outline TEXT,
            content TEXT,
            word_count INTEGER,
            model_used TEXT,
            cost REAL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE,
            UNIQUE(story_id, chapter_number)
        );

        -- Create indexes
        CREATE INDEX IF NOT EXISTS idx_characters_story ON characters(story_id);
        CREATE INDEX IF NOT EXISTS idx_chapters_story ON chapters(story_id);
        CREATE INDEX IF NOT EXISTS idx_chapters_number ON chapters(story_id, chapter_number);
        "#
    )?;
    Ok(())
}

/// V3架构新表结构
fn create_v3_tables(conn: &mut rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        -- ==================== V3 新表结构 ====================

        -- 场景表（取代章节表成为主要叙事单元）
        CREATE TABLE IF NOT EXISTS scenes (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL,
            sequence_number INTEGER NOT NULL,
            title TEXT,
            dramatic_goal TEXT,             -- 戏剧目标：这个场景要完成什么
            external_pressure TEXT,         -- 外部压迫：环境/反派/事件对角色的压迫
            conflict_type TEXT,             -- 冲突类型
            characters_present TEXT,        -- JSON: [character_id, ...]
            character_conflicts TEXT,       -- JSON: [{a, b, nature, stakes}, ...]
            setting_location TEXT,
            setting_time TEXT,
            setting_atmosphere TEXT,
            content TEXT,
            previous_scene_id TEXT,
            next_scene_id TEXT,
            model_used TEXT,
            cost REAL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE,
            FOREIGN KEY (previous_scene_id) REFERENCES scenes(id),
            FOREIGN KEY (next_scene_id) REFERENCES scenes(id),
            UNIQUE(story_id, sequence_number)
        );

        -- 世界观表
        CREATE TABLE IF NOT EXISTS world_buildings (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL UNIQUE,
            concept TEXT NOT NULL,          -- 宏观世界观概念
            rules TEXT,                     -- JSON: 世界规则列表
            history TEXT,
            cultures TEXT,                  -- JSON: 文化设定
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
        );

        -- 世界规则表
        CREATE TABLE IF NOT EXISTS world_rules (
            id TEXT PRIMARY KEY,
            world_building_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            rule_type TEXT,                 -- magic/technology/social/...
            importance INTEGER,             -- 1-10
            created_at TEXT NOT NULL,
            FOREIGN KEY (world_building_id) REFERENCES world_buildings(id) ON DELETE CASCADE
        );

        -- 场景设置表（故事中的具体地点/时间设置）
        CREATE TABLE IF NOT EXISTS settings (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            location_type TEXT,             -- city/building/nature/...
            sensory_details TEXT,           -- JSON: 感官细节
            significance TEXT,              -- 在故事中的重要性
            created_at TEXT NOT NULL,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
        );

        -- 文字风格表
        CREATE TABLE IF NOT EXISTS writing_styles (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL UNIQUE,
            name TEXT,
            description TEXT,
            tone TEXT,
            pacing TEXT,
            vocabulary_level TEXT,
            sentence_structure TEXT,
            custom_rules TEXT,              -- JSON: 自定义规则
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
        );

        -- 知识图谱实体表
        CREATE TABLE IF NOT EXISTS kg_entities (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL,
            name TEXT NOT NULL,
            entity_type TEXT NOT NULL,      -- character/location/item/concept/event/organization
            attributes TEXT,                -- JSON
            embedding BLOB,                 -- 向量嵌入（可选）
            first_seen TEXT NOT NULL,
            last_updated TEXT NOT NULL,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
        );

        -- 知识图谱关系表
        CREATE TABLE IF NOT EXISTS kg_relations (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL,
            source_id TEXT NOT NULL,
            target_id TEXT NOT NULL,
            relation_type TEXT NOT NULL,
            strength REAL NOT NULL,         -- 0-1
            evidence TEXT,                  -- JSON: 场景ID列表
            first_seen TEXT NOT NULL,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE,
            FOREIGN KEY (source_id) REFERENCES kg_entities(id),
            FOREIGN KEY (target_id) REFERENCES kg_entities(id)
        );

        -- 工作室配置表（存储每部小说的独立配置）
        CREATE TABLE IF NOT EXISTS studio_configs (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL UNIQUE,
            pen_name TEXT,
            llm_config TEXT,                -- JSON: LLM配置
            ui_config TEXT,                 -- JSON: UI配置
            agent_bots TEXT,                -- JSON: Agent Bot配置
            frontstage_theme TEXT,          -- CSS内容
            backstage_theme TEXT,           -- CSS内容
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
        );

        -- 场景批注表
        CREATE TABLE IF NOT EXISTS scene_annotations (
            id TEXT PRIMARY KEY,
            scene_id TEXT NOT NULL,
            story_id TEXT NOT NULL,
            content TEXT NOT NULL,
            annotation_type TEXT NOT NULL DEFAULT 'note',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            resolved_at TEXT,
            FOREIGN KEY (scene_id) REFERENCES scenes(id) ON DELETE CASCADE,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
        );

        -- 文本内联批注表（TipTap range comments）
        CREATE TABLE IF NOT EXISTS text_annotations (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL,
            scene_id TEXT,
            chapter_id TEXT,
            content TEXT NOT NULL,
            annotation_type TEXT NOT NULL DEFAULT 'note',
            from_pos INTEGER NOT NULL,
            to_pos INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            resolved_at TEXT,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
        );

        -- 故事摘要表（知识蒸馏、剧情总结等）
        CREATE TABLE IF NOT EXISTS story_summaries (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL,
            summary_type TEXT NOT NULL DEFAULT 'knowledge_distillation',
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
        );

        -- 创建索引
        CREATE INDEX IF NOT EXISTS idx_scenes_story ON scenes(story_id);
        CREATE INDEX IF NOT EXISTS idx_scenes_sequence ON scenes(story_id, sequence_number);
        CREATE INDEX IF NOT EXISTS idx_scenes_prev ON scenes(previous_scene_id);
        CREATE INDEX IF NOT EXISTS idx_scenes_next ON scenes(next_scene_id);
        
        CREATE INDEX IF NOT EXISTS idx_world_buildings_story ON world_buildings(story_id);
        CREATE INDEX IF NOT EXISTS idx_world_rules_wb ON world_rules(world_building_id);
        CREATE INDEX IF NOT EXISTS idx_settings_story ON settings(story_id);
        CREATE INDEX IF NOT EXISTS idx_writing_styles_story ON writing_styles(story_id);
        
        CREATE INDEX IF NOT EXISTS idx_kg_entities_story ON kg_entities(story_id);
        CREATE INDEX IF NOT EXISTS idx_kg_entities_type ON kg_entities(entity_type);
        CREATE INDEX IF NOT EXISTS idx_kg_relations_story ON kg_relations(story_id);
        CREATE INDEX IF NOT EXISTS idx_kg_relations_source ON kg_relations(source_id);
        CREATE INDEX IF NOT EXISTS idx_kg_relations_target ON kg_relations(target_id);
        CREATE INDEX IF NOT EXISTS idx_kg_relations_type ON kg_relations(relation_type);
        
        CREATE INDEX IF NOT EXISTS idx_studio_configs_story ON studio_configs(story_id);
        CREATE INDEX IF NOT EXISTS idx_scene_annotations_scene ON scene_annotations(scene_id);
        CREATE INDEX IF NOT EXISTS idx_scene_annotations_story ON scene_annotations(story_id);
        CREATE INDEX IF NOT EXISTS idx_scene_annotations_resolved ON scene_annotations(resolved_at);
        CREATE INDEX IF NOT EXISTS idx_text_annotations_story ON text_annotations(story_id);
        CREATE INDEX IF NOT EXISTS idx_text_annotations_scene ON text_annotations(scene_id);
        CREATE INDEX IF NOT EXISTS idx_text_annotations_chapter ON text_annotations(chapter_id);
        CREATE INDEX IF NOT EXISTS idx_text_annotations_resolved ON text_annotations(resolved_at);
        CREATE INDEX IF NOT EXISTS idx_story_summaries_story ON story_summaries(story_id);
        CREATE INDEX IF NOT EXISTS idx_story_summaries_type ON story_summaries(story_id, summary_type);
        "#
    )?;
    Ok(())
}

/// 数据库迁移
fn run_migrations(conn: &mut rusqlite::Connection) -> Result<(), rusqlite::Error> {
    // Migration 1: 添加实体归档字段 (v3.2.0)
    let columns: Vec<String> = conn.prepare(
        "PRAGMA table_info(kg_entities)"
    )?.query_map([], |row| {
        let name: String = row.get(1)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;
    
    if !columns.iter().any(|c| c == "is_archived") {
        conn.execute(
            "ALTER TABLE kg_entities ADD COLUMN is_archived INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    if !columns.iter().any(|c| c == "archived_at") {
        conn.execute(
            "ALTER TABLE kg_entities ADD COLUMN archived_at TEXT",
            [],
        )?;
    }
    
    // 创建归档索引（仅在 kg_entities 表已存在时）
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_kg_entities_archived ON kg_entities(is_archived)",
        [],
    )?;
    
    // Migration 2: 添加实体保留字段 (v3.1.0 - 如果缺失)
    if !columns.iter().any(|c| c == "confidence_score") {
        conn.execute(
            "ALTER TABLE kg_entities ADD COLUMN confidence_score REAL",
            [],
        )?;
    }
    if !columns.iter().any(|c| c == "access_count") {
        conn.execute(
            "ALTER TABLE kg_entities ADD COLUMN access_count INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    if !columns.iter().any(|c| c == "last_accessed") {
        conn.execute(
            "ALTER TABLE kg_entities ADD COLUMN last_accessed TEXT",
            [],
        )?;
    }

    // Migration 3: 创建场景批注表 (v3.2.0)
    let annotation_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='scene_annotations'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if annotation_tables.is_empty() {
        conn.execute(
            "CREATE TABLE scene_annotations (
                id TEXT PRIMARY KEY,
                scene_id TEXT NOT NULL,
                story_id TEXT NOT NULL,
                content TEXT NOT NULL,
                annotation_type TEXT NOT NULL DEFAULT 'note',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                resolved_at TEXT,
                FOREIGN KEY (scene_id) REFERENCES scenes(id) ON DELETE CASCADE,
                FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_scene_annotations_scene ON scene_annotations(scene_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_scene_annotations_story ON scene_annotations(story_id)",
            [],
        )?;
    }

    // Migration 4: 创建文本内联批注表 (v3.2.0)
    let text_annotation_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='text_annotations'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if text_annotation_tables.is_empty() {
        conn.execute(
            "CREATE TABLE text_annotations (
                id TEXT PRIMARY KEY,
                story_id TEXT NOT NULL,
                scene_id TEXT,
                chapter_id TEXT,
                content TEXT NOT NULL,
                annotation_type TEXT NOT NULL DEFAULT 'note',
                from_pos INTEGER NOT NULL,
                to_pos INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                resolved_at TEXT,
                FOREIGN KEY (story_id) REFERENCES stories(id) ON DELETE CASCADE
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_text_annotations_story ON text_annotations(story_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_text_annotations_scene ON text_annotations(scene_id)",
            [],
        )?;
    }
    
    Ok(())
}
