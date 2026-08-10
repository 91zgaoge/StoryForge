# 弹性扩张：续写侧创作资产强关联 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让续写流程通过「轮换账本数据 + 扩张债务弹性配额 + 动态资产菜单」强制模型活用创作资产，解决角色少、场景单一、冲突弱、推进慢的问题。

**Architecture:** 新建 `creative_engine/expansion/` 模块（ledger/debt/asset_menu 三件套，纯 Rust 零 LLM 成本），产出注入到现有 beat_planner prompt 模板（条件块）与 WriteTimeBundle 新段（⑧e）；beat 计划结构复用既有字段并新增 `character_moves`/`selected_asset_ids`；资产选用历史存 `stories.asset_history_json`（V122）用于轮换排除。

**Tech Stack:** Rust（src-tauri）、rusqlite/r2d2 SQLite、serde_json、tera 风格模板（`{{#if}}` 条件块，`TemplateEngine::render_with_conditions`）。

## Global Constraints

- 仓库 `/Users/yuzaimu/projects/StoryForge`，master 直接工作；中文 conventional commit；**只提交，不推送、不打 tag、不改版本号**（并入未发布的 v0.34.0）
- `.recovery/` 目录勿动勿提交；pre-commit 钩子（cargo +nightly fmt + prettier）不得绕过
- 每个任务提交前：`cd src-tauri && cargo +nightly fmt`；测试基线：`cargo test --lib` = **1217 passed / 2 ignored**（不得回归）
- DB 测试统一用 `crate::db::connection::create_test_pool()`（内存库跑全量迁移，`src-tauri/src/db/connection.rs:15-58`）
- prompt 一律走 `crate::prompts::registry::resolve_prompt` + 内置 md 兜底，**禁止在 Rust 中内联硬编码完整 prompt 模板**（executor.rs:1618-1619 既有约定）；模板 front-matter version  bump 到 0.34.0
- 设计文档：`docs/plans/2026-08-09-elastic-expansion-asset-fusion-design.md`
- 与设计的一处已确认偏差：资产选用历史存 `stories.asset_history_json` 而非章节元数据（设计文档写"章节元数据"，因 beat 规划时章节行尚不存在；Task 7 回写设计文档）

---

### Task 1: expansion 模块骨架 + V122 migration + 资产历史存取

**Files:**
- Create: `src-tauri/src/db/migrations/V122__stories_asset_history.sql`
- Modify: `src-tauri/src/db/migrations/mod.rs`（注册 V122，仿 V119 纯 SQL 注册的既有写法）
- Create: `src-tauri/src/creative_engine/expansion/mod.rs`
- Modify: `src-tauri/src/creative_engine/mod.rs`（加 `pub mod expansion;`）

**Interfaces:**
- Consumes: `crate::db::connection::DbPool`（r2d2 SQLite 连接池）、`create_test_pool()`
- Produces（后续任务依赖）:
  - `crate::creative_engine::expansion::{RotationLedger, SceneUsage, CharacterSilence, ExpansionDebt, AssetHistoryEntry}` 类型
  - `pub fn read_asset_history(conn: &rusqlite::Connection, story_id: &str) -> Vec<AssetHistoryEntry>`
  - `pub fn append_asset_history(pool: &DbPool, story_id: &str, chapter: i32, ids: &[String]) -> Result<(), String>`（保留最近 10 条）
  - DB 新列 `stories.asset_history_json TEXT`（NULL=无历史）

- [ ] **Step 1: V122 migration**

`src-tauri/src/db/migrations/V122__stories_asset_history.sql`：

```sql
-- v0.34.0 弹性扩张：资产选用历史（JSON 数组 [{chapter, ids}]，用于资产轮换排除）
ALTER TABLE stories ADD COLUMN asset_history_json TEXT;
```

在 `src-tauri/src/db/migrations/mod.rs` 仿 V119 的注册方式注册 V122（先看该文件 V119/V121 怎么注册，纯 SQL 与 Rust 幂等两种先例都有，选与 V119 相同的纯 SQL 方式）。现有最大号 V121，V122 无冲突。

- [ ] **Step 2: 写失败测试**

`src-tauri/src/creative_engine/expansion/mod.rs`（先只有类型与函数签名，测试模块如下）：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn seed_story(pool: &crate::db::connection::DbPool) -> String {
        let repo = crate::db::repositories::StoryRepository::new(pool.clone());
        let req = crate::db::repositories::CreateStoryRequest {
            title: "测试小说".to_string(),
            description: None,
            genre: Some("玄幻".to_string()),
            style_dna_id: None,
            genre_profile_id: None,
            methodology_id: None,
            reference_book_id: None,
        };
        repo.create(req).unwrap().id
    }

    #[test]
    fn asset_history_roundtrip_and_retention() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let sid = seed_story(&pool);
        // 无历史时为空
        {
            let conn = pool.get().unwrap();
            assert!(read_asset_history(&conn, &sid).is_empty());
        }
        // 追加 12 条，只保留最近 10 条
        for ch in 1..=12 {
            append_asset_history(&pool, &sid, ch, &[format!("beat_card.x{}", ch)]).unwrap();
        }
        let conn = pool.get().unwrap();
        let history = read_asset_history(&conn, &sid);
        assert_eq!(history.len(), 10);
        assert_eq!(history.first().unwrap().chapter, 3); // 最旧的 1、2 被淘汰
        assert_eq!(history.last().unwrap().chapter, 12);
        assert_eq!(history.last().unwrap().ids, vec!["beat_card.x12".to_string()]);
    }
}
```

注意：`StoryRepository`/`CreateStoryRequest` 的确切路径与字段以 `src-tauri/src/db/repositories_tests.rs:12-34` 的既有样例为准，若有出入按样例修正。

运行 `cd src-tauri && cargo test expansion` — 预期 FAIL（模块/函数尚不存在，编译错误）。

- [ ] **Step 3: 实现**

`src-tauri/src/creative_engine/expansion/mod.rs`：

```rust
//! v0.34.0 弹性扩张：轮换账本 + 扩张债务 + 动态资产菜单。
//! 设计：docs/plans/2026-08-09-elastic-expansion-asset-fusion-design.md
//! 纯 Rust 零 LLM 成本；产出注入 beat_planner 模板与 WriteTimeBundle。

pub mod asset_menu;
pub mod debt;
pub mod ledger;

use serde::{Deserialize, Serialize};

/// 近 10 章场景使用账（按地点聚合）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneUsage {
    pub location: String,
    pub count: u32,
}

/// 主要角色沉寂账
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterSilence {
    pub character_id: String,
    pub name: String,
    /// 距上次登场的章数（从未登场 = current_sequence）
    pub chapters_absent: u32,
}

/// 轮换账本原始事实（ledger.rs 聚合，debt.rs 消费）
#[derive(Debug, Clone, Default)]
pub struct RotationLedger {
    /// 当前最大 scene sequence_number（无场景 = 0）
    pub current_sequence: i32,
    /// 近 10 章场景使用次数（按次数降序）
    pub scene_usage: Vec<SceneUsage>,
    /// 角色沉寂账（按沉寂章数降序，仅含 absent > 0）
    pub character_silence: Vec<CharacterSilence>,
    /// 最近一次"新地点首次出现"的 sequence（无 = 0）
    pub last_new_location_seq: i32,
    /// 最近一次"新角色登场或沉寂≥3章角色回归"的 sequence（无 = 0）
    pub last_character_refresh_seq: i32,
    /// 尾部连续 character_conflicts 为空的场景数
    pub trailing_conflict_free: u32,
}

