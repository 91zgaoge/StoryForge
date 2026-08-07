# 指导书 → 创作方法论资产 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 用户上传故事创作指导书（txt/pdf/epub），LLM 自动提炼为带步骤的自定义创作方法论，与 5 种内置方法论同等待遇地接入智能创作全链路。

**Architecture:** 新增 `guidebook_distillation` 模块，复用 `book_deconstruction` 的 parser/chunker/任务系统/进度事件模式；产物落新表 `custom_methodologies`（id 前缀 `custom_`）；续写注入点、策略选择器、步骤推进、前端清单均做 `custom_` 分支适配。

**Tech Stack:** Rust（Tauri 后端）+ React/TS（前端）+ SQLite（r2d2）+ PromptRegistry（resources/prompts/**/*.md）

**设计文档:** `docs/plans/2026-08-07-guidebook-to-methodology-design.md`（已批准，commit 814c033）

## Global Constraints

- 所有 LLM prompt 一律走 PromptRegistry：新增 md 文件放 `resources/prompts/distillation/`，运行时 `resolve_prompt(&pool, id)` 优先、`resolve_prompt_default(id)` 兜底；**禁止在 Rust 代码里硬编码 prompt 文本**。
- 自定义方法论 id 统一前缀 `custom_`（`custom_{uuid}`），与内置 id 区分。
- 测试基线：`cargo test --lib` 1179 项、前端 `npx tsc --noEmit` + `npx vitest run` 367 项，任何任务完成后不得变红。
- 提交信息用中文 conventional commit（如 `feat: ...`）；pre-commit 钩子拦截时运行 `cargo +nightly fmt` / `npx prettier --write`。
- 不提交 `.recovery/` 目录（用户数据）；本计划不做版本号变更、不打 tag。
- 参考实现位置（照搬模式，不要另起炉灶）：
  - 上传/哈希/任务创建：`src-tauri/src/book_deconstruction/service.rs:57 upload_and_analyze`
  - LLM 调用：`src-tauri/src/book_deconstruction/analyzer.rs:798 call_llm`（用 `RoutingRequest` + `llm_service.generate_for_request`）
  - prompt 解析：`analyzer.rs:319 extract_metadata`（resolve_prompt → TemplateEngine::render_with_conditions）
  - JSON 解析：`analyzer.rs:907 parse_json_response`
  - 任务执行器：`src-tauri/src/book_deconstruction/executor.rs`
  - 前端 hook：`src-frontend/src/hooks/useBookDeconstruction.ts`
  - 迁移模式：`src-tauri/src/db/migrations/V097__stories_reference_book_id.rs`
  - 测试 DB：`crate::db::connection::create_test_pool()`（自动跑全部迁移）

---

### Task 1: Migration V120 — guidebooks + custom_methodologies 表

**Files:**
- Create: `src-tauri/src/db/migrations/V120__guidebooks_custom_methodologies.rs`
- Modify: `src-tauri/src/db/migrations/mod.rs`（模块声明 + `all_rust_migrations()` 注册列表）

**Interfaces:**
- Produces: 表 `guidebooks(id, title, author, subject, word_count, file_format, file_hash UNIQUE, file_path, methodology_id, status, progress, error, task_id, created_at, updated_at)`；表 `custom_methodologies(id, guidebook_id FK, name, description, steps_json, enabled, created_at, updated_at)`。后续所有任务依赖这两张表。

- [ ] **Step 1: 写迁移文件**

创建 `src-tauri/src/db/migrations/V120__guidebooks_custom_methodologies.rs`：

```rust
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
```

- [ ] **Step 2: 注册迁移**

在 `src-tauri/src/db/migrations/mod.rs` 中：
1. 模块声明区（仿照 `V118__unify_foreshadowing_thread` 的声明行）添加：
   ```rust
   pub mod V120__guidebooks_custom_methodologies;
   ```
2. `all_rust_migrations()` 返回列表末尾（`Box::new(V118__unify_foreshadowing_thread::Migration),` 之后）添加：
   ```rust
   Box::new(V120__guidebooks_custom_methodologies::Migration),
   ```

- [ ] **Step 3: 跑测试验证**

Run: `cd src-tauri && cargo test --lib v120`
Expected: `v120_creates_tables_idempotent` PASS，其余测试不受影响。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/db/migrations/
git commit -m "feat: 新增 guidebooks 与 custom_methodologies 表（V120）"
```

---

### Task 2: guidebook_distillation 模块骨架 — models + repository

**Files:**
- Create: `src-tauri/src/guidebook_distillation/mod.rs`
- Create: `src-tauri/src/guidebook_distillation/models.rs`
- Create: `src-tauri/src/guidebook_distillation/repository.rs`
- Modify: `src-tauri/src/lib.rs`（模块声明，找 `mod book_deconstruction;` 或 `pub mod book_deconstruction;` 附近）

**Interfaces:**
- Produces:
  - `DistillationStatus`（Display/FromStr，值：pending/extracting/distilling/merging/completed/failed/cancelled）
  - `Guidebook` / `GuidebookListItem` / `DistillationProgressEvent`
  - `MethodologyStep { title, instruction, checklist: Vec<String> }`
  - `CustomMethodology { id, guidebook_id, name, description, steps: Vec<MethodologyStep>, enabled, created_at, updated_at }`，方法 `max_steps() -> i32`
  - `parse_steps(json: &str) -> Vec<MethodologyStep>`
  - LLM 响应类型：`LlmGuidebookMetadataResponse`、`LlmDistillChunkResponse`、`LlmDistillMergeResponse`、`LlmMethodologyStepResponse`、`LlmMethodologyResponse`
  - `DistillationOutput { metadata, methodology }`
  - `GuidebookRepository` / `CustomMethodologyRepository`（CRUD，见代码）

- [ ] **Step 1: models.rs**

```rust
#![allow(dead_code)]
//! 指导书提炼 Models

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

// ==================== 提炼状态 ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DistillationStatus {
    Pending,
    Extracting,
    Distilling,
    Merging,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for DistillationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DistillationStatus::Pending => "pending",
            DistillationStatus::Extracting => "extracting",
            DistillationStatus::Distilling => "distilling",
            DistillationStatus::Merging => "merging",
            DistillationStatus::Completed => "completed",
            DistillationStatus::Failed => "failed",
            DistillationStatus::Cancelled => "cancelled",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for DistillationStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(DistillationStatus::Pending),
            "extracting" => Ok(DistillationStatus::Extracting),
            "distilling" => Ok(DistillationStatus::Distilling),
            "merging" => Ok(DistillationStatus::Merging),
            "completed" => Ok(DistillationStatus::Completed),
            "failed" => Ok(DistillationStatus::Failed),
            "cancelled" => Ok(DistillationStatus::Cancelled),
            _ => Err(format!("Unknown distillation status: {}", s)),
        }
    }
}

// ==================== 指导书主表模型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guidebook {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub word_count: Option<i64>,
    pub file_format: Option<String>,
    pub file_hash: Option<String>,
    pub file_path: Option<String>,
    pub methodology_id: Option<String>,
    pub status: DistillationStatus,
    pub progress: i32,
    pub error: Option<String>,
    pub task_id: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidebookListItem {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub word_count: Option<i64>,
    pub file_format: Option<String>,
    pub methodology_id: Option<String>,
    pub status: String,
    pub progress: i32,
    pub created_at: String,
}

// ==================== 自定义方法论 ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MethodologyStep {
    pub title: String,
    pub instruction: String,
    #[serde(default)]
    pub checklist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMethodology {
    pub id: String,
    pub guidebook_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<MethodologyStep>,
    pub enabled: bool,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

impl CustomMethodology {
    /// 最大步数（章节完成自动推进到顶后停留），至少为 1
    pub fn max_steps(&self) -> i32 {
        (self.steps.len() as i32).max(1)
    }
}

/// 解析 steps_json；坏数据返回空 vec（调用方按「无步骤」处理）
pub fn parse_steps(json: &str) -> Vec<MethodologyStep> {
    serde_json::from_str(json).unwrap_or_default()
}

// ==================== 进度事件 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationProgressEvent {
    pub guidebook_id: String,
    pub status: String,
    pub progress: i32,
    pub current_step: String,
    pub message: Option<String>,
    #[serde(default)]
    pub active_threads: i32,
}

// ==================== LLM 响应类型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmGuidebookMetadataResponse {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmDistillChunkResponse {
    pub points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmDistillMergeResponse {
    pub principles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMethodologyStepResponse {
    pub title: String,
    pub instruction: String,
    #[serde(default)]
    pub checklist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMethodologyResponse {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<LlmMethodologyStepResponse>,
}

/// 提炼流水线的最终产出
#[derive(Debug, Clone)]
pub struct DistillationOutput {
    pub metadata: LlmGuidebookMetadataResponse,
    pub methodology: LlmMethodologyResponse,
}

// ==================== 聚合结果（给前端） ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidebookResult {
    pub guidebook: Guidebook,
    pub methodology: Option<CustomMethodology>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_roundtrip() {
        for s in [
            DistillationStatus::Pending,
            DistillationStatus::Extracting,
            DistillationStatus::Distilling,
            DistillationStatus::Merging,
            DistillationStatus::Completed,
            DistillationStatus::Failed,
            DistillationStatus::Cancelled,
        ] {
            let text = s.to_string();
            assert_eq!(text.parse::<DistillationStatus>().unwrap(), s);
        }
        assert!("bogus".parse::<DistillationStatus>().is_err());
    }

    #[test]
    fn parse_steps_handles_valid_and_invalid() {
        let json = r#"[{"title":"步骤一","instruction":"做某事","checklist":["a","b"]}]"#;
        let steps = parse_steps(json);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].checklist, vec!["a", "b"]);
        // checklist 缺省
        let no_checklist = parse_steps(r#"[{"title":"t","instruction":"i"}]"#);
        assert!(no_checklist[0].checklist.is_empty());
        // 坏 JSON → 空
        assert!(parse_steps("not json").is_empty());
    }

    #[test]
    fn max_steps_at_least_one() {
        let cm = CustomMethodology {
            id: "custom_x".into(),
            guidebook_id: None,
            name: "n".into(),
            description: None,
            steps: vec![],
            enabled: true,
            created_at: Local::now(),
            updated_at: Local::now(),
        };
        assert_eq!(cm.max_steps(), 1);
    }
}
```

- [ ] **Step 2: repository.rs**

模式照搬 `src-tauri/src/book_deconstruction/repository.rs`（`pool.get()?` + `params!` + `query_map`）。

```rust
//! 指导书与自定义方法论 Repository

