use rusqlite::Connection;

use crate::db::migrations::RustMigration;

/// 将遗留的 chapters.content 迁移到关联 scenes.content，然后删除
/// chapters.content 列。
pub struct Migration;

impl RustMigration for Migration {
    fn version(&self) -> i32 {
        115
    }

    fn description(&self) -> &'static str {
        "drop chapter content"
    }

    fn apply(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        // 1. 将仍存在于 chapters.content 的内容回填到关联 Scene（仅当 Scene
        //    内容为空时）。 Scene 是章节内容的唯一真相源。
        conn.execute(
            r#"UPDATE scenes
               SET content = COALESCE(content, (
                   SELECT chapters.content FROM chapters
                   WHERE chapters.id = scenes.chapter_id
                     AND chapters.content IS NOT NULL
                     AND chapters.content != ''
               ))
               WHERE content IS NULL OR content = '';"#,
            [],
        )?;

        // 2. 删除 chapters 表的 content 列（如仍存在）。
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(chapters)")?
            .query_map([], |row| {
                let name: String = row.get(1)?;
                Ok(name)
            })?
            .collect::<Result<Vec<_>, _>>()?;

        if columns.contains(&"content".to_string()) {
            conn.execute("ALTER TABLE chapters DROP COLUMN content", [])?;
        }

        Ok(())
    }
}