/// 四项扩张债务（连续停滞章数）
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExpansionDebt {
    pub conflict: u32,
    pub scene: u32,
    pub character: u32,
    pub foreshadow: u32,
}

/// 资产选用历史条目（stories.asset_history_json）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetHistoryEntry {
    pub chapter: i32,
    pub ids: Vec<String>,
}

const ASSET_HISTORY_KEEP: usize = 10;

/// 读取资产选用历史（NULL/损坏 JSON → 空）
pub fn read_asset_history(conn: &rusqlite::Connection, story_id: &str) -> Vec<AssetHistoryEntry> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT asset_history_json FROM stories WHERE id = ?1",
            rusqlite::params![story_id],
            |r| r.get(0),
        )
        .unwrap_or(None);
    raw.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// 追加一条资产选用历史，只保留最近 10 条
pub fn append_asset_history(
    pool: &crate::db::connection::DbPool,
    story_id: &str,
    chapter: i32,
    ids: &[String],
) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }
    let conn = pool.get().map_err(|e| e.to_string())?;
    let mut history = read_asset_history(&conn, story_id);
    history.push(AssetHistoryEntry {
        chapter,
        ids: ids.to_vec(),
    });
    if history.len() > ASSET_HISTORY_KEEP {
        history = history.split_off(history.len() - ASSET_HISTORY_KEEP);
    }
    let json = serde_json::to_string(&history).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE stories SET asset_history_json = ?1 WHERE id = ?2",
        rusqlite::params![json, story_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
```

`src-tauri/src/creative_engine/mod.rs` 加 `pub mod expansion;`（按该文件既有模块声明的字母序/风格插入）。

- [ ] **Step 4: 测试通过**

运行 `cd src-tauri && cargo test expansion` — 1 例 PASS。`cargo test --lib` 全量不回归（1217+1）。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db/migrations/ src-tauri/src/creative_engine/
git commit -m "新增(创作): expansion 模块骨架 + stories.asset_history_json（V122）资产选用历史"
```

---

### Task 2: 轮换账本聚合（ledger.rs）

**Files:**
- Create: `src-tauri/src/creative_engine/expansion/ledger.rs`

**Interfaces:**
- Consumes: Task 1 的 `RotationLedger/SceneUsage/CharacterSilence`；`scenes` 表（`setting_location`、`characters_present` JSON 数组、`character_conflicts` JSON、`sequence_number`，`src-tauri/src/db/connection.rs:603-631`）；`characters` 表（`connection.rs:213-226`）
- Produces: `impl RotationLedger { pub fn load_sync(pool: &DbPool, story_id: &str) -> Result<Self, String>; pub fn render_for_prompt(&self) -> Option<String>; }`

- [ ] **Step 1: 写失败测试**

ledger.rs 测试模块：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;

    fn seed_story_with_scenes(pool: &DbPool) -> (String, String, String) {
        let repo = crate::db::repositories::StoryRepository::new(pool.clone());
        let req = crate::db::repositories::CreateStoryRequest {
            title: "账本测试".to_string(),
            description: None,
            genre: Some("玄幻".to_string()),
            style_dna_id: None,
            genre_profile_id: None,
            methodology_id: None,
            reference_book_id: None,
        };
        let sid = repo.create(req).unwrap().id;
        let conn = pool.get().unwrap();
        // 两个角色：阿岩（每章都在）、林雪（只在第 1 章出现）
        conn.execute(
            "INSERT INTO characters (id, story_id, name) VALUES ('c1', ?1, '阿岩')",
            params![sid],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO characters (id, story_id, name) VALUES ('c2', ?1, '林雪')",
            params![sid],
        )
        .unwrap();
        // 5 章：第 1-3 章在练功房，第 4 章开辟山道（新地点），第 5 章回练功房
        // 第 1 章林雪登场（c1+c2），第 2-5 章只有阿岩（c1）
        // 第 4、5 章 character_conflicts 为空（尾部连续 2 章无冲突）
        let scenes = [
            (1, "练功房", "[\"c1\",\"c2\"]", "[{\"a\":\"c1\",\"b\":\"c2\",\"nature\":\"师徒\",\"stakes\":\"传承\"}]"),
            (2, "练功房", "[\"c1\"]", "[{\"a\":\"c1\",\"b\":\"x\",\"nature\":\"敌对\",\"stakes\":\"生死\"}]"),
            (3, "练功房", "[\"c1\"]", "[{\"a\":\"c1\",\"b\":\"x\",\"nature\":\"敌对\",\"stakes\":\"生死\"}]"),
            (4, "山道", "[\"c1\"]", "[]"),
            (5, "练功房", "[\"c1\"]", "[]"),
        ];
        for (seq, loc, present, conflicts) in scenes {
            conn.execute(
                "INSERT INTO scenes (id, story_id, sequence_number, title, setting_location, characters_present, character_conflicts, content) \
                 VALUES (?1, ?2, ?3, '章', ?4, ?5, ?6, '正文')",
                params![format!("s{}", seq), sid, seq, loc, present, conflicts],
            )
            .unwrap();
        }
        drop(conn);
        (sid, "c1".to_string(), "c2".to_string())
    }

    #[test]
    fn ledger_aggregates_scene_usage_and_character_silence() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let (sid, _c1, c2) = seed_story_with_scenes(&pool);
        let ledger = RotationLedger::load_sync(&pool, &sid).unwrap();

        assert_eq!(ledger.current_sequence, 5);
        // 近 10 章使用账：练功房 4 次、山道 1 次，降序
        assert_eq!(
            ledger.scene_usage,
            vec![
                SceneUsage { location: "练功房".into(), count: 4 },
                SceneUsage { location: "山道".into(), count: 1 },
            ]
        );
        // 林雪第 1 章登场后沉寂 4 章；阿岩不沉寂
        assert_eq!(ledger.character_silence.len(), 1);
        assert_eq!(ledger.character_silence[0].character_id, c2);
        assert_eq!(ledger.character_silence[0].chapters_absent, 4);
        // 最近一次新地点 = 第 4 章（山道首次出现）
        assert_eq!(ledger.last_new_location_seq, 4);
        // 最近一次角色更新 = 第 1 章（林雪此后再未回归/无新角色）
        assert_eq!(ledger.last_character_refresh_seq, 1);
        // 尾部连续 2 章无冲突
        assert_eq!(ledger.trailing_conflict_free, 2);
    }

    #[test]
    fn empty_story_yields_default_ledger_and_no_prompt_text() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let repo = crate::db::repositories::StoryRepository::new(pool.clone());
        let req = crate::db::repositories::CreateStoryRequest {
            title: "空书".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
            genre_profile_id: None,
            methodology_id: None,
            reference_book_id: None,
        };
        let sid = repo.create(req).unwrap().id;
        let ledger = RotationLedger::load_sync(&pool, &sid).unwrap();
        assert_eq!(ledger.current_sequence, 0);
        assert!(ledger.render_for_prompt().is_none());
    }

    #[test]
    fn render_for_prompt_lists_usage_and_silence() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let (sid, _, _) = seed_story_with_scenes(&pool);
        let ledger = RotationLedger::load_sync(&pool, &sid).unwrap();
        let text = ledger.render_for_prompt().unwrap();
        assert!(text.contains("练功房×4"));
        assert!(text.contains("山道×1"));
        assert!(text.contains("林雪"));
        assert!(text.contains("4 章未登场"));
    }
}
```

运行 `cd src-tauri && cargo test expansion` — 预期 FAIL（编译错误：load_sync/render_for_prompt 不存在）。

- [ ] **Step 2: 实现**

`src-tauri/src/creative_engine/expansion/ledger.rs`：

```rust
//! 轮换账本：场景使用 + 角色沉寂的聚合（零 LLM 成本，纯 SQL/内存计算）。