use chrono::Local;
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
             created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
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
             status, progress, created_at FROM guidebooks ORDER BY created_at DESC",
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
                status: row.get("status")?,
                progress: row.get("progress")?,
                created_at: row.get("created_at")?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn update_status(&self, id: &str, status: DistillationStatus, progress: i32) -> RepoResult<()> {
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
            params![title, author, subject, methodology_id, Local::now().to_rfc3339(), id],
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
             enabled, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                cm.id,
                cm.guidebook_id,
                cm.name,
                cm.description,
                serde_json::to_string(&cm.steps)?,
                cm.enabled as i32,
                cm.created_at.to_rfc3339(),
                cm.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    fn row_to_cm(row: &rusqlite::Row) -> rusqlite::Result<CustomMethodology> {
        let steps_json: String = row.get("steps_json")?;
        let created: String = row.get("created_at")?;
        let updated: String = row.get("updated_at")?;
        let enabled: i32 = row.get("enabled")?;
        Ok(CustomMethodology {
            id: row.get("id")?,
            guidebook_id: row.get("guidebook_id")?,
            name: row.get("name")?,
            description: row.get("description")?,
            steps: parse_steps(&steps_json),
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

    /// 更新名称/描述/步骤/启用状态（None 字段不动）
    pub fn update(
        &self,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
        steps: Option<&[MethodologyStep]>,
        enabled: Option<bool>,
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
        Ok(())
    }

    pub fn delete(&self, id: &str) -> RepoResult<()> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM custom_methodologies WHERE id = ?1", params![id])?;
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
        repo.update_status("g1", DistillationStatus::Distilling, 40).unwrap();
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
    fn custom_methodology_crud_flow() {
        let pool = create_test_pool().unwrap();
        let repo = CustomMethodologyRepository::new(pool);
        let cm = CustomMethodology {
            id: "custom_m1".into(),
            guidebook_id: None,
            name: "三幕冲突法".into(),
            description: Some("d".into()),
            steps: vec![
                MethodologyStep { title: "s1".into(), instruction: "i1".into(), checklist: vec![] },
                MethodologyStep { title: "s2".into(), instruction: "i2".into(), checklist: vec!["c".into()] },
            ],
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
        repo.update("custom_m1", None, None, None, Some(false)).unwrap();
        assert!(repo.list_enabled().unwrap().is_empty());
        assert_eq!(repo.list_all().unwrap().len(), 1);
        // 改名
        repo.update("custom_m1", Some("改名"), None, None, None).unwrap();
        assert_eq!(repo.get_by_id("custom_m1").unwrap().unwrap().name, "改名");
        repo.delete("custom_m1").unwrap();
        assert!(repo.get_by_id("custom_m1").unwrap().is_none());
    }
}
```

- [ ] **Step 3: mod.rs + lib.rs 声明**

创建 `src-tauri/src/guidebook_distillation/mod.rs`：

```rust
//! 指导书提炼模块：上传故事创作指导书 → LLM 提炼为自定义创作方法论资产

pub mod models;
pub mod repository;

pub use models::*;
pub use repository::{CustomMethodologyRepository, GuidebookRepository};
```

在 `src-tauri/src/lib.rs` 中找到 `book_deconstruction` 的模块声明行，在其后添加：

```rust
pub mod guidebook_distillation;
```

- [ ] **Step 4: 跑测试验证**

Run: `cd src-tauri && cargo test --lib guidebook_distillation`
Expected: 4 个新测试 PASS（status_roundtrip / parse_steps / max_steps / 两个 crud_flow），全量 `cargo test --lib` 无回归。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/guidebook_distillation/ src-tauri/src/lib.rs
git commit -m "feat: 指导书提炼模块骨架——models 与 repository"
```

---

### Task 3: 提炼 prompt 资产（4 个 md 文件）

**Files:**
- Create: `resources/prompts/distillation/distill_metadata.md`
- Create: `resources/prompts/distillation/distill_chunk.md`
- Create: `resources/prompts/distillation/distill_merge.md`
- Create: `resources/prompts/distillation/distill_methodology.md`
- Modify: `src-tauri/src/prompts/registry.rs`（仅 tests mod 追加一个测试）

**Interfaces:**
- Produces: PromptRegistry id `distill_metadata`（变量 `text`）、`distill_chunk`（变量 `text`）、`distill_merge`（变量 `points`）、`distill_methodology`（变量 `principles`、`book_title`）。Task 4 的 distiller 按这些 id 与变量名渲染。

frontmatter 格式参照 `resources/prompts/methodology/methodology_snowflake_step1.md`（`---` 包围的 YAML：id/name/description/category/version/variables）。

- [ ] **Step 1: distill_metadata.md**

```markdown
---
id: distill_metadata
name: "指导书元信息识别"
description: "识别故事创作指导书的标题、作者与主题"
category: distillation
version: 0.33.2
variables:
  - text
---

请分析以下书籍开头，识别这是一本什么书。只输出 JSON，不要有任何其他文字。

要求：
1. title: 书名（如无法确定则为null）
2. author: 作者名（如无法确定则为null）
3. subject: 本书主题的一句话概括（例如"小说冲突设计""人物塑造方法"）

文本样本：
{{text}}

JSON格式：
{"title":"...","author":"...","subject":"..."}
```

- [ ] **Step 2: distill_chunk.md**

```markdown
---
id: distill_chunk
name: "指导书分块要点提炼"
description: "从指导书文本片段中提炼可操作的创作原则与技巧要点"
category: distillation
version: 0.33.2
variables:
  - text
---

你是一位小说创作方法论专家。以下是一本故事创作指导书的片段，请提炼其中所有**可操作**的创作原则、技巧、步骤要点。

要求：
1. 每条要点一句话，必须是可执行的创作指导（"应该怎么做"），而不是观点陈述或例子
2. 保留原文中的专业术语
3. 忽略序言、致谢、出版信息等无实质内容的部分
4. 若该片段没有可提炼的内容，返回空数组
5. 只输出 JSON，不要有任何其他文字

文本片段：
{{text}}

JSON格式：
{"points":["要点1","要点2"]}
```

- [ ] **Step 3: distill_merge.md**

```markdown
---
id: distill_merge
name: "指导书要点合并去重"
description: "合并全书提炼的创作要点，去重并按主题聚类"
category: distillation
version: 0.33.2
variables:
  - points
---

你是一位小说创作方法论专家。以下是从一本故事创作指导书各章节提炼出的创作要点列表，请合并去重。

要求：
1. 语义相同的要点合并为一条，保留最准确的表述
2. 按主题归类排序（如：冲突设计、人物塑造、结构节奏、世界观、对白等）
3. 最终保留最重要的 10-20 条
4. 每条一句话，保持可执行性
5. 只输出 JSON，不要有任何其他文字

原始要点列表：
{{points}}

JSON格式：
{"principles":["原则1","原则2"]}
```

- [ ] **Step 4: distill_methodology.md**

```markdown
---
id: distill_methodology
name: "创作方法论结构化生成"
description: "把合并后的创作原则组织成带步骤的创作方法论"
category: distillation
version: 0.33.2
variables:
  - principles
  - book_title
---

你是一位小说创作方法论专家。以下是从指导书《{{book_title}}》提炼的核心创作原则，请把它们组织成一套**分步骤执行**的创作方法论，供 AI 在续写小说时逐步应用。

要求：
1. name: 方法论名称，不超过 12 个字，体现该书核心思想（如"三幕冲突驱动法"）
2. description: 一句话描述该方法论的适用场景与核心价值
3. steps: 3-8 个执行步骤，按创作先后顺序排列；每个步骤包含：
   - title: 步骤名称（不超过 10 个字）
   - instruction: 该步骤的详细执行指引（100-200 字，直接以第二人称指令语气写给执行者，包含该步骤要运用的原则）
   - checklist: 2-4 条该步骤完成质量的自检项（每条一句话，疑问句或判断句）
4. 所有原则必须被分配到某个步骤中，不得遗漏核心思想
5. 只输出 JSON，不要有任何其他文字

核心创作原则：
{{principles}}

JSON格式：
{"name":"...","description":"...","steps":[{"title":"...","instruction":"...","checklist":["..."]}]}
```

- [ ] **Step 5: 注册表测试**

在 `src-tauri/src/prompts/registry.rs` 的 `#[cfg(test)]` 测试模块中（参照其中已有的内置 prompt 解析测试写法）追加：

```rust
#[test]
fn distillation_prompts_registered() {
    for id in [
        "distill_metadata",
        "distill_chunk",
        "distill_merge",
        "distill_methodology",
    ] {
        assert!(
            resolve_prompt_default(id).is_some(),
            "内置 prompt {} 未注册",
            id
        );
    }
}
```

- [ ] **Step 6: 跑测试验证**

Run: `cd src-tauri && cargo test --lib distillation_prompts`
Expected: PASS。

- [ ] **Step 7: Commit**

```bash
git add resources/prompts/distillation/ src-tauri/src/prompts/registry.rs
git commit -m "feat: 新增指导书提炼的四个 prompt 资产"
```

---

### Task 4: distiller.rs — LLM 提炼状态机

**Files:**
- Create: `src-tauri/src/guidebook_distillation/distiller.rs`
- Modify: `src-tauri/src/guidebook_distillation/mod.rs`（加 `pub mod distiller;`）
- Modify: `src-tauri/src/book_deconstruction/analyzer.rs:907`（`fn parse_json_response` 改 `pub(crate) fn parse_json_response`；若 `extract_sample` 是私有 `fn`，同样改 `pub(crate)`）

**Interfaces:**
- Consumes: `TextChunk` / `AnalysisError`（`crate::book_deconstruction::models`）、`parse_json_response`（`crate::book_deconstruction::analyzer`）、`create_chunks` 产物（Task 5 提供）、prompt id（Task 3）。
- Produces: `GuidebookDistiller::new(llm_service, app_handle, pool, concurrency)`；`distiller.distill(guidebook_id, chunks, heartbeat, cancel_check) -> Result<DistillationOutput, AnalysisError>`。

- [ ] **Step 1: 提升 analyzer.rs 两个函数的可见性**

`src-tauri/src/book_deconstruction/analyzer.rs`：
- 第 907 行：`fn parse_json_response<T: serde::de::DeserializeOwned>(...)` → `pub(crate) fn parse_json_response<T: serde::de::DeserializeOwned>(...)`
- 找到 `fn extract_sample(`，改为 `pub(crate) fn extract_sample(`

- [ ] **Step 2: distiller.rs**

```rust
//! 指导书提炼器：LLM 状态机
//!
//! 流程：元信息（→10%）→ 分块提炼（10→70%，并发）→ 合并去重（→85%）→
//! 结构化方法论（→100%）。方法论 JSON 解析失败重试一次，仍失败则整体 Failed。

use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

use tauri::{AppHandle, Emitter};
use tokio::sync::Semaphore;

use super::models::*;
use crate::{
    book_deconstruction::{
        analyzer::{extract_sample, parse_json_response},
        models::{AnalysisError, TextChunk},
    },
    db::DbPool,
    llm::LlmService,
};

pub struct GuidebookDistiller {
    llm_service: LlmService,
    app_handle: AppHandle,
    pool: DbPool,
    semaphore: Arc<Semaphore>,
    active_requests: Arc<AtomicI32>,
}

impl GuidebookDistiller {
    pub fn new(
        llm_service: LlmService,
        app_handle: AppHandle,
        pool: DbPool,
        concurrency: usize,
    ) -> Self {
        Self {
            llm_service,
            app_handle,
            pool,
            semaphore: Arc::new(Semaphore::new(concurrency.max(1).min(100))),
            active_requests: Arc::new(AtomicI32::new(0)),
        }
    }

    pub async fn distill(
        &self,
        guidebook_id: &str,
        chunks: &[TextChunk],
        heartbeat_callback: Option<Box<dyn Fn() + Send + Sync>>,
        cancel_check: Option<Box<dyn Fn() -> bool + Send + Sync>>,
    ) -> Result<DistillationOutput, AnalysisError> {
        let check_cancel = || -> Result<(), AnalysisError> {
            if let Some(ref cb) = cancel_check {
                if cb() {
                    return Err(AnalysisError::Cancelled("用户取消提炼".to_string()));
                }
            }
            Ok(())
        };
        let heartbeat = || {
            if let Some(ref cb) = heartbeat_callback {
                cb();
            }
        };

        // Step 1: 元信息（→10%）
        self.emit_progress(guidebook_id, "extracting", 5, "正在识别指导书元信息...")
            .await;
        heartbeat();
        check_cancel()?;
        let sample = extract_sample(
            &chunks.first().map(|c| c.content.clone()).unwrap_or_default(),
            3000,
        );
        let metadata = self.extract_metadata(&sample).await?;
        let book_title = metadata.title.clone().unwrap_or_else(|| "未命名".to_string());
        self.emit_progress(guidebook_id, "distilling", 10, &format!("识别完成：《{}》", book_title))
            .await;
        heartbeat();
        check_cancel()?;

        // Step 2: 分块提炼（10→70%，并发）
        let total = chunks.len();
        self.emit_progress(
            guidebook_id,
            "distilling",
            12,
            &format!("正在分块提炼创作要点（共 {} 块）...", total),
        )
        .await;
        let points = self.distill_chunks(guidebook_id, chunks, &cancel_check).await?;
        heartbeat();
        check_cancel()?;

        // Step 3: 合并去重（→85%）
        self.emit_progress(guidebook_id, "merging", 72, "正在合并去重创作要点...")
            .await;
        let principles = self.merge_points(&points).await?;
        heartbeat();
        check_cancel()?;

        // Step 4: 结构化方法论（→100%），JSON 失败重试一次
        self.emit_progress(guidebook_id, "merging", 88, "正在生成创作方法论...")
            .await;
        let methodology = match self.generate_methodology(&principles, &book_title).await {
            Ok(m) => m,
            Err(e) => {
                log::warn!("[GuidebookDistiller] methodology 首次生成失败，重试一次: {}", e);
                self.generate_methodology(&principles, &book_title).await?
            }
        };
        self.emit_progress(guidebook_id, "merging", 100, "提炼完成").await;
        heartbeat();

        Ok(DistillationOutput { metadata, methodology })
    }

    // ==================== 各步骤实现 ====================

    fn render_prompt(&self, id: &str, vars: &[(&str, String)]) -> Option<String> {
        let tpl = crate::prompts::registry::resolve_prompt(&self.pool, id)
            .ok()
            .or_else(|| crate::prompts::registry::resolve_prompt_default(id))?;
        let mut map = std::collections::HashMap::new();
        for (k, v) in vars {
            map.insert(k.to_string(), v.clone());
        }
        Some(crate::prompts::engine::TemplateEngine::render_with_conditions(
            &tpl, &map,
        ))
    }

    async fn extract_metadata(
        &self,
        sample_text: &str,
    ) -> Result<LlmGuidebookMetadataResponse, AnalysisError> {
        let prompt = self
            .render_prompt("distill_metadata", &[("text", sample_text.to_string())])
            .ok_or_else(|| AnalysisError::LlmError("prompt distill_metadata 未注册".into()))?;
        let resp = call_llm(
            &self.llm_service,
            "guidebook_metadata",
            prompt,
            Some(500),
            Some(0.3),
        )
        .await?;
        parse_json_response(&resp)
    }

    async fn distill_chunks(
        &self,
        guidebook_id: &str,
        chunks: &[TextChunk],
        cancel_check: &Option<Box<dyn Fn() -> bool + Send + Sync>>,
    ) -> Result<Vec<String>, AnalysisError> {
        let total = chunks.len();
        let processed = Arc::new(AtomicI32::new(0));
        let mut set = tokio::task::JoinSet::new();

        for chunk in chunks {
            if let Some(cb) = cancel_check {
                if cb() {
                    return Err(AnalysisError::Cancelled("用户取消提炼".to_string()));
                }
            }
            let sem = self.semaphore.clone();
            let llm = self.llm_service.clone();
            let pool = self.pool.clone();
            let active = self.active_requests.clone();
            let processed = processed.clone();
            let app = self.app_handle.clone();
            let gid = guidebook_id.to_string();
            let content = chunk.content.clone();
            let total = total;

            set.spawn(async move {
                let _permit = sem.acquire().await.map_err(|e| {
                    AnalysisError::LlmError(format!("Semaphore error: {}", e))
                })?;
                active.fetch_add(1, Ordering::Relaxed);
                let result = async {
                    let tpl = crate::prompts::registry::resolve_prompt(&pool, "distill_chunk")
                        .ok()
                        .or_else(|| {
                            crate::prompts::registry::resolve_prompt_default("distill_chunk")
                        })
                        .ok_or_else(|| {
                            AnalysisError::LlmError("prompt distill_chunk 未注册".into())
                        })?;
                    let mut vars = std::collections::HashMap::new();
                    vars.insert("text".to_string(), content);
                    let prompt = crate::prompts::engine::TemplateEngine::render_with_conditions(
                        &tpl, &vars,
                    );
                    let resp =
                        call_llm(&llm, "guidebook_chunk", prompt, Some(2000), Some(0.3)).await?;
                    let parsed: LlmDistillChunkResponse = parse_json_response(&resp)?;
                    Ok::<Vec<String>, AnalysisError>(parsed.points)
                }
                .await;
                active.fetch_sub(1, Ordering::Relaxed);
                let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
                // 10→70 按分块进度线性推进
                let progress = 10 + (60 * done / total.max(1) as i32);
                let _ = app.emit(
                    "guidebook-distillation-progress",
                    DistillationProgressEvent {
                        guidebook_id: gid,
                        status: "distilling".to_string(),
                        progress,
                        current_step: format!("分块提炼中 {}/{}", done, total),
                        message: None,
                        active_threads: active.load(Ordering::Relaxed),
                    },
                );
                result
            });
        }

        let mut all_points = Vec::new();
        while let Some(res) = set.join_next().await {
            match res {
                Ok(Ok(points)) => all_points.extend(points),
                Ok(Err(e)) => {
                    // 单块失败不致命：记录并继续（剩余块仍可提供要点）
                    log::warn!("[GuidebookDistiller] 单块提炼失败，跳过: {}", e);
                }
                Err(e) => {
                    return Err(AnalysisError::LlmError(format!("Join error: {}", e)));
                }
            }
        }
        Ok(all_points)
    }

    async fn merge_points(&self, points: &[String]) -> Result<Vec<String>, AnalysisError> {
        if points.is_empty() {
            return Err(AnalysisError::LlmError(
                "全书未提炼出任何创作要点".to_string(),
            ));
        }
        // 截断防爆 token：每条 200 字、总量 12000 字
        let joined = points
            .iter()
            .map(|p| {
                let s = p.trim();
                if s.chars().count() > 200 {
                    s.chars().take(200).collect::<String>()
                } else {
                    s.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let joined = if joined.chars().count() > 12000 {
            joined.chars().take(12000).collect::<String>()
        } else {
            joined
        };
        let prompt = self
            .render_prompt("distill_merge", &[("points", joined)])
            .ok_or_else(|| AnalysisError::LlmError("prompt distill_merge 未注册".into()))?;
        let resp = call_llm(
            &self.llm_service,
            "guidebook_merge",
            prompt,
            Some(2000),
            Some(0.3),
        )
        .await?;
        let parsed: LlmDistillMergeResponse = parse_json_response(&resp)?;
        if parsed.principles.is_empty() {
            return Err(AnalysisError::LlmError("合并后原则为空".to_string()));
        }
        Ok(parsed.principles)
    }

    async fn generate_methodology(
        &self,
        principles: &[String],
        book_title: &str,
    ) -> Result<LlmMethodologyResponse, AnalysisError> {
        let prompt = self
            .render_prompt(
                "distill_methodology",
                &[
                    ("principles", principles.join("\n")),
                    ("book_title", book_title.to_string()),
                ],
            )
            .ok_or_else(|| {
                AnalysisError::LlmError("prompt distill_methodology 未注册".into())
            })?;
        let resp = call_llm(
            &self.llm_service,
            "guidebook_methodology",
            prompt,
            Some(4000),
            Some(0.5),
        )
        .await?;
        let parsed: LlmMethodologyResponse = parse_json_response(&resp)?;
        validate_methodology(parsed)
    }

    async fn emit_progress(&self, guidebook_id: &str, status: &str, progress: i32, message: &str) {
        let _ = self.app_handle.emit(
            "guidebook-distillation-progress",
            DistillationProgressEvent {
                guidebook_id: guidebook_id.to_string(),
                status: status.to_string(),
                progress,
                current_step: message.to_string(),
                message: Some(message.to_string()),
                active_threads: self.active_requests.load(Ordering::Relaxed),
            },
        );
    }
}

/// 校验提炼产物：名称非空、至少一个步骤、每个步骤有 instruction
fn validate_methodology(m: LlmMethodologyResponse) -> Result<LlmMethodologyResponse, AnalysisError> {
    if m.name.trim().is_empty() {
        return Err(AnalysisError::LlmError("方法论名称为空".to_string()));
    }
    if m.steps.is_empty() {
        return Err(AnalysisError::LlmError("方法论步骤为空".to_string()));
    }
    if m.steps.iter().any(|s| s.instruction.trim().is_empty()) {
        return Err(AnalysisError::LlmError("存在空 instruction 的步骤".to_string()));
    }
    Ok(m)
}

/// LLM 调用（与 book_deconstruction/analyzer.rs:798 call_llm 相同的路由方式；
/// use 语句中的 RoutingRequest/Complexity/Priority/TaskType 照搬该文件顶部）
async fn call_llm(
    llm_service: &LlmService,
    context_label: &str,
    prompt: String,
    max_tokens: Option<i32>,
    temperature: Option<f32>,
) -> Result<String, AnalysisError> {
    let request = crate::llm::routing::RoutingRequest {
        task: crate::llm::routing::TaskType::Analysis,
        complexity: crate::llm::routing::Complexity::Medium,
        budget_priority: crate::llm::routing::Priority::Low,
        speed_priority: crate::llm::routing::Priority::Low,
        estimated_input_tokens: 0,
        constraints: vec![],
    };
    llm_service
        .generate_for_request(request, prompt, max_tokens, temperature, Some(context_label))
        .await
        .map(|r| r.content)
        .map_err(|e| AnalysisError::LlmError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_methodology_accepts_valid() {
        let m = LlmMethodologyResponse {
            name: "三幕冲突法".into(),
            description: Some("d".into()),
            steps: vec![LlmMethodologyStepResponse {
                title: "建冲突".into(),
                instruction: "先确立核心冲突".into(),
                checklist: vec!["冲突是否明确？".into()],
            }],
        };
        assert!(validate_methodology(m).is_ok());
    }

    #[test]
    fn validate_methodology_rejects_empty() {
        let no_name = LlmMethodologyResponse {
            name: "  ".into(),
            description: None,
            steps: vec![LlmMethodologyStepResponse {
                title: "t".into(),
                instruction: "i".into(),
                checklist: vec![],
            }],
        };
        assert!(validate_methodology(no_name).is_err());

        let no_steps = LlmMethodologyResponse {
            name: "n".into(),
            description: None,
            steps: vec![],
        };
        assert!(validate_methodology(no_steps).is_err());

        let empty_instruction = LlmMethodologyResponse {
            name: "n".into(),
            description: None,
            steps: vec![LlmMethodologyStepResponse {
                title: "t".into(),
                instruction: " ".into(),
                checklist: vec![],
            }],
        };
        assert!(validate_methodology(empty_instruction).is_err());
    }
}
```

注意：`crate::llm::routing::` 路径若与 analyzer.rs 顶部 use 语句不一致（例如是 `crate::llm::RoutingRequest` 的 re-export），照搬 analyzer.rs 的写法。

- [ ] **Step 3: mod.rs 更新**

`src-tauri/src/guidebook_distillation/mod.rs` 添加：

```rust
pub mod distiller;
```

- [ ] **Step 4: 跑测试验证**

Run: `cd src-tauri && cargo test --lib guidebook_distillation`
Expected: 含 `validate_methodology_*` 在内的模块测试 PASS，全量无回归。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/guidebook_distillation/ src-tauri/src/book_deconstruction/analyzer.rs
git commit -m "feat: 指导书 LLM 提炼状态机（分块提炼/合并/结构化方法论）"
```

---

### Task 5: service + commands + executor + 注册

**Files:**
- Create: `src-tauri/src/guidebook_distillation/service.rs`
- Create: `src-tauri/src/guidebook_distillation/commands.rs`
- Create: `src-tauri/src/guidebook_distillation/executor.rs`
- Modify: `src-tauri/src/guidebook_distillation/mod.rs`（加 service/commands/executor 声明与 re-export）
- Modify: `src-tauri/src/task_system/models.rs:79`（`TaskType` 枚举加 `GuidebookDistillation` 变体 + 第 94 行 Display + 第 110 行 FromStr）
- Modify: `src-tauri/src/lib.rs`（注册 executor，约 402-409 行 BookDeconstructionExecutor 注册之后）
- Modify: `src-tauri/src/handlers.rs`（命令注册，196-201 行 book_deconstruction 命令附近）

**Interfaces:**
- Consumes: Task 2 repository、Task 4 distiller、`parse_book` / `create_chunks`（`crate::book_deconstruction`）、`ParseError`。
- Produces（前端 invoke 名）：
  - `upload_guidebook(file_path) -> String`
  - `get_guidebook_distillation_status(guidebook_id) -> GuidebookStatusResponse { guidebook_id, status, progress, current_step, error }`
  - `get_guidebook_result(guidebook_id) -> GuidebookResult`
  - `list_guidebooks() -> Vec<GuidebookListItem>`
  - `delete_guidebook(guidebook_id) -> ()`
  - `cancel_guidebook_distillation(guidebook_id) -> ()`
  - `list_all_methodologies() -> Vec<MethodologyInfo>`
  - `update_custom_methodology(id, name?, description?, steps?, enabled?) -> ()`
  - `delete_custom_methodology(id) -> ()`
- Produces（Rust 侧）：`render_custom_methodology_extension(pool, methodology_id, step) -> Option<String>`（Task 6 用）。

- [ ] **Step 1: TaskType 加变体**

`src-tauri/src/task_system/models.rs`：
- 枚举（79 行附近，`BookDeconstruction, // 拆书分析` 之后）加：
  ```rust
  GuidebookDistillation, // 指导书提炼
  ```
- Display（94 行附近）加：
  ```rust
  TaskType::GuidebookDistillation => write!(f, "guidebook_distillation"),
  ```
- FromStr（110 行附近）加：
  ```rust
  "guidebook_distillation" => TaskType::GuidebookDistillation,
  ```
- 若该文件有序列化/其他 match 穷举（编译器会报错指出），一并补全。
- 若 `TaskService` 对 `book_deconstruction` 有墙钟放宽逻辑（`grep -rn "book_deconstruction" src-tauri/src/task_system/`），将 `guidebook_distillation` 加入同一分支。
- 若订阅特性有清单（`grep -rn "book_deconstruction" src-tauri/src/subscription/`），登记 `guidebook_distillation` 为同档特性。

- [ ] **Step 2: service.rs**

```rust
//! 指导书提炼 Service：上传校验 → 去重 → 解析 → 任务系统执行提炼。

use std::{path::Path, sync::Arc};

use chrono::Local;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use super::{
    distiller::GuidebookDistiller,
    models::*,
    repository::{CustomMethodologyRepository, GuidebookRepository},
};
use crate::{
    book_deconstruction::{
        chunker::create_chunks,
        models::{AnalysisError, ParseError, TextChunk},
        parser::parse_book,
    },
    db::DbPool,
    llm::LlmService,
    task_system::{models::CreateTaskRequest, service::TaskService},
};

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB

pub struct GuidebookDistillationService {
    pool: DbPool,
    llm_service: LlmService,
    app_handle: AppHandle,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuidebookStatusResponse {
    pub guidebook_id: String,
    pub status: String,
    pub progress: i32,
    pub current_step: Option<String>,
    pub error: Option<String>,
}

impl GuidebookDistillationService {
    pub fn new(pool: DbPool, llm_service: LlmService, app_handle: AppHandle) -> Self {
        Self {
            pool,
            llm_service,
            app_handle,
        }
    }

    // ==================== 上传并提炼 ====================

    pub async fn upload_and_distill(&self, file_path: &Path) -> Result<String, ParseError> {
        self.validate_file(file_path)?;
        let file_hash = self.compute_file_hash(file_path).await?;

        let repo = GuidebookRepository::new(self.pool.clone());
        if let Ok(Some(existing)) = repo.get_by_hash(&file_hash) {
            log::info!("[GuidebookDistillation] File already exists: {}", existing.id);
            return Ok(existing.id);
        }

        let guidebook_id = Uuid::new_v4().to_string();
        let app_dir = self
            .app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        let dir = app_dir.join("guidebooks");
        std::fs::create_dir_all(&dir)
            .map_err(|e| ParseError::IoError(format!("Failed to create guidebooks dir: {}", e)))?;
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt")
            .to_lowercase();
        let dest_path = dir.join(format!("{}.{}", guidebook_id, ext));
        tokio::fs::copy(file_path, &dest_path)
            .await
            .map_err(|e| ParseError::IoError(format!("Failed to copy file: {}", e)))?;

        let parsed = parse_book(&dest_path, None)?;

        let now = Local::now();
        let book = Guidebook {
            id: guidebook_id.clone(),
            title: parsed.title.clone().unwrap_or_else(|| "未命名".to_string()),
            author: parsed.author.clone(),
            subject: None,
            word_count: Some(parsed.word_count as i64),
            file_format: Some(ext),
            file_hash: Some(file_hash),
            file_path: Some(dest_path.to_string_lossy().to_string()),
            methodology_id: None,
            status: DistillationStatus::Pending,
            progress: 0,
            error: None,
            task_id: None,
            created_at: now,
            updated_at: now,
        };
        repo.create(&book)
            .map_err(|e| ParseError::StorageError(format!("Failed to create record: {}", e)))?;

        let payload = serde_json::json!({
            "guidebook_id": guidebook_id,
            "file_path": dest_path.to_string_lossy().to_string(),
        })
        .to_string();
        let task_req = CreateTaskRequest {
            name: format!(
                "指导书提炼: {}",
                parsed.title.clone().unwrap_or_else(|| "未命名".to_string())
            ),
            description: Some(format!("提炼 {} 字的指导书", parsed.word_count)),
            task_type: "guidebook_distillation".to_string(),
            schedule_type: "once".to_string(),
            cron_pattern: None,
            payload: Some(payload),
            enabled: Some(true),
            max_retries: Some(3),
            heartbeat_timeout_seconds: Some(600),
        };

        let task_service = self.app_handle.state::<TaskService>();
        match task_service.create_task(task_req) {
            Ok(task) => {
                let _ = repo.update_task_id(&guidebook_id, &task.id);
                let _ = repo.update_status(&guidebook_id, DistillationStatus::Pending, 0);
            }
            Err(e) => {
                log::error!(
                    "[GuidebookDistillation] 任务创建失败，回退直接后台提炼: {}",
                    e
                );
                let pool = self.pool.clone();
                let llm_service = self.llm_service.clone();
                let app_handle = self.app_handle.clone();
                let gid = guidebook_id.clone();
                let chunks = create_chunks(&parsed);
                tauri::async_runtime::spawn(async move {
                    let service =
                        GuidebookDistillationService::new(pool.clone(), llm_service, app_handle);
                    if let Err(e) = service.run_distillation(&gid, &chunks, None, None).await {
                        log::error!("[GuidebookDistillation] 回退提炼失败 {}: {}", gid, e);
                        let repo = GuidebookRepository::new(pool.clone());
                        let _ = repo.update_error(&gid, &e.to_string());
                    }
                });
            }
        }

        Ok(guidebook_id)
    }

    /// 执行提炼（任务系统与回退路径共用）
    pub async fn run_distillation(
        &self,
        guidebook_id: &str,
        chunks: &[TextChunk],
        heartbeat: Option<Box<dyn Fn() + Send + Sync>>,
        cancel_check: Option<Box<dyn Fn() -> bool + Send + Sync>>,
    ) -> Result<(), AnalysisError> {
        let repo = GuidebookRepository::new(self.pool.clone());
        repo.update_status(guidebook_id, DistillationStatus::Distilling, 0)
            .map_err(|e| AnalysisError::StorageError(e.to_string()))?;

        let concurrency = {
            let app_dir = self.app_handle.path().app_data_dir().unwrap_or_default();
            crate::config::AppConfig::load(&app_dir)
                .map(|c| c.book_deconstruction_concurrency)
                .unwrap_or(3)
        };

        let distiller = GuidebookDistiller::new(
            self.llm_service.clone(),
            self.app_handle.clone(),
            self.pool.clone(),
            concurrency,
        );
        let output = distiller
            .distill(guidebook_id, chunks, heartbeat, cancel_check)
            .await?;

        // 落库：自定义方法论
        let methodology_id = format!("custom_{}", Uuid::new_v4());
        let now = Local::now();
        let cm = CustomMethodology {
            id: methodology_id.clone(),
            guidebook_id: Some(guidebook_id.to_string()),
            name: output.methodology.name.clone(),
            description: output.methodology.description.clone(),
            steps: output
                .methodology
                .steps
                .iter()
                .map(|s| MethodologyStep {
                    title: s.title.clone(),
                    instruction: s.instruction.clone(),
                    checklist: s.checklist.clone(),
                })
                .collect(),
            enabled: true,
            created_at: now,
            updated_at: now,
        };
        CustomMethodologyRepository::new(self.pool.clone())
            .create(&cm)
            .map_err(|e| AnalysisError::StorageError(e.to_string()))?;

        repo.update_distilled(
            guidebook_id,
            output.metadata.title.as_deref(),
            output.metadata.author.as_deref(),
            output.metadata.subject.as_deref(),
            &methodology_id,
        )
        .map_err(|e| AnalysisError::StorageError(e.to_string()))?;
        repo.update_status(guidebook_id, DistillationStatus::Completed, 100)
            .map_err(|e| AnalysisError::StorageError(e.to_string()))?;
        log::info!(
            "[GuidebookDistillation] {} 提炼完成 → 方法论 {}（{}）",
            guidebook_id,
            methodology_id,
            cm.name
        );
        Ok(())
    }

    // ==================== 查询与管理 ====================

    pub fn get_status(&self, guidebook_id: &str) -> Result<GuidebookStatusResponse, crate::error::AppError> {
        let repo = GuidebookRepository::new(self.pool.clone());
        let book = repo
            .get_by_id(guidebook_id)
            .map_err(|e| crate::error::AppError::database(e.to_string()))?
            .ok_or_else(|| crate::error::AppError::not_found("指导书不存在"))?;
        Ok(GuidebookStatusResponse {
            guidebook_id: book.id,
            status: book.status.to_string(),
            progress: book.progress,
            current_step: None,
            error: book.error,
        })
    }

    pub fn get_result(&self, guidebook_id: &str) -> Result<GuidebookResult, crate::error::AppError> {
        let repo = GuidebookRepository::new(self.pool.clone());
        let book = repo
            .get_by_id(guidebook_id)
            .map_err(|e| crate::error::AppError::database(e.to_string()))?
            .ok_or_else(|| crate::error::AppError::not_found("指导书不存在"))?;
        let methodology = book
            .methodology_id
            .as_deref()
            .and_then(|mid| {
                CustomMethodologyRepository::new(self.pool.clone())
                    .get_by_id(mid)
                    .ok()
                    .flatten()
            });
        Ok(GuidebookResult {
            guidebook: book,
            methodology,
        })
    }

    pub fn list_guidebooks(&self) -> Result<Vec<GuidebookListItem>, crate::error::AppError> {
        GuidebookRepository::new(self.pool.clone())
            .list_all()
            .map_err(|e| crate::error::AppError::database(e.to_string()))
    }

    pub fn delete_guidebook(&self, guidebook_id: &str) -> Result<(), crate::error::AppError> {
        let repo = GuidebookRepository::new(self.pool.clone());
        if let Ok(Some(book)) = repo.get_by_id(guidebook_id) {
            if let Some(path) = book.file_path {
                let _ = std::fs::remove_file(path);
            }
        }
        repo.delete(guidebook_id)
            .map_err(|e| crate::error::AppError::database(e.to_string()))
    }

    /// 取消：置 Cancelled；任务系统侧取消由前端对任务的操作或 executor 的
    /// cancel_check 完成（照搬 BookDeconstructionService::cancel_analysis 的做法，
    /// 若该方法通过 TaskService 取消任务则同样处理 task_id）。
    pub fn cancel_distillation(&self, guidebook_id: &str) -> Result<(), crate::error::AppError> {
        let repo = GuidebookRepository::new(self.pool.clone());
        if let Ok(Some(book)) = repo.get_by_id(guidebook_id) {
            if let Some(task_id) = book.task_id {
                let task_service = self.app_handle.state::<TaskService>();
                let _ = task_service.cancel_task(&task_id);
            }
        }
        repo.update_status(guidebook_id, DistillationStatus::Cancelled, 0)
            .map_err(|e| crate::error::AppError::database(e.to_string()))
    }

    // ==================== 文件工具（与拆书同款规则） ====================

    fn validate_file(&self, file_path: &Path) -> Result<(), ParseError> {
        if !file_path.exists() {
            return Err(ParseError::IoError("文件不存在".to_string()));
        }
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !["txt", "pdf", "epub"].contains(&ext.as_str()) {
            return Err(ParseError::InvalidFormat(format!(
                "不支持的格式 .{}，仅支持 txt/pdf/epub",
                ext
            )));
        }
        let size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
        if size > MAX_FILE_SIZE {
            return Err(ParseError::FileTooLarge(format!(
                "文件大小 {}MB 超过 100MB 上限",
                size / 1024 / 1024
            )));
        }
        Ok(())
    }

    async fn compute_file_hash(&self, file_path: &Path) -> Result<String, ParseError> {
        let content = tokio::fs::read(file_path)
            .await
            .map_err(|e| ParseError::IoError(format!("读取文件失败: {}", e)))?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }
}

/// 续写注入点用：渲染自定义方法论当前步骤的约束文本。
/// 未知 id / 已禁用 / 无步骤 → None（调用方静默跳过注入）。
pub fn render_custom_methodology_extension(
    pool: &DbPool,
    methodology_id: &str,
    step: i32,
) -> Option<String> {
    let cm = CustomMethodologyRepository::new(pool.clone())
        .get_by_id(methodology_id)
        .ok()
        .flatten()?;
    if !cm.enabled || cm.steps.is_empty() {
        return None;
    }
    let idx = ((step.max(1) as usize) - 1).min(cm.steps.len() - 1);
    let s = &cm.steps[idx];
    let checklist = if s.checklist.is_empty() {
        String::new()
    } else {
        format!(
            "\n检查清单：\n{}",
            s.checklist
                .iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    Some(format!(
        "【创作方法论（{}·第{}步：{}）】\n{}{}",
        cm.name,
        idx + 1,
        s.title,
        s.instruction,
        checklist
    ))
}
```

注意：
- `AppError::database` / `AppError::not_found` 的确切构造器名以 `src-tauri/src/error.rs` 现有用法为准（参照 book_deconstruction service 里 `AppError::from` 的转换方式；若拆书命令直接返回 `Result<_, AppError>` 并用 `.map_err(AppError::from)`，照搬即可）。
- `TaskService::cancel_task` 方法名以 `src-tauri/src/task_system/service.rs` 实际取消 API 为准（参照 `BookDeconstructionService::cancel_analysis` 的实现）。

- [ ] **Step 3: executor.rs**

照搬 `src-tauri/src/book_deconstruction/executor.rs` 的结构，但不走 LitSeg pipeline（取消用共享 AtomicBool）：

```rust
//! 指导书提炼 Task Executor

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tauri::{AppHandle, Manager};

use super::service::GuidebookDistillationService;
use crate::{
    book_deconstruction::{chunker::create_chunks, parser::parse_book},
    db::DbPool,
    guidebook_distillation::repository::GuidebookRepository,
    llm::LlmService,
    task_system::{
        executor::{TaskExecutionContext, TaskExecutor},
        models::*,
    },
};

pub struct GuidebookDistillationExecutor {
    pool: DbPool,
    llm_service: LlmService,
    app_handle: AppHandle,
}

impl GuidebookDistillationExecutor {
    pub fn new(pool: DbPool, llm_service: LlmService, app_handle: AppHandle) -> Self {
        Self {
            pool,
            llm_service,
            app_handle,
        }
    }
}

#[async_trait::async_trait]
impl TaskExecutor for GuidebookDistillationExecutor {
    fn can_handle(&self, task_type: &TaskType) -> bool {
        *task_type == TaskType::GuidebookDistillation
    }

    async fn execute(&self, task: &Task) -> Result<TaskResult, Box<dyn std::error::Error>> {
        log::info!("[GuidebookDistillationExecutor] Task {} started", task.id);
        let ctx =
            TaskExecutionContext::new(task.id.clone(), self.pool.clone(), self.app_handle.clone());
        ctx.log("info", "开始指导书提炼任务");

        let payload: serde_json::Value = match task.payload.as_deref() {
            Some(p) => serde_json::from_str(p).unwrap_or_else(|_| serde_json::json!({})),
            None => serde_json::json!({}),
        };
        let guidebook_id = match payload.get("guidebook_id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => {
                return Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some("Missing guidebook_id in task payload".to_string()),
                })
            }
        };
        let file_path = match payload.get("file_path").and_then(|v| v.as_str()) {
            Some(p) => p.to_string(),
            None => {
                return Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some("Missing file_path in task payload".to_string()),
                })
            }
        };

        // 取消监控：任务系统取消 → 共享标志 → distiller 的 cancel_check
        let cancel_flag = Arc::new(AtomicBool::new(false));
        {
            let flag = cancel_flag.clone();
            let loop_task_id = task.id.clone();
            let loop_pool = self.pool.clone();
            let loop_app = self.app_handle.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    let check_ctx = TaskExecutionContext::new(
                        loop_task_id.clone(),
                        loop_pool.clone(),
                        loop_app.clone(),
                    );
                    if check_ctx.is_cancelled() {
                        flag.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            });
        }

        ctx.update_progress("parsing", 0, "正在解析文件...");
        ctx.heartbeat();

        let path = std::path::PathBuf::from(file_path);
        let parsed = match tokio::task::spawn_blocking(move || parse_book(&path, None)).await {
            Ok(Ok(p)) => p,
            Ok(Err(e)) => {
                let _ = GuidebookRepository::new(self.pool.clone())
                    .update_error(&guidebook_id, &format!("文件解析失败: {}", e));
                return Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some(format!("文件解析失败: {}", e)),
                });
            }
            Err(e) => {
                return Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some(format!("解析任务异常: {}", e)),
                });
            }
        };

        let chunks = create_chunks(&parsed);
        let service = GuidebookDistillationService::new(
            self.pool.clone(),
            self.llm_service.clone(),
            self.app_handle.clone(),
        );
        let flag_for_check = cancel_flag.clone();
        // 心跳闭包：每个主要步骤回调一次，刷新任务系统心跳
        // （与 book_deconstruction/executor.rs 传给 analyzer 的 heartbeat_callback 同模式）
        let hb_pool = self.pool.clone();
        let hb_app = self.app_handle.clone();
        let hb_task_id = task.id.clone();
        let heartbeat = Box::new(move || {
            let ctx =
                TaskExecutionContext::new(hb_task_id.clone(), hb_pool.clone(), hb_app.clone());
            ctx.heartbeat();
        });
        let result = service
            .run_distillation(
                &guidebook_id,
                &chunks,
                Some(heartbeat),
                Some(Box::new(move || flag_for_check.load(Ordering::Relaxed))),
            )
            .await;

        match result {
            Ok(()) => {
                ctx.update_progress("completed", 100, "提炼完成");
                Ok(TaskResult {
                    success: true,
                    result_json: Some(
                        serde_json::json!({ "guidebook_id": guidebook_id }).to_string(),
                    ),
                    error_message: None,
                })
            }
            Err(e) => {
                let _ = GuidebookRepository::new(self.pool.clone())
                    .update_error(&guidebook_id, &e.to_string());
                Ok(TaskResult {
                    success: false,
                    result_json: None,
                    error_message: Some(e.to_string()),
                })
            }
        }
    }
}
```

`TaskResult` 字段（success/result_json/error_message）以 book_deconstruction/executor.rs 实际用法为准。

- [ ] **Step 4: commands.rs**

```rust
//! 指导书提炼 Tauri Commands

use std::sync::Arc;

use tauri::{command, AppHandle, Manager};

use super::{
    models::*,
    repository::CustomMethodologyRepository,
    service::{GuidebookDistillationService, GuidebookStatusResponse},
};
use crate::{
    db::DbPool, domain::methodology::MethodologyType, error::AppError, llm::LlmService,
    subscription::SubscriptionService,
};

fn new_service(app_handle: &AppHandle) -> Result<GuidebookDistillationService, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let llm_service = LlmService::new(app_handle.clone());
    Ok(GuidebookDistillationService::new(
        pool,
        llm_service,
        app_handle.clone(),
    ))
}

fn get_user_id(app_handle: &AppHandle) -> String {
    let app_dir = app_handle.path().app_data_dir().unwrap_or_default();
    let machine_id_path = app_dir.join(".machine_id");
    if machine_id_path.exists() {
        std::fs::read_to_string(&machine_id_path)
            .unwrap_or_default()
            .trim()
            .to_string()
    } else {
        "local".to_string()
    }
}

/// 上传指导书并开始提炼
#[command(rename_all = "snake_case")]
pub async fn upload_guidebook(file_path: String, app_handle: AppHandle) -> Result<String, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let user_id = get_user_id(&app_handle);
    let subscription = SubscriptionService::new(pool.clone());
    if !subscription.has_feature_access(&user_id, "guidebook_distillation")? {
        return Err(AppError::subscription_required(
            "guidebook_distillation",
            "指导书提炼功能需要 Pro 订阅，请升级以继续使用",
        ));
    }
    let service = new_service(&app_handle)?;
    service
        .upload_and_distill(std::path::Path::new(&file_path))
        .await
        .map_err(AppError::from)
}

#[command(rename_all = "snake_case")]
pub async fn get_guidebook_distillation_status(
    guidebook_id: String,
    app_handle: AppHandle,
) -> Result<GuidebookStatusResponse, AppError> {
    new_service(&app_handle)?.get_status(&guidebook_id)
}

#[command(rename_all = "snake_case")]
pub async fn get_guidebook_result(
    guidebook_id: String,
    app_handle: AppHandle,
) -> Result<GuidebookResult, AppError> {
    new_service(&app_handle)?.get_result(&guidebook_id)
}

#[command(rename_all = "snake_case")]
pub async fn list_guidebooks(app_handle: AppHandle) -> Result<Vec<GuidebookListItem>, AppError> {
    new_service(&app_handle)?.list_guidebooks()
}

#[command(rename_all = "snake_case")]
pub async fn delete_guidebook(
    guidebook_id: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    new_service(&app_handle)?.delete_guidebook(&guidebook_id)
}

#[command(rename_all = "snake_case")]
pub async fn cancel_guidebook_distillation(
    guidebook_id: String,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    new_service(&app_handle)?.cancel_distillation(&guidebook_id)
}

// ==================== 方法论清单与自定义方法论管理 ====================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MethodologyInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub max_steps: i32,
    pub is_custom: bool,
    pub source_book: Option<String>,
    pub enabled: bool,
}

/// 全量方法论清单：无 + 5 内置 + 自定义（含禁用，前端标记）
#[command(rename_all = "snake_case")]
pub async fn list_all_methodologies(app_handle: AppHandle) -> Result<Vec<MethodologyInfo>, AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let mut out = vec![MethodologyInfo {
        id: String::new(),
        name: "无（自由创作）".to_string(),
        description: "不指定特定方法论，AI 自由发挥".to_string(),
        max_steps: 1,
        is_custom: false,
        source_book: None,
        enabled: true,
    }];
    for mt in crate::creative_engine::methodology::MethodologyEngine::list_available() {
        let id = crate::domain::methodology::methodology_type_id(&mt);
        out.push(MethodologyInfo {
            id: id.to_string(),
            name: mt.name().to_string(),
            description: mt.description().to_string(),
            max_steps: crate::domain::methodology::methodology_max_steps(id),
            is_custom: false,
            source_book: None,
            enabled: true,
        });
    }
    let cm_repo = CustomMethodologyRepository::new(pool.clone());
    let guidebook_repo = super::repository::GuidebookRepository::new(pool);
    for cm in cm_repo.list_all().map_err(|e| AppError::database(e.to_string()))? {
        let source_book = cm
            .guidebook_id
            .as_deref()
            .and_then(|gid| guidebook_repo.get_by_id(gid).ok().flatten())
            .map(|g| g.title);
        out.push(MethodologyInfo {
            id: cm.id,
            name: cm.name,
            description: cm.description.unwrap_or_default(),
            max_steps: cm.max_steps(),
            is_custom: true,
            source_book,
            enabled: cm.enabled,
        });
    }
    Ok(out)
}

#[command(rename_all = "snake_case")]
pub async fn update_custom_methodology(
    id: String,
    name: Option<String>,
    description: Option<String>,
    steps: Option<Vec<MethodologyStep>>,
    enabled: Option<bool>,
    app_handle: AppHandle,
) -> Result<(), AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    CustomMethodologyRepository::new(pool)
        .update(
            &id,
            name.as_deref(),
            description.as_deref(),
            steps.as_deref(),
            enabled,
        )
        .map_err(|e| AppError::database(e.to_string()))
}

/// 删除自定义方法论：引用它的故事 methodology_id 置空
#[command(rename_all = "snake_case")]
pub async fn delete_custom_methodology(id: String, app_handle: AppHandle) -> Result<(), AppError> {
    let pool = app_handle.state::<DbPool>().inner().clone();
    let repo = CustomMethodologyRepository::new(pool);
    repo.clear_story_references(&id)
        .map_err(|e| AppError::database(e.to_string()))?;
    repo.delete(&id)
        .map_err(|e| AppError::database(e.to_string()))
}
```

- [ ] **Step 5: mod.rs 更新**

```rust
pub mod commands;
pub mod distiller;
pub mod executor;
pub mod models;
pub mod repository;
pub mod service;

pub use models::*;
pub use repository::{CustomMethodologyRepository, GuidebookRepository};
pub use service::render_custom_methodology_extension;
```

- [ ] **Step 6: lib.rs 注册 executor + handlers.rs 注册命令**

`src-tauri/src/lib.rs`（约 409 行 `task_service.register_executor(executor);` 之后）：

```rust
    let guidebook_executor = std::sync::Arc::new(
        guidebook_distillation::executor::GuidebookDistillationExecutor::new(
            pool.clone(),
            llm::LlmService::new(app_handle.clone()),
            app_handle.clone(),
        ),
    );
    task_service.register_executor(guidebook_executor);
```

`src-tauri/src/handlers.rs`（196-201 行 book_deconstruction 命令附近）添加：

```rust
    guidebook_distillation::commands::upload_guidebook,
    guidebook_distillation::commands::get_guidebook_distillation_status,
    guidebook_distillation::commands::get_guidebook_result,
    guidebook_distillation::commands::list_guidebooks,
    guidebook_distillation::commands::delete_guidebook,
    guidebook_distillation::commands::cancel_guidebook_distillation,
    guidebook_distillation::commands::list_all_methodologies,
    guidebook_distillation::commands::update_custom_methodology,
    guidebook_distillation::commands::delete_custom_methodology,
```

- [ ] **Step 7: 编译 + 测试验证**

Run: `cd src-tauri && cargo test --lib`
Expected: 编译通过、1179+ 测试全绿（本任务无新单测，靠编译与既有回归把关）。

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/guidebook_distillation/ src-tauri/src/task_system/models.rs src-tauri/src/lib.rs src-tauri/src/handlers.rs src-tauri/src/subscription/ src-tauri/src/task_system/
git commit -m "feat: 指导书提炼服务、命令与任务执行器接入"
```

---

### Task 6: 创作链接入 — 自定义方法论与内置同等待遇

**Files:**
- Modify: `src-tauri/src/domain/methodology.rs`（加常量/谓词/`methodology_type_id`）
- Modify: `src-tauri/src/creative_engine/write_time_bundle.rs:209` 附近（load_sync 的方法论注入分支）
- Modify: `src-tauri/src/scene_commands.rs:211` 附近（advance_methodology_step 的 custom 分支）
- Modify: `src-tauri/src/strategy/asset_catalog.rs:21`（methodology_assets 支持 custom）

**Interfaces:**
- Consumes: `render_custom_methodology_extension`（Task 5）、`CustomMethodologyRepository`（Task 2）。
- Produces:
  - `crate::domain::methodology::CUSTOM_METHODOLOGY_PREFIX: &str = "custom_"`
  - `crate::domain::methodology::is_custom_methodology_id(id: &str) -> bool`
  - `crate::domain::methodology::methodology_type_id(mt: &MethodologyType) -> &'static str`

- [ ] **Step 1: 写失败测试**

`src-tauri/src/domain/methodology.rs` 的 tests mod 追加：

```rust
    #[test]
    fn custom_methodology_id_predicate() {
        assert!(is_custom_methodology_id("custom_abc123"));
        assert!(!is_custom_methodology_id("snowflake"));
        assert_eq!(CUSTOM_METHODOLOGY_PREFIX, "custom_");
    }

    #[test]
    fn methodology_type_id_roundtrip() {
        assert_eq!(methodology_type_id(&MethodologyType::Snowflake), "snowflake");
        assert_eq!(
            methodology_type_id(&MethodologyType::HighDensityWorldBuilding),
            "high_density_world_building"
        );
        assert_eq!(methodology_type_id(&MethodologyType::HeroJourney), "hero_journey");
    }
```

Run: `cd src-tauri && cargo test --lib methodology` → Expected: FAIL（函数未定义）。

- [ ] **Step 2: domain/methodology.rs 实现**

在 `normalize_methodology_id` 之前添加：

```rust
/// 自定义（指导书提炼）方法论的 id 前缀
pub const CUSTOM_METHODOLOGY_PREFIX: &str = "custom_";

/// 是否为自定义方法论 id
pub fn is_custom_methodology_id(id: &str) -> bool {
    id.starts_with(CUSTOM_METHODOLOGY_PREFIX)
}

/// MethodologyType 枚举 → canonical 字符串 id
pub fn methodology_type_id(mt: &MethodologyType) -> &'static str {
    match mt {
        MethodologyType::Snowflake => "snowflake",
        MethodologyType::SceneStructure => "scene_structure",
        MethodologyType::HeroJourney => "hero_journey",
        MethodologyType::CharacterDepth => "character_depth",
        MethodologyType::HighDensityWorldBuilding => "high_density_world_building",
    }
}
```

Run: `cd src-tauri && cargo test --lib methodology` → Expected: PASS。

- [ ] **Step 3: write_time_bundle.rs 续写注入分支**

找到 load_sync 中的方法论加载段（约 209 行，注释 `// v0.31.0: 加载方法论扩展` 处）：

旧代码：

```rust
        let methodology_extension = match story.methodology_id.as_deref() {
            Some(mid) if !mid.is_empty() => {
                let step = story.methodology_step.unwrap_or(1);
                resolve_methodology_extension(mid, step)
            }
            _ => None,
        };
```

新代码：

```rust
        let methodology_extension = match story.methodology_id.as_deref() {
            Some(mid) if !mid.is_empty() => {
                let step = story.methodology_step.unwrap_or(1);
                if crate::domain::methodology::is_custom_methodology_id(mid) {
                    // 自定义（指导书提炼）方法论：从 DB 渲染当前步骤
                    crate::guidebook_distillation::render_custom_methodology_extension(
                        &pool, mid, step,
                    )
                } else {
                    resolve_methodology_extension(mid, step)
                }
            }
            _ => None,
        };
```

注意：确认该作用域内 `pool` 变量的确切名称（其下方 `GenreProfileRepository::new(pool.clone())` 说明有 `pool` 可用；若实际是 `self.pool` 或其他名字，按其调整）。

- [ ] **Step 4: scene_commands.rs 步骤推进分支**

`advance_methodology_step`（约 200-220 行），在 `let next = crate::domain::methodology::next_methodology_step(mid, current);` 处改为：

```rust
    let next = if crate::domain::methodology::is_custom_methodology_id(mid) {
        // 自定义方法论：最大步数 = 步骤数，到顶停留
        let max = crate::guidebook_distillation::CustomMethodologyRepository::new(pool.clone())
            .get_by_id(mid)
            .ok()
            .flatten()
            .map(|cm| cm.max_steps())
            .unwrap_or(1);
        (current.max(1) + 1).min(max)
    } else {
        crate::domain::methodology::next_methodology_step(mid, current)
    };
```

- [ ] **Step 5: asset_catalog.rs 策略选择器可见**

`methodology_assets()` 改签名并追加自定义方法论：

```rust
/// 把创作方法论转换为可选择资产（含指导书提炼的自定义方法论）
pub fn methodology_assets(pool: Option<&crate::db::DbPool>) -> Vec<SelectableAsset> {
    let mut assets = MethodologyEngine::list_available()
        .into_iter()
        .map(|mt| {
            let id = format!("methodology.{}", methodology_id(mt));
            SelectableAsset {
                id: id.clone(),
                kind: AssetKind::Methodology,
                name: mt.name().to_string(),
                description: mt.description().to_string(),
                when_to_use: methodology_when_to_use(mt),
                input_description: Some(
                    "故事概念、目标字数、当前创作阶段（世界观/大纲/场景/正文）".to_string(),
                ),
                output_description: Some("该方法论的 system prompt 扩展与步骤指引".to_string()),
                payload: serde_json::json!({
                    "methodology_type": mt,
                    "id": methodology_id(mt),
                }),
                metadata: Default::default(),
            }
        })
        .collect::<Vec<_>>();

    if let Some(pool) = pool {
        if let Ok(customs) =
            crate::guidebook_distillation::CustomMethodologyRepository::new(pool.clone())
                .list_enabled()
        {
            for cm in customs {
                assets.push(SelectableAsset {
                    id: format!("methodology.{}", cm.id),
                    kind: AssetKind::Methodology,
                    name: cm.name.clone(),
                    description: cm.description.clone().unwrap_or_default(),
                    when_to_use: format!(
                        "由指导书提炼的自定义方法论（{} 个步骤）。{}",
                        cm.steps.len(),
                        cm.description.clone().unwrap_or_default()
                    ),
                    input_description: Some(
                        "故事概念、目标字数、当前创作阶段（世界观/大纲/场景/正文）".to_string(),
                    ),
                    output_description: Some(
                        "该方法论的步骤指引与检查清单".to_string(),
                    ),
                    payload: serde_json::json!({
                        "id": cm.id,
                        "custom": true,
                    }),
                    metadata: Default::default(),
                });
            }
        }
    }

    assets
}
```

更新所有调用点：Run `grep -rn "methodology_assets" src-tauri/src --include=*.rs`，把每个 `methodology_assets()` 改为 `methodology_assets(Some(&pool))`（若调用点有 pool；没有 pool 的传 `None` 并在代码注释说明）。编译器会指出遗漏处。

- [ ] **Step 6: 写 custom 注入的单元测试**

`src-tauri/src/guidebook_distillation/service.rs` 的 tests mod（无则新建）追加：

```rust
#[cfg(test)]
mod extension_tests {
    use super::*;
    use crate::db::connection::create_test_pool;

    fn seed_cm(pool: &DbPool, enabled: bool) {
        CustomMethodologyRepository::new(pool.clone())
            .create(&CustomMethodology {
                id: "custom_t1".into(),
                guidebook_id: None,
                name: "冲突驱动法".into(),
                description: None,
                steps: vec![
                    MethodologyStep {
                        title: "立冲突".into(),
                        instruction: "确立核心冲突".into(),
                        checklist: vec!["冲突明确吗？".into()],
                    },
                    MethodologyStep {
                        title: "升级".into(),
                        instruction: "升级冲突".into(),
                        checklist: vec![],
                    },
                ],
                enabled,
                created_at: chrono::Local::now(),
                updated_at: chrono::Local::now(),
            })
            .unwrap();
    }

    #[test]
    fn render_extension_picks_step_and_formats() {
        let pool = create_test_pool().unwrap();
        seed_cm(&pool, true);
        let text = render_custom_methodology_extension(&pool, "custom_t1", 2).unwrap();
        assert!(text.contains("冲突驱动法"));
        assert!(text.contains("第2步：升级"));
        assert!(text.contains("升级冲突"));
        // 越界 step 钳到最后一步
        let clamped = render_custom_methodology_extension(&pool, "custom_t1", 99).unwrap();
        assert!(clamped.contains("第2步"));
        // 第 1 步带检查清单
        let step1 = render_custom_methodology_extension(&pool, "custom_t1", 1).unwrap();
        assert!(step1.contains("检查清单"));
        assert!(step1.contains("冲突明确吗？"));
    }

    #[test]
    fn render_extension_none_when_disabled_or_unknown() {
        let pool = create_test_pool().unwrap();
        seed_cm(&pool, false);
        assert!(render_custom_methodology_extension(&pool, "custom_t1", 1).is_none());
        assert!(render_custom_methodology_extension(&pool, "custom_unknown", 1).is_none());
    }
}
```

Run: `cd src-tauri && cargo test --lib extension_tests` → Expected: PASS。

- [ ] **Step 7: 全量回归**

Run: `cd src-tauri && cargo test --lib`
Expected: 全绿（含新测试）。

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/domain/methodology.rs src-tauri/src/creative_engine/write_time_bundle.rs src-tauri/src/scene_commands.rs src-tauri/src/strategy/ src-tauri/src/guidebook_distillation/
git commit -m "feat: 自定义方法论接入创作链——续写注入/步骤推进/策略选择器"
```

---

### Task 7: 前端 — 指导书提炼面板 + 方法论清单动态化

**Files:**
- Create: `src-frontend/src/types/guidebook-distillation.ts`
- Create: `src-frontend/src/hooks/useGuidebookDistillation.ts`
- Create: `src-frontend/src/hooks/useMethodologies.ts`
- Create: `src-frontend/src/components/guidebook-distillation/GuidebookDistillationPanel.tsx`
- Modify: `src-frontend/src/pages/BookDeconstruction.tsx`（加 Tab）
- Modify: `src-frontend/src/pages/settings/MethodologySettings.tsx:7`（删硬编码 METHODOLOGIES，改 hook）
- Modify: `src-frontend/src/pages/Stories.tsx:784` 附近（方法论 select 改 hook）
- Test: `src-frontend/src/hooks/__tests__/useMethodologies.test.tsx`

**Interfaces:**
- Consumes: Task 5 的 9 个 Tauri 命令；事件 `guidebook-distillation-progress`。
- Produces: `useAllMethodologies()`（返回 `MethodologyInfo[]`）、`GuidebookDistillationPanel` 组件。

- [ ] **Step 1: types/guidebook-distillation.ts**

```typescript
export interface MethodologyStep {
  title: string;
  instruction: string;
  checklist: string[];
}

export interface CustomMethodology {
  id: string;
  guidebook_id: string | null;
  name: string;
  description: string | null;
  steps: MethodologyStep[];
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface Guidebook {
  id: string;
  title: string;
  author: string | null;
  subject: string | null;
  word_count: number | null;
  file_format: string | null;
  file_hash: string | null;
  file_path: string | null;
  methodology_id: string | null;
  status: string;
  progress: number;
  error: string | null;
  task_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface GuidebookListItem {
  id: string;
  title: string;
  author: string | null;
  subject: string | null;
  word_count: number | null;
  file_format: string | null;
  methodology_id: string | null;
  status: string;
  progress: number;
  created_at: string;
}

export interface GuidebookStatusResponse {
  guidebook_id: string;
  status: string;
  progress: number;
  current_step: string | null;
  error: string | null;
}

export interface GuidebookResult {
  guidebook: Guidebook;
  methodology: CustomMethodology | null;
}

export interface DistillationProgressEvent {
  guidebook_id: string;
  status: string;
  progress: number;
  current_step: string;
  message: string | null;
  active_threads?: number;
}

export interface MethodologyInfo {
  id: string;
  name: string;
  description: string;
  max_steps: number;
  is_custom: boolean;
  source_book: string | null;
  enabled: boolean;
}
```

- [ ] **Step 2: hooks/useGuidebookDistillation.ts**

模式照搬 `useBookDeconstruction.ts`（含 `guidebook-distillation-progress` 事件监听 + 3s 轮询兜底；拆书那套还监听 pipeline-progress/task-progress，提炼不需要 pipeline-progress，保留 task-status-changed 即可，简单起见可以只做事件 + 轮询两通道）：

```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { loggedInvoke } from '@/services/tauri';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import type {
  GuidebookListItem,
  GuidebookResult,
  GuidebookStatusResponse,
  DistillationProgressEvent,
  MethodologyStep,
} from '@/types/guidebook-distillation';

const GUIDEBOOKS_KEY = 'guidebooks';
const RESULT_KEY = 'guidebook-result';
const DISTILL_STATUS_KEY = 'guidebook-distill-status';

export function useUploadGuidebook() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (filePath: string) => {
      return await loggedInvoke<string>('upload_guidebook', { file_path: filePath });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [GUIDEBOOKS_KEY] });
    },
  });
}

export function useGuidebooks() {
  return useQuery({
    queryKey: [GUIDEBOOKS_KEY],
    queryFn: async () => {
      return await loggedInvoke<GuidebookListItem[]>('list_guidebooks');
    },
  });
}

export function useGuidebookDistillationStatus(guidebookId: string | null) {
  const [liveStatus, setLiveStatus] = useState<GuidebookStatusResponse | null>(null);

  useEffect(() => {
    if (!guidebookId) return;
    let unlisten: (() => void) | undefined;
    const setup = async () => {
      unlisten = await listen<DistillationProgressEvent>(
        'guidebook-distillation-progress',
        event => {
          if (event.payload.guidebook_id === guidebookId) {
            setLiveStatus({
              guidebook_id: guidebookId,
              status: event.payload.status,
              progress: event.payload.progress,
              current_step: event.payload.current_step,
              error: undefined,
            });
          }
        }
      );
    };
    setup();
    return () => {
      if (unlisten) unlisten();
    };
  }, [guidebookId]);

  const query = useQuery({
    queryKey: [DISTILL_STATUS_KEY, guidebookId],
    queryFn: async () => {
      if (!guidebookId) return null;
      return await loggedInvoke<GuidebookStatusResponse>('get_guidebook_distillation_status', {
        guidebook_id: guidebookId,
      });
    },
    refetchInterval: query => {
      const data = query.state.data;
      if (!data) return false;
      return ['pending', 'extracting', 'distilling', 'merging'].includes(data.status)
        ? 3000
        : false;
    },
    enabled: !!guidebookId,
  });

  return liveStatus ?? query.data ?? null;
}

export function useGuidebookResult(guidebookId: string | null) {
  return useQuery({
    queryKey: [RESULT_KEY, guidebookId],
    queryFn: async () => {
      if (!guidebookId) return null;
      return await loggedInvoke<GuidebookResult>('get_guidebook_result', {
        guidebook_id: guidebookId,
      });
    },
    enabled: !!guidebookId,
  });
}

export function useDeleteGuidebook() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (guidebookId: string) => {
      await loggedInvoke<void>('delete_guidebook', { guidebook_id: guidebookId });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [GUIDEBOOKS_KEY] });
    },
  });
}

