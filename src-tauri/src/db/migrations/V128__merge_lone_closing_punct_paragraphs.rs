use rusqlite::{params, Connection};

use crate::{db::migrations::RustMigration, utils::text::TextUtils};

pub struct Migration;

impl RustMigration for Migration {
    fn version(&self) -> i32 {
        128
    }

    fn description(&self) -> &'static str {
        "merge lone closing-punct paragraphs in scene content"
    }

    /// 存量修复：此前前端 naive 的 `\n` -> `</p><p>` 转换把 LLM 软换行
    /// 单独成行的闭合引号（" ' 」 』 等）变成了 `<p>"</p>` 孤闭合标字段，
    /// 并以 HTML 形态存进 scenes.content。对所有含 `<p>` 的 scene 跑
    /// `TextUtils::merge_lone_closing_punct_paragraphs`（与前端
    /// `format.ts::mergeLoneClosingPunctParagraphs` 同规则），有变化才 UPDATE。
    /// 幂等：合并后不再存在孤闭合标字段，重跑无匹配；空库 no-op。
    fn apply(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        let tx = conn.transaction()?;
        let rows: Vec<(String, String)> = {
            let mut stmt = tx.prepare(
                "SELECT id, COALESCE(content, '') FROM scenes WHERE content LIKE '%</p>%'",
            )?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            out
        };

        let mut changed = 0usize;
        for (id, content) in rows {
            let merged = TextUtils::merge_lone_closing_punct_paragraphs(&content);
            if merged != content {
                tx.execute(
                    "UPDATE scenes SET content = ?1 WHERE id = ?2",
                    params![merged, id],
                )?;
                changed += 1;
            }
        }

        tx.commit()?;
        if changed > 0 {
            log::info!(
                "[V128] merged lone closing-punct paragraphs in {} scenes",
                changed
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::*;

    fn insert_scene(conn: &Connection, id: &str, story_id: &str, seq: i32, content: &str) {
        let now = Local::now().to_rfc3339();
        conn.execute(
            "INSERT INTO scenes (id, story_id, sequence_number, title, content,
             characters_present, character_conflicts, execution_stage, chapter_id,
             created_at, updated_at)
             VALUES (?1, ?2, ?3, '场景', ?4, '[]', '[]', 'drafting', NULL, ?5, ?5)",
            params![id, story_id, seq, content, now],
        )
        .unwrap();
    }

    fn scene_content(conn: &Connection, id: &str) -> String {
        conn.query_row(
            "SELECT COALESCE(content, '') FROM scenes WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .unwrap()
    }

    #[test]
    fn v128_merges_lone_closing_punct_paragraphs() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let mut conn = pool.get().unwrap();
        let now = Local::now().to_rfc3339();
        conn.execute(
            "INSERT INTO stories (id, title, created_at, updated_at) VALUES ('s1', '测试', ?1, ?1)",
            [&now],
        )
        .unwrap();

        insert_scene(&conn, "sc-bad", "s1", 1, "<p>他控制着局面'。</p><p>\"</p>");
        insert_scene(&conn, "sc-entity", "s1", 2, "<p>段落。</p><p>&rdquo;</p>");
        insert_scene(
            &conn,
            "sc-good",
            "s1",
            3,
            "<p>正常段落。</p><p>另一段。</p>",
        );
        insert_scene(&conn, "sc-plain", "s1", 4, "纯文本内容\n没有段落标签");

        Migration.apply(&mut conn).unwrap();

        assert_eq!(scene_content(&conn, "sc-bad"), "<p>他控制着局面'。\"</p>");
        assert_eq!(scene_content(&conn, "sc-entity"), "<p>段落。&rdquo;</p>");
        // 无孤闭合标字段的内容不动
        assert_eq!(
            scene_content(&conn, "sc-good"),
            "<p>正常段落。</p><p>另一段。</p>"
        );
        // 无 <p> 的内容不动
        assert_eq!(scene_content(&conn, "sc-plain"), "纯文本内容\n没有段落标签");

        // 幂等：重跑后逐字段对比相等
        let snapshot = |c: &Connection| {
            c.prepare("SELECT id, COALESCE(content, '') FROM scenes ORDER BY id")
                .unwrap()
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        };
        let before = snapshot(&conn);
        Migration.apply(&mut conn).unwrap();
        assert_eq!(snapshot(&conn), before, "重跑 apply 不应改动任何场景");
    }

    #[test]
    fn v128_empty_db_is_noop() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let mut conn = pool.get().unwrap();
        Migration.apply(&mut conn).unwrap();
    }
}