use super::{CharacterSilence, RotationLedger, SceneUsage};
use crate::db::connection::DbPool;
use rusqlite::params;
use std::collections::{HashMap, HashSet};

/// 使用账统计窗口（近 N 章）
const USAGE_WINDOW: i32 = 10;
/// 渲染时最多列出的场景/角色条数
const RENDER_CAP: usize = 8;

impl RotationLedger {
    pub fn load_sync(pool: &DbPool, story_id: &str) -> Result<Self, String> {
        let conn = pool.get().map_err(|e| e.to_string())?;

        let current: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sequence_number), 0) FROM scenes WHERE story_id = ?1",
                params![story_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if current == 0 {
            return Ok(Self::default());
        }

        // 全量场景（seq, location, characters_present, has_conflict），按章升序。
        // 章节量级百级，行数据小，内存聚合即可。
        let mut stmt = conn
            .prepare(
                "SELECT sequence_number, COALESCE(setting_location, ''), \
                        COALESCE(characters_present, '[]'), COALESCE(character_conflicts, '[]') \
                 FROM scenes WHERE story_id = ?1 ORDER BY sequence_number ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows: Vec<(i32, String, Vec<String>, bool)> = stmt
            .query_map(params![story_id], |r| {
                Ok((
                    r.get::<_, i32>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .map(|(seq, loc, present_json, conflicts_json)| {
                let present: Vec<String> =
                    serde_json::from_str(&present_json).unwrap_or_default();
                let conflicts: serde_json::Value =
                    serde_json::from_str(&conflicts_json).unwrap_or_default();
                let has_conflict = conflicts.as_array().map(|a| !a.is_empty()).unwrap_or(false);
                (seq, loc, present, has_conflict)
            })
            .collect();

        // ① 近 USAGE_WINDOW 章场景使用次数（空地点名跳过）
        let mut usage_map: HashMap<String, u32> = HashMap::new();
        for (seq, loc, _, _) in &rows {
            if *seq > current - USAGE_WINDOW && !loc.is_empty() {
                *usage_map.entry(loc.clone()).or_insert(0) += 1;
            }
        }
        let mut scene_usage: Vec<SceneUsage> = usage_map
            .into_iter()
            .map(|(location, count)| SceneUsage { location, count })
            .collect();
        scene_usage.sort_by(|a, b| b.count.cmp(&a.count).then(a.location.cmp(&b.location)));
        scene_usage.truncate(RENDER_CAP);

        // ② 最近一次新地点首次出现的 sequence
        let mut seen_locations: HashSet<&str> = HashSet::new();
        let mut last_new_location_seq = 0;
        for (seq, loc, _, _) in &rows {
            if !loc.is_empty() && seen_locations.insert(loc.as_str()) {
                last_new_location_seq = *seq;
            }
        }

        // ③ 最近一次角色更新（新角色登场，或沉寂 ≥3 章角色回归）的 sequence
        let mut last_character_refresh_seq = 0;
        let mut last_seen: HashMap<String, i32> = HashMap::new();
        for (seq, _, present, _) in &rows {
            for cid in present {
                match last_seen.get(cid) {
                    None => last_character_refresh_seq = *seq, // 新角色
                    Some(prev) if seq - prev >= 3 => last_character_refresh_seq = *seq, // 回归
                    _ => {}
                }
                last_seen.insert(cid.clone(), *seq);
            }
        }

        // ④ 角色沉寂账（characters 表全量，按沉寂降序）
        let mut char_stmt = conn
            .prepare("SELECT id, name FROM characters WHERE story_id = ?1")
            .map_err(|e| e.to_string())?;
        let mut character_silence: Vec<CharacterSilence> = char_stmt
            .query_map(params![story_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .map(|(cid, name)| {
                let last = last_seen.get(&cid).copied().unwrap_or(0);
                let absent = if last == 0 { current } else { current - last };
                CharacterSilence {
                    character_id: cid,
                    name,
                    chapters_absent: absent.max(0) as u32,
                }
            })
            .filter(|c| c.chapters_absent > 0)
            .collect();
        character_silence.sort_by(|a, b| b.chapters_absent.cmp(&a.chapters_absent));
        character_silence.truncate(RENDER_CAP);

        // ⑤ 尾部连续无冲突场景数
        let trailing_conflict_free = rows
            .iter()
            .rev()
            .take_while(|(_, _, _, has)| !has)
            .count() as u32;

        Ok(RotationLedger {
            current_sequence: current,
            scene_usage,
            character_silence,
            last_new_location_seq,
            last_character_refresh_seq,
            trailing_conflict_free,
        })
    }

    /// 渲染为 prompt 注入段；无任何数据时返回 None（整段省略）
    pub fn render_for_prompt(&self) -> Option<String> {
        if self.current_sequence == 0
            || (self.scene_usage.is_empty() && self.character_silence.is_empty())
        {
            return None;
        }
        let mut out = String::from("【场景与角色轮换账本（调度参考）】\n");
        if !self.scene_usage.is_empty() {
            let usage = self
                .scene_usage
                .iter()
                .map(|u| format!("{}×{}", u.location, u.count))
                .collect::<Vec<_>>()
                .join("、");
            out.push_str(&format!(
                "近 10 章场景使用：{}（优先选择使用次数少或未曾使用过的场景）\n",
                usage
            ));
        }
        if !self.character_silence.is_empty() {
            let silence = self
                .character_silence
                .iter()
                .map(|c| format!("{}（{} 章未登场）", c.name, c.chapters_absent))
                .collect::<Vec<_>>()
                .join("、");
            out.push_str(&format!(
                "角色沉寂：{}（沉寂 ≥3 章的角色建议安排回归或推进其弧光）",
                silence
            ));
        }
        Some(out.trim_end().to_string())
    }
}
```

注意：`characters`/`scenes` 表若有 NOT NULL 列导致测试 INSERT 失败，按 `connection.rs:603-631` 实际 schema 补列，不改断言语义。

- [ ] **Step 3: 测试通过**

运行 `cd src-tauri && cargo test expansion` — 累计 4 例 PASS；`cargo test --lib` 全量不回归。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/creative_engine/expansion/ledger.rs
git commit -m "新增(创作): 轮换账本聚合——场景使用账+角色沉寂账（零 LLM 成本）"
```

---

### Task 3: 扩张债务计算与弹性配额文案（debt.rs）

**Files:**
- Create: `src-tauri/src/creative_engine/expansion/debt.rs`

**Interfaces:**
- Consumes: Task 1 `ExpansionDebt`、Task 2 `RotationLedger`；`foreshadowing_tracker` 表（`setup_scene_id`/`payoff_scene_id`，V015+V027+V118）
- Produces:
  - `pub const CONFLICT_STAGNATION_THRESHOLD: u32 = 2;` / `SCENE_..=3` / `CHARACTER_..=3` / `FORESHADOW_..=3`
  - `pub enum QuotaItem { NewScene, CharacterMove, ConflictEscalation, ForeshadowMove }`
  - `impl ExpansionDebt { pub fn compute(pool: &DbPool, story_id: &str, ledger: &RotationLedger) -> Result<Self, String>; pub fn triggered(&self) -> Vec<QuotaItem>; pub fn quota_text(&self) -> Option<String>; }`

- [ ] **Step 1: 写失败测试**

debt.rs 测试模块：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn debt(c: u32, s: u32, ch: u32, f: u32) -> ExpansionDebt {
        ExpansionDebt { conflict: c, scene: s, character: ch, foreshadow: f }
    }

    #[test]
    fn thresholds_trigger_expected_items() {
        // 全部低于阈值：零干扰
        assert!(debt(1, 2, 2, 2).triggered().is_empty());
        assert!(debt(1, 2, 2, 2).quota_text().is_none());
        // 恰好达标：触发
        assert_eq!(debt(2, 0, 0, 0).triggered(), vec![QuotaItem::ConflictEscalation]);
        assert_eq!(debt(0, 3, 0, 0).triggered(), vec![QuotaItem::NewScene]);
        assert_eq!(debt(0, 0, 3, 0).triggered(), vec![QuotaItem::CharacterMove]);
        assert_eq!(debt(0, 0, 0, 3).triggered(), vec![QuotaItem::ForeshadowMove]);
        // 多项同时触发
        assert_eq!(debt(2, 3, 0, 0).triggered().len(), 2);
    }

    #[test]
    fn quota_text_escalates_with_debt_depth() {
        let mild = debt(2, 0, 0, 0).quota_text().unwrap();
        assert!(mild.contains("冲突"));
        assert!(mild.contains("必须"));
        // 深度债务（超阈值 ≥2）措辞升级
        let deep = debt(4, 0, 0, 0).quota_text().unwrap();
        assert!(deep.contains("严重停滞"));
    }

    #[test]
    fn compute_reads_foreshadow_stagnation() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let repo = crate::db::repositories::StoryRepository::new(pool.clone());
        let req = crate::db::repositories::CreateStoryRequest {
            title: "债务测试".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
            genre_profile_id: None,
            methodology_id: None,
            reference_book_id: None,
        };
        let sid = repo.create(req).unwrap().id;
        let conn = pool.get().unwrap();
        // 5 个场景，第 2 章埋过一条伏笔
        for seq in 1..=5 {
            conn.execute(
                "INSERT INTO scenes (id, story_id, sequence_number, title, setting_location, characters_present, character_conflicts, content) \
                 VALUES (?1, ?2, ?3, '章', '练功房', '[\"c1\"]', '[]', '正文')",
                rusqlite::params![format!("s{}", seq), sid, seq],
            )
            .unwrap();
        }
        conn.execute(
            "INSERT INTO foreshadowing_tracker (id, story_id, content, setup_scene_id, status, importance, created_at) \
             VALUES ('f1', ?1, '神秘令牌', 's2', 'setup', 5, datetime('now'))",
            rusqlite::params![sid],
        )
        .unwrap();
        drop(conn);

        let ledger = crate::creative_engine::expansion::ledger::RotationLedger::load_sync(&pool, &sid).unwrap();
        let d = ExpansionDebt::compute(&pool, &sid, &ledger).unwrap();
        // 伏笔停滞 = 5 - 2 = 3，达阈值
        assert_eq!(d.foreshadow, 3);
        assert!(d.triggered().contains(&QuotaItem::ForeshadowMove));
        // 场景：5 章同一地点，last_new_location_seq=1 → 债务 4
        assert_eq!(d.scene, 4);
        // 冲突：尾部 5 章全无冲突 → 债务 5
        assert_eq!(d.conflict, 5);
    }

    #[test]
    fn no_foreshadow_rows_means_zero_debt() {
        // 旧书兼容：从未有伏笔记录 → 不加压（设计：旧书初始低干扰）
        let pool = crate::db::connection::create_test_pool().unwrap();
        let repo = crate::db::repositories::StoryRepository::new(pool.clone());
        let req = crate::db::repositories::CreateStoryRequest {
            title: "无伏笔".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
            genre_profile_id: None,
            methodology_id: None,
            reference_book_id: None,
        };
        let sid = repo.create(req).unwrap().id;
        let ledger = RotationLedger::default();
        let d = ExpansionDebt::compute(&pool, &sid, &ledger).unwrap();
        assert_eq!(d.foreshadow, 0);
    }
}
```

运行 `cd src-tauri && cargo test expansion` — 预期 FAIL（编译错误）。

- [ ] **Step 2: 实现**

`src-tauri/src/creative_engine/expansion/debt.rs`：

```rust
//! 扩张债务：四项停滞指标的阈值判定与弹性配额文案。
//! 低于阈值零干扰；达标必填；超阈值 ≥2 措辞升级为"严重停滞"。

use super::{ExpansionDebt, RotationLedger};
use crate::db::connection::DbPool;
use rusqlite::params;

pub const CONFLICT_STAGNATION_THRESHOLD: u32 = 2;
pub const SCENE_STAGNATION_THRESHOLD: u32 = 3;
pub const CHARACTER_STAGNATION_THRESHOLD: u32 = 3;
pub const FORESHADOW_STAGNATION_THRESHOLD: u32 = 3;

/// 配额项（与 BeatPlan 字段的映射见 Task 5）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaItem {
    NewScene,
    CharacterMove,
    ConflictEscalation,
    ForeshadowMove,
}

impl ExpansionDebt {
    pub fn compute(
        pool: &DbPool,
        story_id: &str,
        ledger: &RotationLedger,
    ) -> Result<Self, String> {
        let current = ledger.current_sequence;
        let stagnation = |last: i32| -> u32 {
            if current == 0 || last == 0 {
                0
            } else {
                (current - last).max(0) as u32
            }
        };

        // 伏笔停滞：最近一次埋设/回收距当前的章数；表里无任何记录 → 0（旧书零干扰）
        let conn = pool.get().map_err(|e| e.to_string())?;
        let foreshadow_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM foreshadowing_tracker WHERE story_id = ?1",
                params![story_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let foreshadow = if foreshadow_count == 0 || current == 0 {
            0
        } else {
            let last: Option<i32> = conn
                .query_row(
                    "SELECT MAX(s.sequence_number) FROM scenes s WHERE s.story_id = ?1 AND s.id IN ( \
                        SELECT setup_scene_id FROM foreshadowing_tracker WHERE story_id = ?1 AND setup_scene_id IS NOT NULL \
                        UNION \
                        SELECT payoff_scene_id FROM foreshadowing_tracker WHERE story_id = ?1 AND payoff_scene_id IS NOT NULL \
                    )",
                    params![story_id],
                    |r| r.get(0),
                )
                .unwrap_or(None);
            stagnation(last.unwrap_or(0))
        };

        Ok(ExpansionDebt {
            conflict: ledger.trailing_conflict_free,
            scene: stagnation(ledger.last_new_location_seq),
            character: stagnation(ledger.last_character_refresh_seq),
            foreshadow,
        })
    }

    /// 达到阈值的配额项（平稳期返回空 = 零干扰）
    pub fn triggered(&self) -> Vec<QuotaItem> {
        let mut items = Vec::new();
        if self.conflict >= CONFLICT_STAGNATION_THRESHOLD {
            items.push(QuotaItem::ConflictEscalation);
        }
        if self.scene >= SCENE_STAGNATION_THRESHOLD {
            items.push(QuotaItem::NewScene);
        }
        if self.character >= CHARACTER_STAGNATION_THRESHOLD {
            items.push(QuotaItem::CharacterMove);
        }
        if self.foreshadow >= FORESHADOW_STAGNATION_THRESHOLD {
            items.push(QuotaItem::ForeshadowMove);
        }
        items
    }

    /// 渲染硬性扩张任务段；无触发返回 None
    pub fn quota_text(&self) -> Option<String> {
        let items = self.triggered();
        if items.is_empty() {
            return None;
        }
        let emph = |debt: u32, threshold: u32| -> &'static str {
            if debt >= threshold + 2 {
                "严重停滞"
            } else {
                "停滞"
            }
        };
        let mut lines = vec!["【本章扩张任务（硬性要求，必须落实）】".to_string()];
        for item in items {
            let line = match item {
                QuotaItem::ConflictEscalation => format!(
                    "冲突已{} {} 章——本章必须选择一条活跃冲突线将其升级（加压、反转或代价显现），不得原地踏步、不得仅靠对话过渡。",
                    emph(self.conflict, CONFLICT_STAGNATION_THRESHOLD),
                    self.conflict
                ),
                QuotaItem::NewScene => format!(
                    "场景已{} {} 章——本章必须离开当前场景，开辟一个有叙事功能的新场景（给出名称与剧情关联），不得继续在原场景打转。",
                    emph(self.scene, SCENE_STAGNATION_THRESHOLD),
                    self.scene
                ),
                QuotaItem::CharacterMove => format!(
                    "角色已{} {} 章无更新——本章必须安排一名沉寂角色回归（推进其弧光）或引入一名有叙事功能的新角色。",
                    emph(self.character, CHARACTER_STAGNATION_THRESHOLD),
                    self.character
                ),
                QuotaItem::ForeshadowMove => format!(
                    "伏笔已{} {} 章无动静——本章必须埋设一条新伏笔，或推进/回收到期伏笔。",
                    emph(self.foreshadow, FORESHADOW_STAGNATION_THRESHOLD),
                    self.foreshadow
                ),
            };
            lines.push(line);
        }
        Some(lines.join("\n"))
    }
}
```

- [ ] **Step 3: 测试通过**

运行 `cd src-tauri && cargo test expansion` — 累计 8 例 PASS；`cargo test --lib` 全量不回归。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/creative_engine/expansion/debt.rs
git commit -m "新增(创作): 扩张债务计算与弹性配额文案（阈值触发+深度加压）"
```

---

### Task 4: 动态资产菜单粗筛（asset_menu.rs）

**Files:**
- Create: `src-tauri/src/creative_engine/expansion/asset_menu.rs`

**Interfaces:**
- Consumes: Task 1 `read_asset_history`；`crate::creative_engine::beat_cards::registry::builtin_beat_cards()`（31 张，字段 `id, name, category, function, when_to_use, remix_hint, avoid, tags`）；`crate::creative_engine::story_engines::builtin_story_engines()`（21 种，字段 `id, name, payoff, best_payoff, avoid, pairs_well_with, tags`，实际路径以 `story_engines/mod.rs:55` 为准，注意是 mod.rs 直接导出还是子模块 re-export）；`crate::creative_engine::pressure_relationships::builtin_pressure_relationships()`（13 种，字段 `id, name, pressure_source, works_with, tags`）
- Produces:
  - `pub struct AssetMenuItem { pub id: String, pub kind: &'static str, pub line: String }`
  - `pub fn build_asset_menu(pool: &DbPool, story_id: &str, chapter_number: i32) -> Vec<AssetMenuItem>`（≤5 项：2 桥段卡 + 2 剧情引擎 + 1 高压关系）
  - `pub fn render_asset_menu(items: &[AssetMenuItem]) -> Option<String>`

- [ ] **Step 1: 写失败测试**

asset_menu.rs 测试模块：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn seed_story(pool: &DbPool) -> String {
        let repo = crate::db::repositories::StoryRepository::new(pool.clone());
        let req = crate::db::repositories::CreateStoryRequest {
            title: "菜单测试".to_string(),
            description: None,
            genre: None,
            style_dna_id: None,
            genre_profile_id: None,
            methodology_id: None,
            reference_book_id: None,
        };
        repo.create(req).unwrap().id
    }

    #[test]
    fn menu_has_expected_shape_and_kinds() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let sid = seed_story(&pool);
        let menu = build_asset_menu(&pool, &sid, 1);
        assert_eq!(menu.len(), 5);
        assert_eq!(menu.iter().filter(|m| m.kind == "桥段卡").count(), 2);
        assert_eq!(menu.iter().filter(|m| m.kind == "剧情引擎").count(), 2);
        assert_eq!(menu.iter().filter(|m| m.kind == "高压关系").count(), 1);
        // 每项都是一行摘要且带 id
        for item in &menu {
            assert!(!item.id.is_empty());
            assert!(item.line.contains(&item.id));
        }
    }

    #[test]
    fn recent_history_is_excluded() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let sid = seed_story(&pool);
        // 把当前第 1 章会选中的 5 个全部写进历史，下一章必须全换
        let first = build_asset_menu(&pool, &sid, 1);
        let ids: Vec<String> = first.iter().map(|m| m.id.clone()).collect();
        crate::creative_engine::expansion::append_asset_history(&pool, &sid, 1, &ids).unwrap();
        let second = build_asset_menu(&pool, &sid, 2);
        for item in &second {
            assert!(!ids.contains(&item.id), "{} 应被轮换排除", item.id);
        }
    }

    #[test]
    fn menu_is_deterministic_for_same_inputs() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let sid = seed_story(&pool);
        let a = build_asset_menu(&pool, &sid, 7);
        let b = build_asset_menu(&pool, &sid, 7);
        assert_eq!(
            a.iter().map(|m| &m.id).collect::<Vec<_>>(),
            b.iter().map(|m| &m.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn render_menu_formats_one_liners() {
        let pool = crate::db::connection::create_test_pool().unwrap();
        let sid = seed_story(&pool);
        let menu = build_asset_menu(&pool, &sid, 1);
        let text = render_asset_menu(&menu).unwrap();
        assert!(text.contains("创作资产菜单"));
        assert!(text.contains("桥段卡"));
        assert!(render_asset_menu(&[]).is_none());
    }
}
```

运行 `cd src-tauri && cargo test expansion` — 预期 FAIL（编译错误）。

- [ ] **Step 2: 实现**

`src-tauri/src/creative_engine/expansion/asset_menu.rs`：

```rust
//! 动态资产菜单：Rust 侧粗筛（轮换排除 + 确定性轮转），零 LLM 成本。
//! beat_planner 从 ≤5 个候选中精选 1-2 个，选中 ID 写入资产历史。

use super::read_asset_history;
use crate::db::connection::DbPool;
use std::collections::HashSet;

/// 近 N 条历史内用过的资产排除出候选
const RECENT_EXCLUDE_ENTRIES: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetMenuItem {
    pub id: String,
    pub kind: &'static str,
    /// 一行摘要：「[kind] name——function/payoff/pressure_source（id）」
    pub line: String,
}

/// 从候选中确定性轮转选取 n 个（排除近期用过的；排除后为空则回退全集）
fn pick_rotating<T>(
    items: Vec<T>,
    id_of: impl Fn(&T) -> &str,
    recent: &HashSet<String>,
    n: usize,
    offset: usize,
) -> Vec<T> {
    let fresh: Vec<T> = items
        .into_iter()
        .filter(|i| !recent.contains(id_of(i)))
        .collect();
    // 注意：回退全集需要原始列表，调用方保证 builtin 库远大于排除集，此处 fresh 为空时直接返回空
    // （31/21/13 的库，排除上限 5 条历史，实践中不会空）
    let len = fresh.len();
    if len == 0 {
        return Vec::new();
    }
    let start = offset % len;
    (0..len)
        .map(|i| fresh[(start + i) % len].clone())
        .take(n)
        .collect()
}

pub fn build_asset_menu(pool: &DbPool, story_id: &str, chapter_number: i32) -> Vec<AssetMenuItem> {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let history = read_asset_history(&conn, story_id);
    let recent: HashSet<String> = history
        .iter()
        .rev()
        .take(RECENT_EXCLUDE_ENTRIES)
        .flat_map(|e| e.ids.iter().cloned())
        .collect();
    let offset = chapter_number.max(0) as usize;

    let mut menu = Vec::new();

    let cards = crate::creative_engine::beat_cards::registry::builtin_beat_cards();
    for c in pick_rotating(cards, |c| c.id.as_str(), &recent, 2, offset) {
        menu.push(AssetMenuItem {
            line: format!("[桥段卡] {}——{}（{}）", c.name, c.function, c.id),
            id: c.id,
            kind: "桥段卡",
        });
    }

    let engines = crate::creative_engine::story_engines::builtin_story_engines();
    for e in pick_rotating(engines, |e| e.id.as_str(), &recent, 2, offset + 1) {
        menu.push(AssetMenuItem {
            line: format!("[剧情引擎] {}——{}（{}）", e.name, e.payoff, e.id),
            id: e.id,
            kind: "剧情引擎",
        });
    }

    let rels = crate::creative_engine::pressure_relationships::builtin_pressure_relationships();
    for r in pick_rotating(rels, |r| r.id.as_str(), &recent, 1, offset + 2) {
        menu.push(AssetMenuItem {
            line: format!("[高压关系] {}——{}（{}）", r.name, r.pressure_source, r.id),
            id: r.id,
            kind: "高压关系",
        });
    }

    menu
}

/// 渲染为 prompt 菜单段；空菜单返回 None
pub fn render_asset_menu(items: &[AssetMenuItem]) -> Option<String> {
    if items.is_empty() {
        return None;
    }
    let lines = items
        .iter()
        .enumerate()
        .map(|(i, m)| format!("{}. {}", i + 1, m.line))
        .collect::<Vec<_>>()
        .join("\n");
    Some(format!(
        "【本章可用创作资产菜单】\n{}\n（从中精选 1-2 个融入本章，将其 id 写入输出的 selected_asset_ids；如均不适用可留空数组）",
        lines
    ))
}
```

注意：
- `pick_rotating` 的泛型需要 `T: Clone`（上面代码用 `.clone()`，签名补 `T: Clone` 约束）。
- `builtin_story_engines()` 的确切导出路径以 `src-tauri/src/creative_engine/story_engines/mod.rs:55` 为准；若 `creative_engine/mod.rs` 对子模块的 re-export 方式不同（如 `story_engines::mod`），按实际调整 use 路径。
- `BeatCard.function`/`StoryEngine.payoff`/`PressureRelationship.pressure_source` 字段名以事实清单（设计文档第 5 节引用）为准。

- [ ] **Step 3: 测试通过**

运行 `cd src-tauri && cargo test expansion` — 累计 12 例 PASS；`cargo test --lib` 全量不回归。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/creative_engine/expansion/asset_menu.rs
git commit -m "新增(创作): 动态资产菜单粗筛——轮换排除+确定性轮转（31 桥段卡/21 剧情引擎/13 高压关系）"
```

---

### Task 5: beat_planner 接入（BeatPlan 扩展 + 模板 + executor 注入）

**Files:**
- Modify: `src-tauri/src/planner/executor.rs`（`BeatPlan` 结构体 :48-68、`execute_beat_planner` :1538 起、`render_beat_plan_prompt` :1681-1699、`degraded_beat_output` :1712-1723）
- Modify: `resources/prompts/writer/writer_beat_plan.md`（front-matter version → 0.34.0，加三个条件块与新 JSON 字段）

**Interfaces:**
- Consumes: Task 2 `RotationLedger::load_sync/render_for_prompt`、Task 3 `ExpansionDebt::compute/quota_text`、Task 4 `build_asset_menu/render_asset_menu`、Task 1 `append_asset_history`
- Produces:
  - `BeatPlan` 新字段 `character_moves: String`（#[serde(default)]）、`selected_asset_ids: Vec<String>`（#[serde(default)]）
  - 模板新变量 `expansion_quota` / `rotation_ledger` / `asset_menu`（均为可选文本）
  - 降级输出 content 从空串改为 Rust 侧默认配额文案（writer 依赖检查仍通过）

- [ ] **Step 1: 写失败测试**

`src-tauri/src/planner/executor.rs` 测试模块追加（若无测试模块则新建，参考文件内既有测试风格）：

```rust
#[test]
fn beat_plan_parses_new_expansion_fields() {
    let json = r#"{
        "goal": "夺回令牌",
        "conflict_escalation": "师父当众翻脸",
        "new_elements": "新场景：断剑崖",
        "character_moves": "林雪回归，带来令牌线索",
        "foreshadowing_ops": "埋设：令牌背面的铭文",
        "target_words": 1500,
        "selected_asset_ids": ["beat_card.downfall_relearn_return"]
    }"#;
    let plan: BeatPlan = serde_json::from_str(json).unwrap();
    assert_eq!(plan.character_moves, "林雪回归，带来令牌线索");
    assert_eq!(plan.selected_asset_ids, vec!["beat_card.downfall_relearn_return"]);
    let text = plan.to_prompt_text();
    assert!(text.contains("林雪回归"));
}

#[test]
fn beat_plan_defaults_when_new_fields_absent() {
    // 旧格式输出（无新字段）仍可解析——向后兼容
    let json = r#"{"goal": "g", "conflict_escalation": "c", "new_elements": "n", "foreshadowing_ops": "f", "target_words": 1200}"#;
    let plan: BeatPlan = serde_json::from_str(json).unwrap();
    assert!(plan.character_moves.is_empty());
    assert!(plan.selected_asset_ids.is_empty());
}
```

运行 `cd src-tauri && cargo test planner` — 预期 FAIL（编译错误：字段不存在）。

- [ ] **Step 2: 实现 BeatPlan 扩展**

`src-tauri/src/planner/executor.rs:48-68` 的 `BeatPlan` 加两个字段（保持既有 `#[serde(default)]` 风格）：

```rust
    /// 角色调度：沉寂角色回归/新角色登场及各自行动目的（v0.34.0 弹性扩张）
    #[serde(default)]
    pub character_moves: String,
    /// 本章选用的创作资产 ID（从资产菜单精选，v0.34.0 弹性扩张）
    #[serde(default)]
    pub selected_asset_ids: Vec<String>,
```

`to_prompt_text()`（同文件，找到现有实现）在末尾追加（仅非空时）：

```rust
        if !self.character_moves.is_empty() {
            text.push_str(&format!("\n角色调度：{}", self.character_moves));
        }
        if !self.selected_asset_ids.is_empty() {
            text.push_str(&format!("\n本章选用创作资产：{}", self.selected_asset_ids.join("、")));
        }
```

（若现有 `to_prompt_text` 不是用 `text.push_str` 风格，按其既有构造方式等价追加。）

- [ ] **Step 3: 模板更新**

`resources/prompts/writer/writer_beat_plan.md`：front-matter `version: 0.34.0`，variables 列表加 `expansion_quota` / `rotation_ledger` / `asset_menu`。正文在【创作策略四元组】块之后、【创作指令】之前插入三个条件块，并更新输出 JSON 与要求：

```
{{#if expansion_quota}}
{{expansion_quota}}

{{/if}}
{{#if rotation_ledger}}
{{rotation_ledger}}

{{/if}}
{{#if asset_menu}}
{{asset_menu}}

{{/if}}
```

输出 JSON 改为：

```
请用 JSON 输出本节拍规划（总字数不超过300字）：
{
  "goal": "本节拍的戏剧目标（一句话）",
  "conflict_escalation": "冲突如何升级（一句话）",
  "new_elements": "引入的新元素：有叙事功能的新角色/新场景/新道具（一句话，可为无）",
  "character_moves": "角色调度：哪些角色登场/回归/退场，各自行动目的（一句话，可为无）",
  "foreshadowing_ops": "伏笔操作：埋设/推进/兑现哪条伏笔（一句话，可为无）",
  "target_words": 1500,
  "selected_asset_ids": ["从上方资产菜单精选的资产 id，0-2 个"]
}

要求：
1. 新元素必须有叙事功能，不与世界观冲突
2. 若上方给出【本章扩张任务】，其要求必须在对应字段中落实，相关字段不得为"无"或留空
3. 只输出 JSON，不要其他内容
```

（保留模板其余既有内容不变，仅做上述插入与替换。）

- [ ] **Step 4: executor 注入与降级改造**

`execute_beat_planner`（executor.rs:1538 起）：

a) 在构建 `story_context` 之后、渲染模板之前，插入（story_id 与 chapter_number 以函数内既有上下文变量为准——plan_context 有 `current_story_id`；章号用现有章节数 +1 或上下文既有字段，按代码实际取）：

```rust
// v0.34.0 弹性扩张：轮换账本 + 扩张债务配额 + 资产菜单（纯 Rust，零额外 LLM）
let ledger = crate::creative_engine::expansion::ledger::RotationLedger::load_sync(
    &self.pool,
    &story_id,
)
.unwrap_or_default();
let debt = crate::creative_engine::expansion::debt::ExpansionDebt::compute(
    &self.pool,
    &story_id,
    &ledger,
)
.unwrap_or_default();
let quota_text = debt.quota_text();
let ledger_text = ledger.render_for_prompt();
let menu = crate::creative_engine::expansion::asset_menu::build_asset_menu(
    &self.pool,
    &story_id,
    chapter_number,
);
let menu_text = crate::creative_engine::expansion::asset_menu::render_asset_menu(&menu);
```

b) `render_beat_plan_prompt`（:1681-1699）变量映射加三项（Option 文本按该函数既有 Option 变量处理方式渲染，无则 `{{#if}}` 块不展开）：