export function useCancelGuidebookDistillation() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (guidebookId: string) => {
      await loggedInvoke<void>('cancel_guidebook_distillation', {
        guidebook_id: guidebookId,
      });
    },
    onSuccess: (_, guidebookId) => {
      queryClient.invalidateQueries({ queryKey: [DISTILL_STATUS_KEY, guidebookId] });
      queryClient.invalidateQueries({ queryKey: [GUIDEBOOKS_KEY] });
    },
  });
}

export function useUpdateCustomMethodology() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (input: {
      id: string;
      name?: string;
      description?: string;
      steps?: MethodologyStep[];
      enabled?: boolean;
    }) => {
      const { id, ...rest } = input;
      await loggedInvoke<void>('update_custom_methodology', { id, ...rest });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [RESULT_KEY] });
      queryClient.invalidateQueries({ queryKey: ['all-methodologies'] });
    },
  });
}

export function useDeleteCustomMethodology() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      await loggedInvoke<void>('delete_custom_methodology', { id });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [GUIDEBOOKS_KEY] });
      queryClient.invalidateQueries({ queryKey: ['all-methodologies'] });
    },
  });
}
```

- [ ] **Step 3: hooks/useMethodologies.ts**

```typescript
import { useQuery } from '@tanstack/react-query';
import { loggedInvoke } from '@/services/tauri';
import type { MethodologyInfo } from '@/types/guidebook-distillation';

