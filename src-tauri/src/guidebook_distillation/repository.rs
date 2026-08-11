//! 指导书与自定义方法论 Repository

use chrono::{DateTime, Local};
use rusqlite::params;

use super::models::*;
use crate::db::DbPool;

type RepoResult<T> = Result<T, Box<dyn std::error::Error>>;

// ==================== 指导书 ====================

pub struct GuidebookRepository {
    pool: DbPool,
}

impl GuidebookRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, book: &Guidebook) -> RepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO guidebooks (id, title, author, subject, word_count, file_format, \
             file_hash, file_path, methodology_id, status, progress, error, task_id, \
             merge_into_methodology_id, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
            params![
                book.id,
                book.title,
                book.author,
                book.subject,
                book.word_count,
                book.file_format,
                book.file_hash,
                book.file_path,
                book.methodology_id,
                book.status.to_string(),
                book.progress,
                book.error,
                book.task_id,
                book.merge_into_methodology_id,
                book.created_at.to_rfc3339(),
                book.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn row_to_guidebook(row: &rusqlite::Row) -> rusqlite::Result<Guidebook> {
        let status_str: String = row.get("status")?;
        let created: String = row.get("created_at")?;
        let updated: String = row.get("updated_at")?;
        Ok(Guidebook {
            id: row.get("id")?,
            title: row.get("title")?,
            author: row.get("author")?,
            subject: row.get("subject")?,
            word_count: row.get("word_count")?,
            file_format: row.get("file_format")?,
            file_hash: row.get("file_hash")?,
            file_path: row.get("file_path")?,
            methodology_id: row.get("methodology_id")?,
            status: status_str.parse().unwrap_or(DistillationStatus::Pending),
            progress: row.get("progress")?,
            error: row.get("error")?,
            task_id: row.get("task_id")?,
            merge_into_methodology_id: row.get("merge_into_methodology_id")?,
            created_at: DateTime::parse_from_rfc3339(&created)
                .map(|d| d.with_timezone(&Local))
                .unwrap_or_else(|_| Local::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated)
                .map(|d| d.with_timezone(&Local))
                .unwrap_or_else(|_| Local::now()),
        })
    }

    pub fn get_by_id(&self, id: &str) -> RepoResult<Option<Guidebook>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM guidebooks WHERE id = ?1")?;
        let mut rows = stmt.query_map(params![id], Self::row_to_guidebook)?;
        Ok(rows.next().transpose()?)
    }

    pub fn get_by_hash(&self, hash: &str) -> RepoResult<Option<Guidebook>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM guidebooks WHERE file_hash = ?1")?;
        let mut rows = stmt.query_map(params![hash], Self::row_to_guidebook)?;
        Ok(rows.next().transpose()?)
    }

    pub fn list_all(&self) -> RepoResult<Vec<GuidebookListItem>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, author, subject, word_count, file_format, methodology_id, \
             merge_into_methodology_id, status, progress, created_at FROM guidebooks ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(GuidebookListItem {
                id: row.get("id")?,
                title: row.get("title")?,
                author: row.get("author")?,
                subject: row.get("subject")?,
                word_count: row.get("word_count")?,
                file_format: row.get("file_format")?,
                methodology_id: row.get("methodology_id")?,
                merge_into_methodology_id: row.get("merge_into_methodology_id")?,
                status: row.get("status")?,
                progress: row.get("progress")?,
                created_at: row.get("created_at")?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn update_status(
        &self,
        id: &str,
        status: DistillationStatus,
        progress: i32,
    ) -> RepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE guidebooks SET status = ?1, progress = ?2, updated_at = ?3 WHERE id = ?4",
            params![status.to_string(), progress, Local::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn update_task_id(&self, id: &str, task_id: &str) -> RepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE guidebooks SET task_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![task_id, Local::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn update_error(&self, id: &str, error: &str) -> RepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE guidebooks SET status = 'failed', error = ?1, updated_at = ?2 WHERE id = ?3",
            params![error, Local::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// 重试前重置：清错误、回到 pending、进度归零
    pub fn reset_for_retry(&self, id: &str) -> RepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE guidebooks SET status = 'pending', progress = 0, error = NULL, \
             updated_at = ?1 WHERE id = ?2",
            params![Local::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// 提炼完成后回写元信息与产物关联
    pub fn update_distilled(
        &self,
        id: &str,
        title: Option<&str>,
        author: Option<&str>,
        subject: Option<&str>,
        methodology_id: &str,
    ) -> RepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE guidebooks SET title = COALESCE(?1, title), author = COALESCE(?2, author), \
             subject = COALESCE(?3, subject), methodology_id = ?4, updated_at = ?5 WHERE id = ?6",
            params![
                title,
                author,
                subject,
                methodology_id,
                Local::now().to_rfc3339(),
                id
            ],
        )?;
        Ok(())
    }

    /// 回写合并意图（dedup 命中既有记录且旧记录未落 merge_into 时补写）
    pub fn set_merge_into(&self, id: &str, methodology_id: &str) -> RepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE guidebooks SET merge_into_methodology_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![methodology_id, Local::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> RepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM guidebooks WHERE id = ?1", params![id])?;
        Ok(())
    }
}

// ==================== 自定义方法论 ====================

pub struct CustomMethodologyRepository {
    pool: DbPool,
}

impl CustomMethodologyRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, cm: &CustomMethodology) -> RepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO custom_methodologies (id, guidebook_id, name, description, steps_json, \
             patterns_json, cheatsheet_json, enabled, created_at, updated_at) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                cm.id,
                cm.guidebook_id,
                cm.name,
                cm.description,
                serde_json::to_string(&cm.steps)?,
                serde_json::to_string(&cm.patterns)?,
                serde_json::to_string(&cm.cheatsheet)?,
                cm.enabled as i32,
                cm.created_at.to_rfc3339(),
                cm.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn row_to_cm(row: &rusqlite::Row) -> rusqlite::Result<CustomMethodology> {
        let steps_json: String = row.get("steps_json")?;
        let patterns_json: String = row.get("patterns_json")?;
        let cheatsheet_json: String = row.get("cheatsheet_json")?;
        let created: String = row.get("created_at")?;
        let updated: String = row.get("updated_at")?;
        let enabled: i32 = row.get("enabled")?;
        Ok(CustomMethodology {
            id: row.get("id")?,
            guidebook_id: row.get("guidebook_id")?,
            name: row.get("name")?,
            description: row.get("description")?,
            steps: parse_steps(&steps_json),
            patterns: parse_patterns(&patterns_json),
            cheatsheet: parse_cheatsheet(&cheatsheet_json),
            enabled: enabled != 0,
            created_at: DateTime::parse_from_rfc3339(&created)
                .map(|d| d.with_timezone(&Local))
                .unwrap_or_else(|_| Local::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated)
                .map(|d| d.with_timezone(&Local))
                .unwrap_or_else(|_| Local::now()),
        })
    }

    pub fn get_by_id(&self, id: &str) -> RepoResult<Option<CustomMethodology>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM custom_methodologies WHERE id = ?1")?;
        let mut rows = stmt.query_map(params![id], Self::row_to_cm)?;
        Ok(rows.next().transpose()?)
    }

    pub fn list_all(&self) -> RepoResult<Vec<CustomMethodology>> {
        let conn = self.pool.get()?;
        let mut stmt =
            conn.prepare("SELECT * FROM custom_methodologies ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], Self::row_to_cm)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn list_enabled(&self) -> RepoResult<Vec<CustomMethodology>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT * FROM custom_methodologies WHERE enabled = 1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], Self::row_to_cm)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// 更新名称/描述/步骤/启用状态/技巧模式库/决策速查（None 字段不动）
    pub fn update(
        &self,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
        steps: Option<&[MethodologyStep]>,
        enabled: Option<bool>,
        patterns: Option<&[Technique]>,
        cheatsheet: Option<&Cheatsheet>,
    ) -> RepoResult<()> {
        let conn = self.pool.get()?;
        if let Some(n) = name {
            conn.execute(
                "UPDATE custom_methodologies SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![n, Local::now().to_rfc3339(), id],
            )?;
        }
        if let Some(d) = description {
            conn.execute(
                "UPDATE custom_methodologies SET description = ?1, updated_at = ?2 WHERE id = ?3",
                params![d, Local::now().to_rfc3339(), id],
            )?;
        }
        if let Some(s) = steps {
            conn.execute(
                "UPDATE custom_methodologies SET steps_json = ?1, updated_at = ?2 WHERE id = ?3",
                params![serde_json::to_string(s)?, Local::now().to_rfc3339(), id],
            )?;
        }
        if let Some(e) = enabled {
            conn.execute(
                "UPDATE custom_methodologies SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
                params![e as i32, Local::now().to_rfc3339(), id],
            )?;
        }
        if let Some(p) = patterns {
            conn.execute(
                "UPDATE custom_methodologies SET patterns_json = ?1, updated_at = ?2 WHERE id = ?3",
                params![serde_json::to_string(p)?, Local::now().to_rfc3339(), id],
            )?;
        }
        if let Some(c) = cheatsheet {
            conn.execute(
                "UPDATE custom_methodologies SET cheatsheet_json = ?1, updated_at = ?2 WHERE id = ?3",
                params![serde_json::to_string(c)?, Local::now().to_rfc3339(), id],
            )?;
        }
        Ok(())
    }

    pub fn delete(&self, id: &str) -> RepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM custom_methodologies WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// 引用该方法论的故事数（删除前提示用）
    pub fn count_stories_using(&self, id: &str) -> RepoResult<i64> {
        let conn = self.pool.get()?;
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM stories WHERE methodology_id = ?1",
            params![id],
            |r| r.get(0),
        )?;
        Ok(n)
    }

    /// 删除方法论时把引用它的故事的 methodology_id 置空
    pub fn clear_story_references(&self, id: &str) -> RepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute(
            "UPDATE stories SET methodology_id = NULL, methodology_step = NULL \
             WHERE methodology_id = ?1",
            params![id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::create_test_pool;

    fn sample_guidebook(id: &str) -> Guidebook {
        Guidebook {
            id: id.into(),
            title: "故事".into(),
            author: None,
            subject: None,
            word_count: Some(1000),
            file_format: Some("txt".into()),
            file_hash: Some(format!("hash_{}", id)),
            file_path: None,
            methodology_id: None,
            status: DistillationStatus::Pending,
            progress: 0,
            error: None,
            task_id: None,
            merge_into_methodology_id: None,
            created_at: Local::now(),
            updated_at: Local::now(),
        }
    }

    #[test]
    fn guidebook_crud_flow() {
        let pool = create_test_pool().unwrap();
        let repo = GuidebookRepository::new(pool);
        repo.create(&sample_guidebook("g1")).unwrap();
        // hash 去重查询
        assert!(repo.get_by_hash("hash_g1").unwrap().is_some());
        // 状态推进
        repo.update_status("g1", DistillationStatus::Distilling, 40)
            .unwrap();
        let g = repo.get_by_id("g1").unwrap().unwrap();
        assert_eq!(g.status, DistillationStatus::Distilling);
        assert_eq!(g.progress, 40);
        // 提炼回写
        repo.update_distilled("g1", Some("故事（修订）"), None, Some("技巧"), "custom_m1")
            .unwrap();
        let g = repo.get_by_id("g1").unwrap().unwrap();
        assert_eq!(g.title, "故事（修订）");
        assert_eq!(g.methodology_id.as_deref(), Some("custom_m1"));
        // 列表与删除
        assert_eq!(repo.list_all().unwrap().len(), 1);
        repo.delete("g1").unwrap();
        assert!(repo.get_by_id("g1").unwrap().is_none());
    }

    #[test]
    fn guidebook_merge_into_roundtrip() {
        let pool = create_test_pool().unwrap();
        let repo = GuidebookRepository::new(pool);
        let mut book = sample_guidebook("gm1");
        book.merge_into_methodology_id = Some("custom_target".into());
        repo.create(&book).unwrap();
        let got = repo.get_by_id("gm1").unwrap().unwrap();
        assert_eq!(
            got.merge_into_methodology_id.as_deref(),
            Some("custom_target")
        );
        // 列表项也带该字段
        let items = repo.list_all().unwrap();
        assert_eq!(
            items[0].merge_into_methodology_id.as_deref(),
            Some("custom_target")
        );
        // 普通上传为 None
        repo.create(&sample_guidebook("gm2")).unwrap();
        let got2 = repo.get_by_id("gm2").unwrap().unwrap();
        assert!(got2.merge_into_methodology_id.is_none());
    }

    #[test]
    fn set_merge_into_backfills_merge_intent() {
        let pool = create_test_pool().unwrap();
        let repo = GuidebookRepository::new(pool);
        repo.create(&sample_guidebook("gb1")).unwrap();
        // 初始为空
        assert!(repo
            .get_by_id("gb1")
            .unwrap()
            .unwrap()
            .merge_into_methodology_id
            .is_none());
        // 空记录回写合并意图
        repo.set_merge_into("gb1", "custom_m1").unwrap();
        let g = repo.get_by_id("gb1").unwrap().unwrap();
        assert_eq!(g.merge_into_methodology_id.as_deref(), Some("custom_m1"));
    }

    #[test]
    fn reset_for_retry_clears_status_progress_error() {
        let pool = create_test_pool().unwrap();
        let repo = GuidebookRepository::new(pool);
        repo.create(&sample_guidebook("g9")).unwrap();
        repo.update_error("g9", "LLM 超时").unwrap();
        let g = repo.get_by_id("g9").unwrap().unwrap();
        assert_eq!(g.status, DistillationStatus::Failed);
        assert!(g.error.is_some());
        repo.reset_for_retry("g9").unwrap();
        let g = repo.get_by_id("g9").unwrap().unwrap();
        assert_eq!(g.status, DistillationStatus::Pending);
        assert_eq!(g.progress, 0);
        assert!(g.error.is_none());
    }

    #[test]
    fn custom_methodology_crud_flow() {
        let pool = create_test_pool().unwrap();
        let repo = CustomMethodologyRepository::new(pool);
        let cm = CustomMethodology {
            id: "custom_m1".into(),
            guidebook_id: None,
            name: "三幕冲突法".into(),
            description: Some("d".into()),
            steps: vec![
                MethodologyStep {
                    title: "s1".into(),
                    instruction: "i1".into(),
                    checklist: vec![],
                },
                MethodologyStep {
                    title: "s2".into(),
                    instruction: "i2".into(),
                    checklist: vec!["c".into()],
                },
            ],
            patterns: vec![],
            cheatsheet: Cheatsheet::default(),
            enabled: true,
            created_at: Local::now(),
            updated_at: Local::now(),
        };
        repo.create(&cm).unwrap();
        let got = repo.get_by_id("custom_m1").unwrap().unwrap();
        assert_eq!(got.max_steps(), 2);
        assert_eq!(got.steps[1].checklist, vec!["c"]);
        // enabled 过滤
        assert_eq!(repo.list_enabled().unwrap().len(), 1);
        repo.update("custom_m1", None, None, None, Some(false), None, None)
            .unwrap();
        assert!(repo.list_enabled().unwrap().is_empty());
        assert_eq!(repo.list_all().unwrap().len(), 1);
        // 改名
        repo.update("custom_m1", Some("改名"), None, None, None, None, None)
            .unwrap();
        assert_eq!(repo.get_by_id("custom_m1").unwrap().unwrap().name, "改名");
        repo.delete("custom_m1").unwrap();
        assert!(repo.get_by_id("custom_m1").unwrap().is_none());
    }

    #[test]
    fn custom_methodology_patterns_and_cheatsheet_roundtrip() {
        let pool = create_test_pool().unwrap();
        let repo = CustomMethodologyRepository::new(pool);
        let cm = CustomMethodology {
            id: "custom_p1".into(),
            guidebook_id: None,
            name: "资产测试".into(),
            description: None,
            steps: vec![MethodologyStep {
                title: "s".into(),
                instruction: "i".into(),
                checklist: vec![],
            }],
            patterns: vec![Technique {
                name: "三幕结构".into(),
                when_to_use: "布局全书".into(),
                how: "建置-对抗-解决".into(),
            }],
            cheatsheet: Cheatsheet {
                decision_rules: vec!["当节奏拖沓时删场景，因为每场景须推进冲突".into()],
                anti_patterns: vec![AntiPattern {
                    what: "信息倾倒".into(),
                    why: "读者失去探索欲".into(),
                }],
            },
            enabled: true,
            created_at: Local::now(),
            updated_at: Local::now(),
        };
        repo.create(&cm).unwrap();
        let got = repo.get_by_id("custom_p1").unwrap().unwrap();
        assert_eq!(got.patterns.len(), 1);
        assert_eq!(got.patterns[0].name, "三幕结构");
        assert_eq!(got.cheatsheet.decision_rules.len(), 1);
        assert_eq!(got.cheatsheet.anti_patterns[0].why, "读者失去探索欲");
        // update 新字段
        repo.update(
            "custom_p1",
            None,
            None,
            None,
            None,
            Some(&[Technique {
                name: "新技巧".into(),
                when_to_use: String::new(),
                how: String::new(),
            }]),
            None,
        )
        .unwrap();
        let got = repo.get_by_id("custom_p1").unwrap().unwrap();
        assert_eq!(got.patterns[0].name, "新技巧");
        assert_eq!(got.cheatsheet.decision_rules.len(), 1); // 未传则不动
    }
}