```rust
"expansion_quota" => quota_text,
"rotation_ledger" => ledger_text,
"asset_menu" => menu_text,
```

c) 解析成功后（:1704-1708 的 `Ok(plan)` 分支）写资产历史：

```rust
if !plan.selected_asset_ids.is_empty() {
    let _ = crate::creative_engine::expansion::append_asset_history(
        &self.pool,
        &story_id,
        chapter_number,
        &plan.selected_asset_ids,
    );
}
```

d) `degraded_beat_output` 签名改为带默认内容，所有调用点传入 Rust 侧兜底文案（quota_text 克隆；无配额时给空串——保持 writer 跳过行为）：

```rust
fn degraded_beat_output(reason: &str, default_content: String) -> serde_json::Value {
    serde_json::json!({
        "content": default_content,
        "beat_plan": null,
        "degraded": true,
        "reason": reason,
    })
}
```

调用点（模板缺失/LLM 失败/超时/解析失败共 4 处）统一传 `quota_text.clone().unwrap_or_default()`。注意配额文案须在调用点之前计算（步骤 a 已前置）。

- [ ] **Step 5: 测试通过**

运行 `cd src-tauri && cargo test planner` 与 `cargo test beat` — 新 2 例 PASS，既有用例不回归；`cargo test --lib` 全量不回归。`cargo +nightly fmt`。

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/planner/executor.rs resources/prompts/writer/writer_beat_plan.md
git commit -m "新增(创作): beat_planner 接入弹性扩张——配额/账本/资产菜单注入 + BeatPlan 扩展 + 降级兜底文案"
```

---

### Task 6: WriteTimeBundle ⑧e 账本段 + writer 模板承接

**Files:**
- Modify: `src-tauri/src/domain/write_time_bundle.rs`（结构体加字段，:12-77）
- Modify: `src-tauri/src/creative_engine/write_time_bundle.rs`（`load_sync` 渲染、⑩段附近的 `to_prompt()` 加 ⑧e）
- Modify: `resources/prompts/writer/orchestrator_timesliced_writer.md`（version → 0.34.0，加硬性落实要求）

**Interfaces:**
- Consumes: Task 2 `RotationLedger::load_sync/render_for_prompt`
- Produces: `WriteTimeBundle.rotation_ledger_text: Option<String>`；`to_prompt()` 新段 ⑧e

- [ ] **Step 1: 写失败测试**

`src-tauri/src/creative_engine/write_time_bundle.rs` 测试模块追加（参考文件内既有测试）：

```rust
#[test]
fn to_prompt_includes_rotation_ledger_when_present() {
    let mut bundle = WriteTimeBundle::empty_for_test(); // 若无此辅助，用既有测试的 bundle 构造方式
    bundle.rotation_ledger_text = Some(
        "【场景与角色轮换账本（调度参考）】\n近 10 章场景使用：练功房×4".to_string(),
    );
    let prompt = bundle.to_prompt();
    assert!(prompt.contains("轮换账本"));
    assert!(prompt.contains("练功房×4"));
}