/** 全量方法论清单：无 + 5 内置 + 自定义（含禁用项，前端自行过滤或标记） */
export function useAllMethodologies() {
  return useQuery({
    queryKey: ['all-methodologies'],
    queryFn: async () => {
      return await loggedInvoke<MethodologyInfo[]>('list_all_methodologies');
    },
  });
}
```

- [ ] **Step 4: GuidebookDistillationPanel.tsx**

结构照搬 `BookDeconstruction.tsx` 主区（上传按钮 + 卡片列表 + 选中详情）。组件职责：列表/上传/进度/结果编辑一体。实现要点（完整代码按此写，样式类名参照 `BookListGrid`/`AnalysisProgress` 同款 tailwind 风格）：

- 顶部：标题"指导书提炼"+ 说明文案"上传故事创作指导书（txt/pdf/epub），自动提炼为可调用的创作方法论" + "上传指导书"按钮。
- 上传：`@tauri-apps/plugin-dialog` 的 `open({ filters: [{ name: '指导书', extensions: ['txt','pdf','epub'] }] })`（照搬 `BookUploadPanel.tsx` 的调用方式），成功后调 `useUploadGuidebook`。
- 列表：`useGuidebooks()` 渲染卡片（书名/作者/字数/状态徽章/进度条）；进行中的卡片显示 `useGuidebookDistillationStatus(id)` 的进度与当前步骤 + 取消按钮（`useCancelGuidebookDistillation`）。
- 选中已完成项：下方展示 `useGuidebookResult(id)` 的方法论编辑器——
  - 名称 `<input>`、描述 `<textarea>`
  - 步骤列表：每步 title `<input>`、instruction `<textarea>`、checklist `<textarea>`（一行一条，保存时 split('\n') 过滤空行）
  - 启用开关（checkbox）、保存按钮（`useUpdateCustomMethodology`）、删除方法论按钮（confirm 后 `useDeleteCustomMethodology`，文案提示"引用它的故事将恢复为无方法论"）
- 删除指导书：卡片上的删除按钮（confirm，提示"提炼出的方法论会保留但失去来源关联"）。

- [ ] **Step 5: BookDeconstruction.tsx 加 Tab**

在该组件加 state 与 Tab 头（放在现有搜索/上传工具栏上方）：

```tsx
  const [activeTab, setActiveTab] = useState<'books' | 'guidebooks'>('books');
