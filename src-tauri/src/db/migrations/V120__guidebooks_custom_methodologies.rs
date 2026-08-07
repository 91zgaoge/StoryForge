use rusqlite::Connection;

use crate::db::migrations::RustMigration;

pub struct Migration;

impl RustMigration for Migration {
    fn version(&self) -> i32 {
        120
    }

    fn description(&self) -> &'static str {
        "guidebooks and custom methodologies"
    }

    fn apply(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS guidebooks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL DEFAULT '未命名',
                author TEXT,
                subject TEXT,
                word_count INTEGER,
                file_format TEXT,
                file_hash TEXT UNIQUE,
                file_path TEXT,
                methodology_id TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                progress INTEGER NOT NULL DEFAULT 0,
                error TEXT,
                task_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS custom_methodologies (
                id TEXT PRIMARY KEY,
                guidebook_id TEXT REFERENCES guidebooks(id) ON DELETE SET NULL,
                name TEXT NOT NULL,
                description TEXT,
                steps_json TEXT NOT NULL DEFAULT '[]',
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_custom_methodologies_guidebook
                ON custom_methodologies(guidebook_id);",
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::db::migrations::RustMigration;

    #[test]
    fn v120_creates_tables_idempotent() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let mut conn = pool.get().unwrap();
        // create_test_pool 已跑全部迁移；再次 apply 验证幂等
        super::Migration
            .apply(&mut conn)
            .expect("re-apply should be idempotent");
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' \
                 AND name IN ('guidebooks','custom_methodologies') ORDER BY name",
            )
            .unwrap();
        let names: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(names, vec!["custom_methodologies", "guidebooks"]);
    }
}