#[test]
fn to_prompt_omits_rotation_ledger_when_absent() {
    let bundle = WriteTimeBundle::empty_for_test();
    assert!(!bundle.to_prompt().contains("轮换账本"));
}
```

（`WriteTimeBundle` 的测试构造辅助以该文件既有测试为准；若无 `empty_for_test`，用既有测试里构造 bundle 的方式，或直接在测试里 `..Default::default()`——若结构体无 Default 实现，本任务可为其 `#[derive(Default)]` 或手写，属最小合理调整，在报告中注明。）

运行 `cd src-tauri && cargo test write_time_bundle` — 预期 FAIL（字段不存在）。

- [ ] **Step 2: 实现**

a) `src-tauri/src/domain/write_time_bundle.rs` 结构体加字段（放 `style_blend_text` 之后）：

```rust
    /// v0.34.0: 场景与角色轮换账本（弹性扩张，expansion::ledger 渲染）
    pub rotation_ledger_text: Option<String>,
```

b) `src-tauri/src/creative_engine/write_time_bundle.rs` 的 `load_sync`：在函数末尾构造返回值的结构体初始化处加（story_id 即参数）：

```rust
    // v0.34.0 弹性扩张：轮换账本段（数据缺失/空书 → None，整段省略）
    let rotation_ledger_text =
        crate::creative_engine::expansion::ledger::RotationLedger::load_sync(pool, story_id)
            .ok()
            .and_then(|l| l.render_for_prompt());
```