```

Tab 按钮区（样式用现有 `cn` + tailwind 按钮风格）：

```tsx
      <div className="flex gap-2 border-b border-gray-200 dark:border-gray-700 mb-4">
        {([
          { key: 'books', label: '书籍拆解' },
          { key: 'guidebooks', label: '指导书提炼' },
        ] as const).map(tab => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={cn(
              'px-4 py-2 text-sm font-medium border-b-2 -mb-px',
              activeTab === tab.key
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            )}
          >
            {tab.label}
          </button>
        ))}
      </div>
```

然后把现有主体内容包进 `{activeTab === 'books' && (<>...</>)}`，并加 `{activeTab === 'guidebooks' && <GuidebookDistillationPanel />}`（import 自 `@/components/guidebook-distillation/GuidebookDistillationPanel`）。

- [ ] **Step 6: MethodologySettings.tsx 动态化**

- 删除第 7-34 行的 `const METHODOLOGIES = [...]`。
- 组件内加 `const { data: methodologies } = useAllMethodologies();`（import 自 `@/hooks/useMethodologies`）。
- 渲染清单处（原来 map `METHODOLOGIES`）改为 map `methodologies ?? []`：
  - `id === ''` 的"无"项照常渲染；
  - `is_custom` 项名称后追加来源徽标：`{m.is_custom && m.source_book && <span className="...">来自《{m.source_book}》</span>}`；
  - `!m.enabled` 的自定义项不渲染（或渲染为禁用态，选简单的不渲染）。
- 步骤指示：`snowflake` / `high_density_world_building` 保留现有 SNOWFLAKE_STEPS / WORLD_BUILDING_PHASES 硬编码展示；选中项 `is_custom && max_steps > 1` 时显示一行文本：`当前：第 {methodologyStep} 步 / 共 {m.max_steps} 步`。
- `handleSelectMethodology` 的 `isStructured` 判断改为：`const target = methodologies?.find(m => m.id === canonical); const isStructured = (target?.max_steps ?? 1) > 1;`

- [ ] **Step 7: Stories.tsx 方法论 select 动态化**

约 784 行的 `<select>` 选项（`<option value="">无</option>` + 5 个硬编码 option，到约 795 行）改为：

```tsx
  const { data: methodologies } = useAllMethodologies();
  ...
  {(methodologies ?? [])
    .filter(m => m.enabled)
    .map(m => (
      <option key={m.id} value={m.id}>
        {m.name}
        {m.is_custom && m.source_book ? `（${m.source_book}）` : ''}
      </option>
    ))}
