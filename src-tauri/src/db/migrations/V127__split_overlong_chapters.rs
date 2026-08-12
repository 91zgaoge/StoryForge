use rusqlite::{Connection, OptionalExtension};

use crate::{
    config::DEFAULT_CHAPTER_SPLIT_MAX_CHARS,
    db::migrations::RustMigration,
    story_system::chapter_splitter::{plan_split, split_chapter_in_tx, ChapterSplitMode},
    utils::text::TextUtils,
};

pub struct Migration;

impl RustMigration for Migration {
    fn version(&self) -> i32 {
        127
    }

    fn description(&self) -> &'static str {
        "split overlong chapters and renumber following chapters"
    }

    /// 存量修复：此前自动分章只允许切"最新章"，续写持续写入非最新章时
    /// 该章会无限增长（真实数据出现单章 19,492 字）。找出所有内容字数超过
    /// 3000 字（`TextUtils::chinese_word_count`，聚合其 scenes 内容）的章，
    /// 用与 `find_split_offset` 相同的逻辑（WordCount 模式，段落/句末边界）
    /// 循环切分 + 重排后续章号，直到每章不超过 3000 字。全程单事务，失败回滚。
    /// 幂等：修复后所有章均不超过 3000 字，重跑无匹配。
    ///
    /// 不依赖 AppConfig：固定默认阈值 3000 / WordCount 模式；新章标题回退
    /// `第{N}章`（迁移期不解析章节合约）。
    fn apply(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        const MAX_CHARS: usize = DEFAULT_CHAPTER_SPLIT_MAX_CHARS;
        const MAX_ITER: usize = 50; // 与 chapter_splitter::MAX_SPLIT_ITERATIONS 对齐

        let tx = conn.transaction()?;
        let chapters: Vec<(String, String)> = {
            let mut stmt =
                tx.prepare("SELECT id, story_id FROM chapters ORDER BY story_id, chapter_number")?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            out
        };

        let mut total_splits = 0usize;
        for (chapter_id, story_id) in chapters {
            let mut current_id = chapter_id;
            for _ in 0..MAX_ITER {
                let Some(number) = tx
                    .query_row(
                        "SELECT chapter_number FROM chapters WHERE id = ?1",
                        [&current_id],
                        |r| r.get::<_, i32>(0),
                    )
                    .optional()?
                else {
                    break;
                };
                let content: String = {
                    let mut stmt = tx.prepare(
                        "SELECT COALESCE(content, '') FROM scenes WHERE chapter_id = ?1 \
                         ORDER BY sequence_number",
                    )?;
                    let rows = stmt.query_map([&current_id], |r| r.get::<_, String>(0))?;
                    let mut parts = Vec::new();
                    for row in rows {
                        parts.push(row?);
                    }
                    parts.concat()
                };
                if TextUtils::chinese_word_count(&content) <= MAX_CHARS {
                    break;
                }
                let Some(plan) = plan_split(&content, ChapterSplitMode::WordCount, MAX_CHARS)
                else {
                    // 找不到切分点（无边界且 keep 为 0 字等）：放弃该章
                    log::warn!(
                        "[V127] chapter {} (story {}) 超阈值但找不到切分点，跳过",
                        current_id,
                        story_id
                    );
                    break;
                };
                let Some(scene_id) = tx
                    .query_row(
                        "SELECT id FROM scenes WHERE chapter_id = ?1 \
                         ORDER BY sequence_number LIMIT 1",
                        [&current_id],
                        |r| r.get::<_, String>(0),
                    )
                    .optional()?
                else {
                    break;
                };
                let new_title = format!("第{}章", number + 1);
                let (new_id, _) = split_chapter_in_tx(
                    &tx,
                    &story_id,
                    &current_id,
                    number,
                    &plan,
                    &new_title,
                    &scene_id,
                )?;
                total_splits += 1;
                current_id = new_id;
            }
        }

        tx.commit()?;
        if total_splits > 0 {
            log::info!("[V127] split overlong chapters: {} splits", total_splits);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use rusqlite::params;

    use super::*;
    use crate::utils::text::TextUtils;

    fn insert_chapter_with_content(
        conn: &Connection,
        id: &str,
        story_id: &str,
        number: i32,
        title: &str,
        content: &str,
    ) {
        let now = Local::now().to_rfc3339();
        conn.execute(
            "INSERT INTO chapters (id, story_id, chapter_number, title, outline, word_count,
             model_used, cost, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, NULL, 0, NULL, 0.0, ?5, ?5)",
            params![id, story_id, number, title, now],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO scenes (id, story_id, sequence_number, title, content,
             characters_present, character_conflicts, execution_stage, chapter_id,
             created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, '[]', '[]', 'drafting', ?6, ?7, ?7)",
            params![
                format!("sc-{}", id),
                story_id,
                number,
                title,
                content,
                id,
                now
            ],
        )
        .unwrap();
    }

    fn chapter_numbers(conn: &Connection, story_id: &str) -> Vec<(String, i32)> {
        conn.prepare(
            "SELECT id, chapter_number FROM chapters WHERE story_id = ?1 ORDER BY chapter_number",
        )
        .unwrap()
        .query_map([story_id], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
    }

    fn chapter_content(conn: &Connection, chapter_id: &str) -> String {
        conn.prepare(
            "SELECT COALESCE(content, '') FROM scenes WHERE chapter_id = ?1 ORDER BY sequence_number",
        )
        .unwrap()
        .query_map([chapter_id], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .concat()
    }

    #[test]
    fn v127_splits_overlong_chapter_and_renumbers_followers() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let mut conn = pool.get().unwrap();
        let now = Local::now().to_rfc3339();
        conn.execute(
            "INSERT INTO stories (id, title, created_at, updated_at) VALUES ('s1', '测试', ?1, ?1)",
            [&now],
        )
        .unwrap();

        // 真实事故现场：单章 19,492 字（10 段×1800 + 尾段 1492），其后还有一章
        let para = "文".repeat(1800);
        let tail = "尾".repeat(1492);
        let mut parts: Vec<&str> = vec![para.as_str(); 10];
        parts.push(tail.as_str());
        let big = parts.join("\n\n");
        assert_eq!(TextUtils::chinese_word_count(&big), 19492);
        insert_chapter_with_content(&conn, "c1", "s1", 1, "第1章", &big);
        insert_chapter_with_content(&conn, "c2", "s1", 2, "第2章", "后续章内容");

        Migration.apply(&mut conn).unwrap();

        let chapters = chapter_numbers(&conn, "s1");
        assert!(
            chapters.len() >= 8,
            "19492 字应切出多章, got {}",
            chapters.len()
        );

        // 每章 ≤ 3000 字
        for (cid, number) in &chapters {
            let wc = TextUtils::chinese_word_count(&chapter_content(&conn, cid));
            assert!(wc <= 3000, "chapter {} exceeds threshold: {}", number, wc);
        }

        // 旧第2章顺延到队尾，章号连续 1..=N
        let c2_number = chapters.iter().find(|(id, _)| id == "c2").unwrap().1;
        assert_eq!(c2_number, chapters.len() as i32);
        let numbers: Vec<i32> = chapters.iter().map(|(_, n)| *n).collect();
        assert_eq!(numbers, (1..=chapters.len() as i32).collect::<Vec<_>>());

        // scenes.sequence_number 与章号对齐
        let scene_seqs: Vec<i32> = conn
            .prepare(
                "SELECT sequence_number FROM scenes WHERE story_id = 's1' ORDER BY sequence_number",
            )
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(scene_seqs, (1..=chapters.len() as i32).collect::<Vec<_>>());

        // 幂等：重跑无变化
        let before = chapter_numbers(&conn, "s1");
        Migration.apply(&mut conn).unwrap();
        assert_eq!(chapter_numbers(&conn, "s1"), before);
        assert_eq!(
            TextUtils::chinese_word_count(&chapter_content(&conn, "c1")),
            TextUtils::chinese_word_count(&chapter_content(&conn, "c1"))
        );
    }

    #[test]
    fn v127_leaves_within_threshold_chapters_untouched() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let mut conn = pool.get().unwrap();
        let now = Local::now().to_rfc3339();
        conn.execute(
            "INSERT INTO stories (id, title, created_at, updated_at) VALUES ('s2', '测试', ?1, ?1)",
            [&now],
        )
        .unwrap();
        insert_chapter_with_content(&conn, "c1", "s2", 1, "第1章", &"短".repeat(1000));
        insert_chapter_with_content(&conn, "c2", "s2", 2, "第2章", &"章".repeat(2000));

        Migration.apply(&mut conn).unwrap();

        assert_eq!(
            chapter_numbers(&conn, "s2"),
            vec![("c1".to_string(), 1), ("c2".to_string(), 2)]
        );
        assert_eq!(chapter_content(&conn, "c1"), "短".repeat(1000));
    }
}