（失败用 `.ok()` 吞掉回退 None——账本是非关键增强段，不得阻断创作主流程。）

c) `to_prompt()` 在 ⑧d 追读力债务段之后加（:738-746 附近，沿用既有编号注释风格）：

```rust
// ⑧e 轮换账本（v0.34.0 弹性扩张：场景/角色使用数据驱动调度）
if let Some(ref ledger) = self.rotation_ledger_text {
    sections.push(ledger.clone());
}
```

d) `resources/prompts/writer/orchestrator_timesliced_writer.md`：front-matter `version: 0.34.0`，在要求 7 之后加：

```
8. 若故事上下文中出现【本章扩张任务】或【场景与角色轮换账本】，或正文前出现节拍规划：其中列出的扩张任务（新场景/角色调度/冲突升级/伏笔操作）是硬性要求，必须在本段正文中落实；账本中的低频场景与沉寂角色是调度的优先候选。
```

- [ ] **Step 3: 测试通过**

运行 `cd src-tauri && cargo test write_time_bundle` — 新 2 例 PASS；`cargo test --lib` 全量不回归。`cargo +nightly fmt`。

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/domain/write_time_bundle.rs src-tauri/src/creative_engine/write_time_bundle.rs resources/prompts/writer/orchestrator_timesliced_writer.md
git commit -m "新增(创作): WriteTimeBundle ⑧e 轮换账本段 + writer 模板硬性落实要求"
```

---

### Task 7: 全量验证 + CHANGELOG + 设计文档回写

**Files:**
- Modify: `CHANGELOG.md`（既有 v0.34.0 条目内增补）
- Modify: `docs/plans/2026-08-09-elastic-expansion-asset-fusion-design.md`（状态行 + 偏差回写）

**Interfaces:**
- Consumes: Task 1-6 全部

- [ ] **Step 1: 全量验证**

```bash
cd src-tauri && cargo test --lib    # 预期 1217 + 新增约 15 例全过，2 ignored
cargo +nightly fmt --check          # 干净
```

若 fmt 有 diff 先 `cargo +nightly fmt` 再复跑测试。

- [ ] **Step 2: CHANGELOG 增补**

在既有 v0.34.0 条目内追加一节（遵循该文件既有中文格式）：

```markdown
- 新增：弹性扩张——续写侧创作资产强关联
  - 轮换账本：近 10 章场景使用账 + 角色沉寂账注入写作上下文（⑧e 段，零 LLM 成本）
  - 扩张债务弹性配额：冲突停滞 2 章/场景固化 3 章/角色固化 3 章/伏笔停滞 3 章触发硬性扩张任务，债务越深措辞越强；平稳期零干扰
  - 动态资产菜单：31 张桥段卡/21 种剧情引擎/13 种高压关系按轮换排除粗筛 5 个候选，beat_planner 精选 1-2 个写入 beat 计划；选用历史存 stories.asset_history_json（V122）
  - beat_planner 降级（超时/失败）时由 Rust 侧注入默认配额文案兜底