```

import `useAllMethodologies`。注意保留 id 为空字符串的"无"项（后端清单首项即是 `id: ""`）。

- [ ] **Step 8: 前端测试**

`src-frontend/src/hooks/__tests__/useMethodologies.test.tsx`（mock 模式参照同目录 `useSettings.test.tsx`）：

```tsx
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { describe, expect, it, vi } from 'vitest';
import React from 'react';

vi.mock('@/services/tauri', () => ({
  loggedInvoke: vi.fn(),
}));

import { loggedInvoke } from '@/services/tauri';
import { useAllMethodologies } from '../useMethodologies';

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
    {children}
  </QueryClientProvider>
);

describe('useAllMethodologies', () => {
  it('调用 list_all_methodologies 并返回清单', async () => {
    vi.mocked(loggedInvoke).mockResolvedValue([
      { id: '', name: '无（自由创作）', description: '', max_steps: 1, is_custom: false, source_book: null, enabled: true },
      { id: 'snowflake', name: '雪花写作法', description: 'd', max_steps: 10, is_custom: false, source_book: null, enabled: true },
      { id: 'custom_x', name: '冲突驱动法', description: 'd', max_steps: 3, is_custom: true, source_book: '故事技巧', enabled: true },
    ]);
    const { result } = renderHook(() => useAllMethodologies(), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(loggedInvoke).toHaveBeenCalledWith('list_all_methodologies');
    expect(result.current.data).toHaveLength(3);
    expect(result.current.data?.[2].is_custom).toBe(true);
  });
});
```

Run: `cd src-frontend && npx vitest run useMethodologies` → Expected: PASS。

- [ ] **Step 9: 前端全量验证**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run`
Expected: 类型检查通过，367+ 测试全绿。

- [ ] **Step 10: Commit**

```bash
git add src-frontend/src/types/guidebook-distillation.ts src-frontend/src/hooks/ src-frontend/src/components/guidebook-distillation/ src-frontend/src/pages/
git commit -m "feat: 指导书提炼面板与方法论清单动态化"
```

---

### Task 8: 全量回归 + CHANGELOG

**Files:**
- Modify: `CHANGELOG.md`（顶部加条目）

- [ ] **Step 1: 全量回归**

```bash
cd src-tauri && cargo test --lib
cd ../src-frontend && npx tsc --noEmit && npx vitest run
```

Expected: Rust 1179+ 全绿；前端 367+ 全绿。

- [ ] **Step 2: 手工冒烟（开发者本机）**

`cd src-tauri && cargo tauri dev` 启动后：
1. 拆书页 → "指导书提炼" Tab → 上传一本 txt 指导书 → 观察进度事件推进到完成；
2. 完成后再看方法论编辑器（名称/步骤/检查清单可编辑、启停可切换）；
3. 故事设置 → 创作方法论：能看到自定义方法论（带来源徽标），选中保存；
4. 续写一段：日志（或覆盖率统计 `methodology_extension`）确认自定义方法论约束被注入；
5. 完成一章：确认 `methodology_step` 推进 +1 且不超过步骤数。

- [ ] **Step 3: CHANGELOG**

`CHANGELOG.md` 顶部（最新条目之上）追加：

```markdown
## [Unreleased]

### 新增
- 指导书提炼：上传故事创作指导书（txt/pdf/epub），自动提炼核心内容为带步骤的自定义创作方法论（名称/描述/分步指引/检查清单），可在故事设置与创建向导中选用，续写时按当前步骤注入约束并随章节完成自动推进，策略选择器可自动挑选；支持编辑、启停与删除（删除时引用故事自动恢复为无方法论）
```

- [ ] **Step 4: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs: CHANGELOG 记录指导书提炼功能"
```

---

## 自审记录（writing-plans checklist）

- Spec 覆盖：设计文档 §1 模块（Task 2/4/5）、§2 表（Task 1）、§3 创作链接入 4 处（Task 6）、§4 前端（Task 7）、§5 错误处理（取消/重试/空产物散见 Task 4/5）、§6 测试（每个 Task 内嵌）。无遗漏。
- 占位符扫描：Task 5 有两处"以实际 API 为准"的注意事项（`AppError` 构造器、`TaskService::cancel_task`、`TaskResult` 字段）——这些是照搬邻近代码的对齐点，已在文本中给出确切的参照文件与方法名，实现者按编译器指引对齐即可。
- 类型一致性：`CustomMethodology.steps` 为 `Vec<MethodologyStep>`（Task 2 定义，Task 5 从 `LlmMethodologyStepResponse` 转换、Task 6 渲染、Task 7 TS 镜像）；`render_custom_methodology_extension(pool, id, step)` 签名在 Task 5 定义、Task 6 调用一致；`methodology_assets(pool: Option<&DbPool>)` 在 Task 6 定义并要求更新全部调用点；`useAllMethodologies` 在 Task 7 Step 3 定义、Step 6/7 消费一致。
