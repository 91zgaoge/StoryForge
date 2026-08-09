use chrono::Local;
use rusqlite::{params, Connection};

use crate::db::migrations::RustMigration;

pub struct Migration;

impl RustMigration for Migration {
    fn version(&self) -> i32 {
        121
    }

    fn description(&self) -> &'static str {
        "repair duplicate chapter titles from auto-split naming bug"
    }

    /// v0.33.7 存量修复：自动分章命名 bug（取"最新合约 goal"作为所有新章
    /// 标题）导致一次切分出的多个章节共用同一标题。将同一故事内标题重复
    /// （≥2 章同名）的章节一律回退为 `第{chapter_number}章`，并同步关联
    /// scenes 的共享标题。幂等：修复后同故事同名章节不再重复，重跑无匹配。
    fn apply(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        let now = Local::now().to_rfc3339();
        let tx = conn.transaction()?;

        // 先取受影响章节（同故事内同名 ≥2 个），再逐章修复，避免 UPDATE 后
        // 丢失"哪些是重复标题"的判定依据。
        let affected: Vec<(String, i32)> = {
            let mut stmt = tx.prepare(
                "SELECT id, chapter_number FROM chapters
                 WHERE title IS NOT NULL AND TRIM(title) <> ''
                   AND (story_id, title) IN (
                       SELECT story_id, title FROM chapters
                       WHERE title IS NOT NULL AND TRIM(title) <> ''
                       GROUP BY story_id, title HAVING COUNT(*) > 1
                   )",
            )?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            out
        };

        for (chapter_id, chapter_number) in &affected {
            let new_title = format!("第{}章", chapter_number);
            tx.execute(
                "UPDATE chapters SET title = ?2, updated_at = ?3 WHERE id = ?1",
                params![chapter_id, new_title, now],
            )?;
            // 标题是 chapter↔scene 共享元数据，同步关联场景
            tx.execute(
                "UPDATE scenes SET title = ?2, updated_at = ?3 WHERE chapter_id = ?1",
                params![chapter_id, new_title, now],
            )?;
        }

        tx.commit()?;
        if !affected.is_empty() {
            log::info!(
                "[V121] repaired duplicate chapter titles: {} chapters",
                affected.len()
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::RustMigration;

    fn insert_chapter(conn: &Connection, id: &str, story_id: &str, number: i32, title: &str) {
        let now = Local::now().to_rfc3339();
        conn.execute(
            "INSERT INTO chapters (id, story_id, chapter_number, title, outline, word_count,
             model_used, cost, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, NULL, 0, NULL, 0.0, ?5, ?5)",
            params![id, story_id, number, title, now],
        )
        .unwrap();
    }

    fn chapter_title(conn: &Connection, id: &str) -> String {
        conn.query_row("SELECT title FROM chapters WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .unwrap()
    }

    #[test]
    fn v121_repairs_duplicate_titles_and_keeps_unique_ones() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let mut conn = pool.get().unwrap();
        let now = Local::now().to_rfc3339();
        conn.execute(
            "INSERT INTO stories (id, title, created_at, updated_at) VALUES ('s1', '测试', ?1, ?1)",
            [&now],
        )
        .unwrap();

        // bug 现场：2-4 章共用同一合约 goal 标题；1 章与 5 章标题唯一
        insert_chapter(&conn, "c1", "s1", 1, "第一章");
        insert_chapter(&conn, "c2", "s1", 2, "雨夜灵堂父亲阴魂现身");
        insert_chapter(&conn, "c3", "s1", 3, "雨夜灵堂父亲阴魂现身");
        insert_chapter(&conn, "c4", "s1", 4, "雨夜灵堂父亲阴魂现身");
        insert_chapter(&conn, "c5", "s1", 5, "江边决战");

        Migration.apply(&mut conn).unwrap();

        assert_eq!(chapter_title(&conn, "c1"), "第一章");
        assert_eq!(chapter_title(&conn, "c2"), "第2章");
        assert_eq!(chapter_title(&conn, "c3"), "第3章");
        assert_eq!(chapter_title(&conn, "c4"), "第4章");
        assert_eq!(chapter_title(&conn, "c5"), "江边决战");

        // 幂等：重跑无匹配、无改动
        Migration.apply(&mut conn).unwrap();
        assert_eq!(chapter_title(&conn, "c2"), "第2章");
        assert_eq!(chapter_title(&conn, "c5"), "江边决战");
    }

    #[test]
    fn v121_same_title_across_different_stories_is_not_duplicate() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let mut conn = pool.get().unwrap();
        let now = Local::now().to_rfc3339();
        for sid in ["sa", "sb"] {
            conn.execute(
                "INSERT INTO stories (id, title, created_at, updated_at) VALUES (?1, '测试', ?2, ?2)",
                params![sid, now],
            )
            .unwrap();
        }
        insert_chapter(&conn, "ca1", "sa", 1, "序章");
        insert_chapter(&conn, "cb1", "sb", 1, "序章");

        Migration.apply(&mut conn).unwrap();

        assert_eq!(chapter_title(&conn, "ca1"), "序章");
        assert_eq!(chapter_title(&conn, "cb1"), "序章");
    }
}
