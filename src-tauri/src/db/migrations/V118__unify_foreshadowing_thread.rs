use rusqlite::Connection;

use crate::db::migrations::RustMigration;

pub struct Migration;

impl RustMigration for Migration {
    fn version(&self) -> i32 {
        118
    }

    fn description(&self) -> &'static str {
        "unify foreshadowing thread model"
    }

    fn apply(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        // 统一模型以 foreshadowing_tracker 为单一物理真源。
        // narrative_threads 表已在 V083 中删除；本迁移仅做 schema 完整性守卫
        // 并提供向后兼容视图，使仍按 narrative_threads 读取的代码不会崩溃。
        Self::ensure_columns(conn)?;
        Self::ensure_view(conn)?;
        Ok(())
    }
}

impl Migration {
    /// 反向迁移：删除兼容视图，并移除 apply() 中添加的列，恢复
    /// foreshadowing_tracker 到 V118 之前的状态。
    ///
    /// 注：当前 migration runner 不自动调用
    /// down，但本方法保留供测试/手动回滚使用。
    pub fn down(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        conn.execute("DROP VIEW IF EXISTS v_narrative_threads", [])?;

        // Columns added by ensure_columns (V118).
        let columns_to_drop: &[&str] = &[
            "setup_event_id",
            "payoff_event_id",
            "risk_signals_score",
            "target_start_scene",
            "target_end_scene",
            "risk_signals",
            "scope_type",
            "ledger_key",
        ];

        #[derive(Debug)]
        struct ColumnInfo {
            name: String,
            type_name: String,
            not_null: bool,
            default_value: Option<String>,
            pk: bool,
        }

        let cols: Vec<ColumnInfo> = conn
            .prepare("PRAGMA table_info(foreshadowing_tracker)")?
            .query_map([], |row| {
                Ok(ColumnInfo {
                    name: row.get(1)?,
                    type_name: row.get(2)?,
                    not_null: row.get::<_, i32>(3)? != 0,
                    default_value: row.get(4)?,
                    pk: row.get::<_, i32>(5)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let keep_cols: Vec<&ColumnInfo> = cols
            .iter()
            .filter(|c| !columns_to_drop.contains(&c.name.as_str()))
            .collect();

        // Only recreate the table if any V118 columns are actually present.
        if keep_cols.len() < cols.len() {
            let col_defs: Vec<String> = keep_cols
                .iter()
                .map(|c| {
                    let mut def = format!("{} {}", c.name, c.type_name);
                    if c.pk {
                        def.push_str(" PRIMARY KEY");
                    }
                    if c.not_null {
                        def.push_str(" NOT NULL");
                    }
                    if let Some(d) = &c.default_value {
                        def.push_str(&format!(" DEFAULT {}", d));
                    }
                    def
                })
                .collect();
            let col_names: Vec<String> = keep_cols.iter().map(|c| c.name.clone()).collect();
            let col_def_list = col_defs.join(", ");
            let col_name_list = col_names.join(", ");

            // Rename/recreate pattern for SQLite versions that do not support
            // ALTER TABLE ... DROP COLUMN.
            conn.execute(
                &format!("CREATE TABLE foreshadowing_tracker_new ({})", col_def_list),
                [],
            )?;
            conn.execute(
                &format!(
                    "INSERT INTO foreshadowing_tracker_new ({}) SELECT {} FROM foreshadowing_tracker",
                    col_name_list, col_name_list
                ),
                [],
            )?;
            conn.execute("DROP TABLE foreshadowing_tracker", [])?;
            conn.execute(
                "ALTER TABLE foreshadowing_tracker_new RENAME TO foreshadowing_tracker",
                [],
            )?;
        }

        Ok(())
    }

    fn ensure_columns(conn: &mut Connection) -> Result<(), rusqlite::Error> {
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(foreshadowing_tracker)")?
            .query_map([], |row| {
                let name: String = row.get(1)?;
                Ok(name)
            })?
            .collect::<Result<Vec<_>, _>>()?;

        if !cols.contains(&"setup_event_id".to_string()) {
            conn.execute(
                "ALTER TABLE foreshadowing_tracker ADD COLUMN setup_event_id TEXT",
                [],
            )?;
        }
        if !cols.contains(&"payoff_event_id".to_string()) {
            conn.execute(
                "ALTER TABLE foreshadowing_tracker ADD COLUMN payoff_event_id TEXT",
                [],
            )?;
        }
        if !cols.contains(&"risk_signals_score".to_string()) {
            conn.execute(
                "ALTER TABLE foreshadowing_tracker ADD COLUMN risk_signals_score REAL DEFAULT 0.0",
                [],
            )?;
        }
        if !cols.contains(&"target_start_scene".to_string()) {
            conn.execute(
                "ALTER TABLE foreshadowing_tracker ADD COLUMN target_start_scene INTEGER",
                [],
            )?;
        }
        if !cols.contains(&"target_end_scene".to_string()) {
            conn.execute(
                "ALTER TABLE foreshadowing_tracker ADD COLUMN target_end_scene INTEGER",
                [],
            )?;
        }
        if !cols.contains(&"risk_signals".to_string()) {
            conn.execute(
                "ALTER TABLE foreshadowing_tracker ADD COLUMN risk_signals TEXT",
                [],
            )?;
        }
        if !cols.contains(&"scope_type".to_string()) {
            conn.execute(
                "ALTER TABLE foreshadowing_tracker ADD COLUMN scope_type TEXT DEFAULT 'story'",
                [],
            )?;
        }
        if !cols.contains(&"ledger_key".to_string()) {
            conn.execute(
                "ALTER TABLE foreshadowing_tracker ADD COLUMN ledger_key TEXT",
                [],
            )?;
        }

        Ok(())
    }

    fn ensure_view(conn: &mut Connection) -> Result<(), rusqlite::Error> {
        conn.execute(
            "CREATE VIEW IF NOT EXISTS v_narrative_threads AS
             SELECT
                 id,
                 story_id,
                 'foreshadow' AS thread_type,
                 content AS thread_data,
                 setup_event_id AS target_id,
                 status,
                 importance,
                 created_at
             FROM foreshadowing_tracker",
            [],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;

    #[test]
    fn v118_applies_and_reverses_cleanly() {
        let mut conn = Connection::open_in_memory().unwrap();
        // Minimal schema simulating a pre-V118 database.
        conn.execute_batch(
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
            );
            CREATE TABLE schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at INTEGER NOT NULL
            );",
        )
        .unwrap();

        // Seed a pre-V118 row to verify data survives down().
        conn.execute(
            "INSERT INTO foreshadowing_tracker (id, story_id, content, status, created_at) \
             VALUES (?1, ?2, ?3, 'setup', ?4)",
            ["fs-1", "story-1", "test-fs", "2024-01-01T00:00:00Z"],
        )
        .unwrap();

        let migration = Migration;
        migration.apply(&mut conn).unwrap();

        // Verify columns were added.
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(foreshadowing_tracker)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(cols.contains(&"setup_event_id".to_string()));
        assert!(cols.contains(&"ledger_key".to_string()));

        // Verify view exists.
        let view_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='view' AND name='v_narrative_threads'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(view_count, 1);

        // Reverse.
        migration.down(&mut conn).unwrap();
        let view_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='view' AND name='v_narrative_threads'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(view_count, 0);

        // V118 columns are removed after reverse.
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(foreshadowing_tracker)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(!cols.contains(&"setup_event_id".to_string()));
        assert!(!cols.contains(&"payoff_event_id".to_string()));
        assert!(!cols.contains(&"risk_signals_score".to_string()));
        assert!(!cols.contains(&"target_start_scene".to_string()));
        assert!(!cols.contains(&"target_end_scene".to_string()));
        assert!(!cols.contains(&"risk_signals".to_string()));
        assert!(!cols.contains(&"scope_type".to_string()));
        assert!(!cols.contains(&"ledger_key".to_string()));

        // Pre-existing data is preserved.
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM foreshadowing_tracker WHERE content = ?1",
                ["test-fs"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