```

「测试」节数字更新为实测值。

- [ ] **Step 3: 设计文档回写**

`docs/plans/2026-08-09-elastic-expansion-asset-fusion-design.md` 状态行改为「已实施（并入 v0.34.0）」，并在文末追加「实施偏差记录」：

```markdown
## 实施偏差记录（2026-08-09）

1. 资产选用历史存 `stories.asset_history_json`（V122）而非设计写的"章节元数据"——beat 规划时章节行尚不存在，且 stories 级历史天然支持跨章轮换排除
2. `BeatPlan` 复用既有 `new_elements/conflict_escalation/foreshadowing_ops` 字段（原本"可为无"），弹性配额改为条件强制；新增 `character_moves` 与 `selected_asset_ids` 两字段
3. 资产粗筛未按叙事阶段过滤（常量库无阶段标签字段），采用确定性轮转 + 轮换排除保证多样性
```

- [ ] **Step 4: Commit**

```bash
git add CHANGELOG.md docs/plans/2026-08-09-elastic-expansion-asset-fusion-design.md
git commit -m "文档: v0.34.0 增补弹性扩张（续写侧资产强关联）+ 设计文档实施偏差回写"
```

---

## Self-Review 记录

- Spec coverage：设计四节 → Task 2（轮换账本）、Task 3（弹性配额）、Task 4（资产选择粗筛）+ Task 5（beat 精选与注入）、Task 6（writer 承接）、Task 1（存储）+ Task 7（降级验证归 Task 5d/文档归 Task 7）。第二期创世贯通与远期质量反馈环不在本计划（设计已注明）。
- 类型一致性：`RotationLedger/ExpansionDebt/QuotaItem/AssetMenuItem/AssetHistoryEntry`、`load_sync/render_for_prompt/compute/quota_text/triggered/build_asset_menu/render_asset_menu/read_asset_history/append_asset_history` 跨任务引用一致；`BeatPlan.character_moves/selected_asset_ids` 在 Task 5 定义并消费。
- 已知实现期微调点（各任务步骤内已注明）：StoryRepository 字段、表 NOT NULL 列、builtin 函数导出路径、bundle 测试构造辅助、degraded_beat_output 调用点数量。
