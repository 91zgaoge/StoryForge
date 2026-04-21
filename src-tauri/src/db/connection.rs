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
    
    // 测试环境：创建 scene_versions 表（被 change_tracks/comment_threads 外键引用）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scene_versions (
            id TEXT PRIMARY KEY,
            scene_id TEXT NOT NULL,
            chapter_id TEXT,
            content TEXT,
            word_count INTEGER,
            created_at TEXT NOT NULL,
            FOREIGN KEY (scene_id) REFERENCES scenes(id) ON DELETE CASCADE,
            FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE CASCADE
        )",
        [],
    )?;
    
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
    // Migration 17: 创建任务表和任务日志表 (v3.5.0)
    let task_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='tasks'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if task_tables.is_empty() {
        conn.execute(
            "CREATE TABLE tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                task_type TEXT NOT NULL DEFAULT 'custom',
                schedule_type TEXT NOT NULL DEFAULT 'once',
                cron_pattern TEXT,
                payload TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                progress INTEGER NOT NULL DEFAULT 0,
                result TEXT,
                error_message TEXT,
                max_retries INTEGER NOT NULL DEFAULT 3,
                retry_count INTEGER NOT NULL DEFAULT 0,
                enabled INTEGER NOT NULL DEFAULT 1,
                last_run_at TEXT,
                next_run_at TEXT,
                last_heartbeat_at TEXT,
                heartbeat_timeout_seconds INTEGER NOT NULL DEFAULT 300,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_tasks_status ON tasks(status)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_tasks_type ON tasks(task_type)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_tasks_enabled ON tasks(enabled)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_tasks_next_run ON tasks(next_run_at)",
            [],
        )?;
        conn.execute(
            "CREATE TABLE task_logs (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                log_level TEXT NOT NULL,
                message TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_task_logs_task ON task_logs(task_id)",
            [],
        )?;
    }

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

        -- 变更追踪表（修订模式）
        CREATE TABLE IF NOT EXISTS change_tracks (
            id TEXT PRIMARY KEY,
            scene_id TEXT,
            chapter_id TEXT,
            version_id TEXT,
            author_id TEXT NOT NULL,
            author_name TEXT,
            change_type TEXT NOT NULL,
            from_pos INTEGER NOT NULL,
            to_pos INTEGER NOT NULL,
            content TEXT,
            status TEXT NOT NULL DEFAULT 'Pending',
            created_at TEXT NOT NULL,
            resolved_at TEXT,
            FOREIGN KEY (scene_id) REFERENCES scenes(id) ON DELETE CASCADE,
            FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE CASCADE,
            FOREIGN KEY (version_id) REFERENCES scene_versions(id)
        );

        -- 评论线程表
        CREATE TABLE IF NOT EXISTS comment_threads (
            id TEXT PRIMARY KEY,
            scene_id TEXT,
            chapter_id TEXT,
            version_id TEXT,
            anchor_type TEXT NOT NULL,
            from_pos INTEGER,
            to_pos INTEGER,
            selected_text TEXT,
            status TEXT NOT NULL DEFAULT 'Open',
            created_at TEXT NOT NULL,
            resolved_at TEXT,
            FOREIGN KEY (scene_id) REFERENCES scenes(id) ON DELETE CASCADE,
            FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE CASCADE,
            FOREIGN KEY (version_id) REFERENCES scene_versions(id)
        );

        -- 评论消息表
        CREATE TABLE IF NOT EXISTS comment_messages (
            id TEXT PRIMARY KEY,
            thread_id TEXT NOT NULL,
            author_id TEXT NOT NULL,
            author_name TEXT,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (thread_id) REFERENCES comment_threads(id) ON DELETE CASCADE
        );

        -- 创建索引
        CREATE INDEX IF NOT EXISTS idx_change_tracks_scene ON change_tracks(scene_id);
        CREATE INDEX IF NOT EXISTS idx_change_tracks_chapter ON change_tracks(chapter_id);
        CREATE INDEX IF NOT EXISTS idx_change_tracks_status ON change_tracks(status);
        CREATE INDEX IF NOT EXISTS idx_comment_threads_scene ON comment_threads(scene_id);
        CREATE INDEX IF NOT EXISTS idx_comment_threads_chapter ON comment_threads(chapter_id);
        CREATE INDEX IF NOT EXISTS idx_comment_messages_thread ON comment_messages(thread_id);
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

        -- 参考小说表（拆书功能）
        CREATE TABLE IF NOT EXISTS reference_books (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            author TEXT,
            genre TEXT,
            word_count INTEGER,
            file_format TEXT,
            file_hash TEXT UNIQUE,
            file_path TEXT,
            world_setting TEXT,
            plot_summary TEXT,
            story_arc TEXT,
            analysis_status TEXT NOT NULL DEFAULT 'pending',
            analysis_progress INTEGER DEFAULT 0,
            analysis_error TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        -- 参考人物表
        CREATE TABLE IF NOT EXISTS reference_characters (
            id TEXT PRIMARY KEY,
            book_id TEXT NOT NULL,
            name TEXT NOT NULL,
            role_type TEXT,
            personality TEXT,
            appearance TEXT,
            relationships TEXT,
            key_scenes TEXT,
            importance_score REAL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (book_id) REFERENCES reference_books(id) ON DELETE CASCADE
        );

        -- 参考场景/章节表
        CREATE TABLE IF NOT EXISTS reference_scenes (
            id TEXT PRIMARY KEY,
            book_id TEXT NOT NULL,
            sequence_number INTEGER NOT NULL,
            title TEXT,
            summary TEXT,
            characters_present TEXT,
            key_events TEXT,
            conflict_type TEXT,
            emotional_tone TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (book_id) REFERENCES reference_books(id) ON DELETE CASCADE
        );

        -- 拆书功能索引
        CREATE INDEX IF NOT EXISTS idx_ref_books_hash ON reference_books(file_hash);
        CREATE INDEX IF NOT EXISTS idx_ref_books_status ON reference_books(analysis_status);
        CREATE INDEX IF NOT EXISTS idx_ref_characters_book ON reference_characters(book_id);
        CREATE INDEX IF NOT EXISTS idx_ref_scenes_book ON reference_scenes(book_id);
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

    // Migration 5: 创建变更追踪表 (v3.3.0)
    let change_track_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='change_tracks'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if change_track_tables.is_empty() {
        conn.execute(
            "CREATE TABLE change_tracks (
                id TEXT PRIMARY KEY,
                scene_id TEXT,
                chapter_id TEXT,
                version_id TEXT,
                author_id TEXT NOT NULL,
                author_name TEXT,
                change_type TEXT NOT NULL,
                from_pos INTEGER NOT NULL,
                to_pos INTEGER NOT NULL,
                content TEXT,
                status TEXT NOT NULL DEFAULT 'Pending',
                created_at TEXT NOT NULL,
                resolved_at TEXT,
                FOREIGN KEY (scene_id) REFERENCES scenes(id) ON DELETE CASCADE,
                FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE CASCADE,
                FOREIGN KEY (version_id) REFERENCES scene_versions(id)
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_change_tracks_scene ON change_tracks(scene_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_change_tracks_chapter ON change_tracks(chapter_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_change_tracks_status ON change_tracks(status)",
            [],
        )?;
    }

    // Migration 5.1: 为旧版 change_tracks 添加 chapter_id (v3.3.0)
    let change_track_columns: Vec<String> = conn.prepare(
        "PRAGMA table_info(change_tracks)"
    )?.query_map([], |row| {
        let name: String = row.get(1)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if !change_track_columns.iter().any(|c| c == "chapter_id") {
        conn.execute(
            "ALTER TABLE change_tracks ADD COLUMN chapter_id TEXT REFERENCES chapters(id) ON DELETE CASCADE",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_change_tracks_chapter ON change_tracks(chapter_id)",
            [],
        )?;
    }

    // Migration 6: 创建评论线程表 (v3.3.0)
    let comment_thread_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='comment_threads'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if comment_thread_tables.is_empty() {
        conn.execute(
            "CREATE TABLE comment_threads (
                id TEXT PRIMARY KEY,
                scene_id TEXT,
                chapter_id TEXT,
                version_id TEXT,
                anchor_type TEXT NOT NULL,
                from_pos INTEGER,
                to_pos INTEGER,
                selected_text TEXT,
                status TEXT NOT NULL DEFAULT 'Open',
                created_at TEXT NOT NULL,
                resolved_at TEXT,
                FOREIGN KEY (scene_id) REFERENCES scenes(id) ON DELETE CASCADE,
                FOREIGN KEY (chapter_id) REFERENCES chapters(id) ON DELETE CASCADE,
                FOREIGN KEY (version_id) REFERENCES scene_versions(id)
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_comment_threads_scene ON comment_threads(scene_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_comment_threads_chapter ON comment_threads(chapter_id)",
            [],
        )?;
        conn.execute(
            "CREATE TABLE comment_messages (
                id TEXT PRIMARY KEY,
                thread_id TEXT NOT NULL,
                author_id TEXT NOT NULL,
                author_name TEXT,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (thread_id) REFERENCES comment_threads(id) ON DELETE CASCADE
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_comment_messages_thread ON comment_messages(thread_id)",
            [],
        )?;
    }

    // Migration 7: 创建角色状态追踪表 (v4.0 - 智能化创作)
    let character_state_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='character_states'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if character_state_tables.is_empty() {
        conn.execute(
            "CREATE TABLE character_states (
                id TEXT PRIMARY KEY,
                story_id TEXT NOT NULL,
                character_id TEXT NOT NULL,
                current_location TEXT,
                current_emotion TEXT,
                active_goal TEXT,
                secrets_known TEXT,
                secrets_unknown TEXT,
                arc_progress REAL,
                last_updated TEXT,
                FOREIGN KEY (character_id) REFERENCES characters(id)
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_character_states_story ON character_states(story_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_character_states_character ON character_states(character_id)",
            [],
        )?;
    }

    // Migration 8: 创建伏笔追踪表 (v4.0 - 智能化创作)
    let foreshadowing_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='foreshadowing_tracker'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if foreshadowing_tables.is_empty() {
        conn.execute(
            "CREATE TABLE foreshadowing_tracker (
                id TEXT PRIMARY KEY,
                story_id TEXT NOT NULL,
                content TEXT NOT NULL,
                setup_scene_id TEXT,
                payoff_scene_id TEXT,
                status TEXT NOT NULL DEFAULT 'setup',
                importance INTEGER,
                created_at TEXT NOT NULL,
                resolved_at TEXT
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_foreshadowing_story ON foreshadowing_tracker(story_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_foreshadowing_status ON foreshadowing_tracker(status)",
            [],
        )?;
    }

    // Migration 9: 创建用户偏好表 (v4.0 - 自适应学习)
    let preference_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='user_preferences'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if preference_tables.is_empty() {
        conn.execute(
            "CREATE TABLE user_preferences (
                id TEXT PRIMARY KEY,
                story_id TEXT NOT NULL,
                preference_type TEXT,
                preference_key TEXT,
                preference_value TEXT,
                confidence REAL,
                evidence_count INTEGER,
                updated_at TEXT
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_user_preferences_story ON user_preferences(story_id)",
            [],
        )?;
    }

    // Migration 10: 创建风格 DNA 表 (v4.0 - 深度风格系统)
    let style_dna_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='style_dnas'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if style_dna_tables.is_empty() {
        conn.execute(
            "CREATE TABLE style_dnas (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                author TEXT,
                dna_json TEXT NOT NULL,
                is_builtin INTEGER NOT NULL DEFAULT 0,
                is_user_created INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_style_dnas_builtin ON style_dnas(is_builtin)",
            [],
        )?;
    }

    // Migration 11: 创建用户反馈日志表 (v4.0 - 自适应学习)
    let feedback_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='user_feedback_log'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if feedback_tables.is_empty() {
        conn.execute(
            "CREATE TABLE user_feedback_log (
                id TEXT PRIMARY KEY,
                story_id TEXT NOT NULL,
                scene_id TEXT,
                chapter_id TEXT,
                feedback_type TEXT NOT NULL,    -- accept / reject / modify
                agent_type TEXT,                -- writer / inspector / etc
                original_ai_text TEXT,          -- AI 生成的原始文本
                final_text TEXT,                -- 用户最终接受的文本
                ai_score REAL,                  -- AI 自评分数
                user_satisfaction INTEGER,      -- 用户满意度 1-5（如提供）
                metadata TEXT,                  -- JSON: 额外上下文
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_feedback_story ON user_feedback_log(story_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_feedback_type ON user_feedback_log(feedback_type)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_feedback_created ON user_feedback_log(created_at)",
            [],
        )?;
    }

    // Migration 12: 创建订阅表 (v3.5.0 - Freemium 付费系统)
    let subscription_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='subscriptions'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if subscription_tables.is_empty() {
        conn.execute(
            "CREATE TABLE subscriptions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                tier TEXT NOT NULL DEFAULT 'free',
                status TEXT NOT NULL DEFAULT 'active',
                started_at TEXT NOT NULL,
                expires_at TEXT,
                payment_provider TEXT,
                payment_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_subscriptions_user ON subscriptions(user_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_subscriptions_tier ON subscriptions(tier)",
            [],
        )?;
    }

    // Migration 13: 创建 AI 使用配额表 (v3.5.0 - Freemium)
    let quota_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='ai_usage_quota'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if quota_tables.is_empty() {
        conn.execute(
            "CREATE TABLE ai_usage_quota (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                tier TEXT NOT NULL DEFAULT 'free',
                daily_limit INTEGER NOT NULL DEFAULT 10,
                daily_used INTEGER NOT NULL DEFAULT 0,
                quota_reset_at TEXT NOT NULL,
                total_used INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_quota_user ON ai_usage_quota(user_id)",
            [],
        )?;
    }

    // Migration 14: 创建 AI 调用日志表 (v3.5.0 - Freemium)
    let usage_log_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='ai_usage_logs'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if usage_log_tables.is_empty() {
        conn.execute(
            "CREATE TABLE ai_usage_logs (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                story_id TEXT,
                chapter_id TEXT,
                agent_type TEXT NOT NULL,
                instruction TEXT,
                prompt_tokens INTEGER,
                completion_tokens INTEGER,
                model_used TEXT,
                cost REAL,
                duration_ms INTEGER,
                tier_at_time TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_usage_logs_user ON ai_usage_logs(user_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_usage_logs_created ON ai_usage_logs(created_at)",
            [],
        )?;
    }

    // Migration 15: AI 使用配额表 V2 (v3.6.0 - 文思泉涌)
    // 添加按功能区分和字数限制的新字段
    let quota_columns: Vec<String> = conn.prepare(
        "PRAGMA table_info(ai_usage_quota)"
    )?.query_map([], |row| {
        let name: String = row.get(1)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if !quota_columns.iter().any(|c| c == "auto_write_used") {
        conn.execute(
            "ALTER TABLE ai_usage_quota ADD COLUMN auto_write_used INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    if !quota_columns.iter().any(|c| c == "auto_write_limit") {
        conn.execute(
            "ALTER TABLE ai_usage_quota ADD COLUMN auto_write_limit INTEGER NOT NULL DEFAULT 10",
            [],
        )?;
    }
    if !quota_columns.iter().any(|c| c == "auto_revise_used") {
        conn.execute(
            "ALTER TABLE ai_usage_quota ADD COLUMN auto_revise_used INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    if !quota_columns.iter().any(|c| c == "auto_revise_limit") {
        conn.execute(
            "ALTER TABLE ai_usage_quota ADD COLUMN auto_revise_limit INTEGER NOT NULL DEFAULT 10",
            [],
        )?;
    }
    if !quota_columns.iter().any(|c| c == "max_chars_per_call") {
        conn.execute(
            "ALTER TABLE ai_usage_quota ADD COLUMN max_chars_per_call INTEGER NOT NULL DEFAULT 1000",
            [],
        )?;
    }

    // Migration 16: 创建拆书功能参考表 (v3.5.0)
    let ref_book_tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='reference_books'"
    )?.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<_>, _>>()?;

    if ref_book_tables.is_empty() {
        conn.execute(
            "CREATE TABLE reference_books (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                author TEXT,
                genre TEXT,
                word_count INTEGER,
                file_format TEXT,
                file_hash TEXT UNIQUE,
                file_path TEXT,
                world_setting TEXT,
                plot_summary TEXT,
                story_arc TEXT,
                analysis_status TEXT NOT NULL DEFAULT 'pending',
                analysis_progress INTEGER DEFAULT 0,
                analysis_error TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_ref_books_hash ON reference_books(file_hash)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_ref_books_status ON reference_books(analysis_status)",
            [],
        )?;
        conn.execute(
            "CREATE TABLE reference_characters (
                id TEXT PRIMARY KEY,
                book_id TEXT NOT NULL,
                name TEXT NOT NULL,
                role_type TEXT,
                personality TEXT,
                appearance TEXT,
                relationships TEXT,
                key_scenes TEXT,
                importance_score REAL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (book_id) REFERENCES reference_books(id) ON DELETE CASCADE
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_ref_characters_book ON reference_characters(book_id)",
            [],
        )?;
        conn.execute(
            "CREATE TABLE reference_scenes (
                id TEXT PRIMARY KEY,
                book_id TEXT NOT NULL,
                sequence_number INTEGER NOT NULL,
                title TEXT,
                summary TEXT,
                characters_present TEXT,
                key_events TEXT,
                conflict_type TEXT,
                emotional_tone TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (book_id) REFERENCES reference_books(id) ON DELETE CASCADE
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_ref_scenes_book ON reference_scenes(book_id)",
            [],
        )?;
    }

    Ok(())
}
