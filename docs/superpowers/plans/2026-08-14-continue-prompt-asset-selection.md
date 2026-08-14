# 续写提示词按拍选取资产 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Agency 续写热路径只把本拍录取角色的完整卡注入主创 prompt；其余本故事相关角色一行名单；大纲去重；前文不再叠三场。

**Architecture:** 新增 `agency/continue_assets.rs` 纯函数编译器（0 LLM、0 I/O）。`WriteTimeBundle::to_prompt()` 全局语义不动。`write_beat_once` / `write_chapter` 先编译 BeatCard，再按准入名单渲染资产。`build_writer_context_from_db` 改为 load + `render_continue_assets`（无卡时录取角色表前 8 人）。

**Tech Stack:** Rust / rusqlite 测试池。设计：`docs/plans/2026-08-14-continue-prompt-asset-selection-design.md`。

**验证基线：** `cd src-tauri && cargo test --lib`（当前 1354 passed / 2 ignored）。前端无逻辑变更，不强制重跑 vitest。

**提交纪律：** 用户未说「提交」前不 `git commit`。本计划勾掉各 Task 的 Commit 步；代码写完后等用户发「提交」。

**禁止：** 改 `to_prompt()`；接 `ContextPrioritizer`；热路径加 LLM 选 key；清 `characters` 脏行；改 ingest 大纲追加；宣称 §13 八次真机探针已过。

---

## File map

| 文件 | 职责 |
|---|---|
| Create `src-tauri/src/agency/continue_assets.rs` | 准入合并、名单、大纲去重、世界观截取、关系过滤、预算、`render_continue_assets` |
| Modify `src-tauri/src/agency/mod.rs` | `pub mod continue_assets;` |
| Modify `src-tauri/src/agency/coordinator.rs` | `load_continue_context_parts`；`build_writer_context_from_db` 走筛选渲染；`write_beat_once` 重排；`generate_chapter_outline` 不再全表角色；`write_chapter` 共用筛选后的 assets |
| Modify `src-tauri/src/agency/tests.rs` | `test_build_continue_writer_context` 仍应通过（单角色=录取）；不改成「表里有谁 prompt 就有谁」 |
| Test `src-tauri/src/agency/continue_assets.rs` `#[cfg(test)]` | 规格 §7 契约 1–5 |
| Test `src-tauri/src/agency/coordinator.rs` 或 `tests.rs` | 规格 §7 契约 6：组装后的 user prompt 不含未录取者「情感内核」 |

漏网：`agency/tools.rs` 的 `bundle.to_prompt()` 是 tool 内全量，**本期不改**（writer 热路径已预注入，不再靠 tool 倾倒）。创世 `build_writer_assets_context` 不动。

GitNexus：改 `write_beat_once` / `generate_chapter_outline` / `build_writer_context_from_db` 前对每个符号跑 `impact({target, direction:"upstream"})`。`to_prompt` **不改**。

---

### Task 1: 大纲去重 + 准入合并纯函数

**Files:**
- Create: `src-tauri/src/agency/continue_assets.rs`
- Modify: `src-tauri/src/agency/mod.rs`

- [ ] **Step 1: 写失败测试**

`continue_assets.rs` 先只放测试（函数尚未存在，应编译失败）：

```rust
//! 续写资产按拍筛选。设计：docs/plans/2026-08-14-continue-prompt-asset-selection-design.md
//! 0 LLM、0 I/O。不改 WriteTimeBundle::to_prompt()。

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condense_outline_dedupes_repeated_core_conflict() {
        let stacked = "【核心冲突】皇权裂痕\n【转折点】密诏\n".repeat(10);
        let out = condense_story_outline(&stacked, "推进到密诏败露");
        let count = out.matches("【核心冲突】皇权裂痕").count();
        assert_eq!(count, 1, "got: {out}");
        assert!(out.chars().count() <= 1200);
        assert!(out.contains("密诏败露") || out.contains("密诏"));
    }

    #[test]
    fn condense_outline_empty_falls_back_to_next_node() {
        let out = condense_story_outline("   ", "下一节点：入宫");
        assert!(out.contains("入宫"), "got: {out}");
        assert!(!out.contains("【核心冲突】【核心冲突】"));
    }

    #[test]
    fn merge_admitted_priority_and_cap_eight() {
        let present = vec!["甲".into(), "乙".into(), "丙".into()];
        let parties = vec!["丁".into()];
        let mentioned = vec!["戊".into(), "己".into()];
        let rest = vec!["庚".into(), "辛".into(), "壬".into(), "癸".into()];
        let got = merge_admitted(&present, &parties, &mentioned, &rest);
        assert_eq!(got, vec!["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛"]);
        assert_eq!(got.len(), 8);
        assert!(!got.iter().any(|n| n == "壬" || n == "癸"));
    }

    #[test]
    fn names_in_text_intersects_character_table() {
        let names = ["苏亦铁", "周奕辰", "何双"];
        let text = "苏亦铁推开殿门，何双跟在后面。";
        let got = names_in_text(&names, text);
        assert_eq!(got, vec!["苏亦铁", "何双"]);
        assert!(!got.iter().any(|n| n == "周奕辰"));
    }
}
```

`mod.rs` 在 `pub mod beat_card;` 后加 `pub mod continue_assets;`。

- [ ] **Step 2: 跑测试确认失败**

```
cd src-tauri && cargo test --lib agency::continue_assets -- --nocapture
```

Expected: compile error `cannot find function condense_story_outline`（或同类）。

- [ ] **Step 3: 最小实现**

```rust
pub const ADMITTED_CAP: usize = 8;
pub const OUTLINE_CHAR_CAP: usize = 1200;

pub fn names_in_text(character_names: &[impl AsRef<str>], text: &str) -> Vec<String> {
    character_names
        .iter()
        .map(|n| n.as_ref())
        .filter(|n| !n.is_empty() && text.contains(*n))
        .map(|n| n.to_string())
        .collect()
}

pub fn merge_admitted(
    present: &[String],
    parties: &[String],
    mentioned: &[String],
    rest: &[String],
) -> Vec<String> {
    let mut out = Vec::new();
    for src in [present, parties, mentioned, rest] {
        for n in src {
            if n.is_empty() {
                continue;
            }
            if !out.iter().any(|e| e == n) {
                out.push(n.clone());
            }
            if out.len() >= ADMITTED_CAP {
                return out;
            }
        }
    }
    out
}

pub fn condense_story_outline(raw: &str, next_node: &str) -> String {
    let raw = raw.trim();
    let mut blocks: Vec<String> = Vec::new();
    if !raw.is_empty() {
        // 按【…】标题切段；连续重复标题块只留首次。
        let mut current = String::new();
        for line in raw.lines() {
            let t = line.trim();
            if t.starts_with('【') && t.contains('】') && !current.is_empty() {
                push_unique_block(&mut blocks, current.trim().to_string());
                current.clear();
            }
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
        if !current.trim().is_empty() {
            push_unique_block(&mut blocks, current.trim().to_string());
        }
        if blocks.is_empty() {
            push_unique_block(&mut blocks, raw.to_string());
        }
    }

    let mut core: Option<String> = None;
    let mut turning: Vec<String> = Vec::new();
    let mut other: Vec<String> = Vec::new();
    for b in blocks {
        if b.contains("【核心冲突】") && core.is_none() {
            core = Some(b);
        } else if b.contains("【转折点】") || b.contains("【关键转折点】") {
            if turning.len() < 3 {
                turning.push(b);
            }
        } else if other.len() < 2 {
            other.push(b);
        }
    }

    if turning.len() > 1 && !next_node.trim().is_empty() {
        let overlapped: Vec<String> = turning
            .iter()
            .filter(|t| overlaps(t, next_node))
            .cloned()
            .collect();
        if !overlapped.is_empty() {
            turning = overlapped.into_iter().take(3).collect();
        }
    }

    let mut parts: Vec<String> = Vec::new();
    if let Some(c) = core {
        parts.push(c);
    }
    parts.extend(turning);
    parts.extend(other);
    let next = next_node.trim();
    if !next.is_empty() && !parts.iter().any(|p| p.contains(next)) {
        parts.push(format!("【下一节点】{next}"));
    }
    if parts.is_empty() && !next.is_empty() {
        parts.push(next.to_string());
    }
    truncate_chars(&parts.join("\n"), OUTLINE_CHAR_CAP)
}

fn push_unique_block(blocks: &mut Vec<String>, block: String) {
    if !blocks.iter().any(|b| b == &block) {
        blocks.push(block);
    }
}

fn overlaps(a: &str, b: &str) -> bool {
    let needle: String = b.chars().filter(|c| !c.is_whitespace()).take(8).collect();
    !needle.is_empty() && a.contains(&needle)
        || b.chars().count() >= 2 && a.contains(b.trim())
}

pub fn truncate_chars(s: &str, max: usize) -> String {
    let n = s.chars().count();
    if n <= max {
        s.to_string()
    } else {
        format!("{}…（已截断）", s.chars().take(max).collect::<String>())
    }
}
```

- [ ] **Step 4: 跑测试确认通过**

```
cd src-tauri && cargo test --lib agency::continue_assets -- --nocapture
```

Expected: 4 passed。

---

### Task 2: 名单（roster）+ 脏名过滤

**Files:**
- Modify: `src-tauri/src/agency/continue_assets.rs`

- [ ] **Step 1: 写失败测试**

```rust
#[test]
fn roster_excludes_admitted_and_orphans() {
    let table = ["苏亦铁", "何双", "周奕辰", "赵奎"];
    let admitted = ["苏亦铁"];
    let evidence = "何双在茶馆坐下。赵奎守门。"; // 无周奕辰
    let got = build_roster(&table, &admitted, evidence);
    assert!(!got.iter().any(|n| n == "苏亦铁"));
    assert!(got.contains(&"何双".to_string()));
    assert!(got.contains(&"赵奎".to_string()));
    assert!(!got.iter().any(|n| n == "周奕辰"), "脏名不得进名单: {got:?}");
}

#[test]
fn roster_line_format_and_cap() {
    let names: Vec<String> = (0..45).map(|i| format!("角{i:02}")).collect();
    let line = render_roster_line(&names);
    assert!(line.starts_with("本拍未上场（禁止新编下列姓名，亦不得当主角使用）"));
    assert!(line.ends_with("等"));
    assert!(!line.contains("情感内核"));
}
```

- [ ] **Step 2: 跑测试确认失败**

```
cd src-tauri && cargo test --lib agency::continue_assets::tests::roster -- --nocapture
```

Expected: `cannot find function build_roster`。

- [ ] **Step 3: 最小实现**

```rust
pub const ROSTER_NAME_CAP: usize = 40;
pub const ROSTER_PREFIX: &str = "本拍未上场（禁止新编下列姓名，亦不得当主角使用）";

pub fn build_roster(
    table_names: &[impl AsRef<str>],
    admitted: &[impl AsRef<str>],
    evidence: &str,
) -> Vec<String> {
    table_names
        .iter()
        .map(|n| n.as_ref())
        .filter(|n| !n.is_empty())
        .filter(|n| !admitted.iter().any(|a| a.as_ref() == *n))
        .filter(|n| evidence.contains(*n))
        .map(|n| n.to_string())
        .collect()
}

pub fn render_roster_line(roster: &[String]) -> String {
    if roster.is_empty() {
        return String::new();
    }
    let overflow = roster.len() > ROSTER_NAME_CAP;
    let shown = &roster[..roster.len().min(ROSTER_NAME_CAP)];
    let mut line = format!("{ROSTER_PREFIX}{}", shown.join("、"));
    if overflow {
        line.push_str("等");
    }
    line
}
```

`evidence` 由调用方拼接：近 5 场 `content` + 各场 `characters_present` 用顿号连起来的名字 + `story_outlines.content`。`characters_present` 里对不上角色表的 token（id）自然不会 `contains` 命中人名。

- [ ] **Step 4: 跑测试确认通过**

```
cd src-tauri && cargo test --lib agency::continue_assets -- --nocapture
```

Expected: 全绿。

---

### Task 3: 关系过滤 + 角色卡渲染 + 世界观截取 + 预算

**Files:**
- Modify: `src-tauri/src/agency/continue_assets.rs`

- [ ] **Step 1: 写失败测试**

```rust
#[test]
fn relationships_require_both_ends_admitted() {
    let lines = vec![
        "■ 甲 -> 乙：社会关系=同僚 ｜ 情感=恨[0.9]".into(),
        "■ 甲 -> 丙：社会关系=路人 ｜ 情感=无[0.1]".into(),
    ];
    let admitted = ["甲", "乙"];
    let got = filter_relationship_lines(&lines, &admitted);
    assert_eq!(got.len(), 1);
    assert!(got[0].contains("甲 -> 乙"));
    assert!(!got.iter().any(|l| l.contains("丙")));
}

#[test]
fn render_cards_only_admitted_and_renames_heading() {
    let chars = vec![core("甲"), core("乙"), core("丙")];
    let admitted = ["甲", "乙"];
    let text = render_admitted_cards(&chars, &admitted);
    assert!(text.contains("【本拍角色（须遵循当前状态）】"));
    assert!(!text.contains("【登场角色（必须严格遵循其当前状态）】"));
    assert!(text.contains("情感内核：甲的情感内核"));
    assert!(text.contains("情感内核：乙的情感内核"));
    assert!(!text.contains("情感内核：丙的情感内核"));
}

#[test]
fn condense_world_drops_history_and_cultures() {
    let raw = "世界概念：阴阳两界裂开了一道缝，缝里是旧朝的祭法。\n\n核心规则：\n- 夜行：入夜后阳气不可过界\n- 血契：以血换名\n\n历史背景：三千年前的巫祭。\n\n文化与势力：\n- 阴司：收魂";
    let out = condense_world_setting(raw, Some("茶馆"), &["苏亦铁".into()]);
    assert!(out.contains("世界概念"));
    assert!(out.contains("夜行") || out.contains("核心规则"));
    assert!(!out.contains("三千年前"));
    assert!(!out.contains("阴司"));
    assert!(out.chars().count() <= 800);
}

#[test]
fn budget_keeps_redline_cards_roster_truncates_prior() {
    let redline = "【⚠️ 世界观红线（绝不可违背，违反即判定为严重错误）】\n禁止时间旅行";
    let cards = "【本拍角色（须遵循当前状态）】\n- 姓名：甲 | 情感内核：核";
    let roster = format!("{ROSTER_PREFIX}乙、丙");
    let prior = format!("【前文】{}", "哈".repeat(8000));
    let out = apply_asset_budget(&[
        redline.to_string(),
        cards.to_string(),
        roster.clone(),
        prior,
    ]);
    assert!(out.chars().count() <= ASSET_CHAR_BUDGET);
    assert!(out.contains("禁止时间旅行"));
    assert!(out.contains("情感内核：核"));
    assert!(out.contains(&roster));
}

fn core(name: &str) -> crate::domain::write_time_bundle::CoreCharacter {
    crate::domain::write_time_bundle::CoreCharacter {
        name: name.into(),
        identity: Some("身份".into()),
        physical_state: None,
        mental_state: None,
        location: None,
        personality: Some("性格".into()),
        emotional_core: Some(format!("{name}的情感内核")),
        emotional_trigger: None,
        emotional_wound: None,
        emotional_need: None,
    }
}
```

- [ ] **Step 2: 跑测试确认失败**

Expected: 缺 `filter_relationship_lines` / `render_admitted_cards` / `condense_world_setting` / `apply_asset_budget`。

- [ ] **Step 3: 最小实现**

关系行格式锁定为 Bundle 现网：`■ {src} -> {tgt}：...`。两端都在 `admitted` 才保留。解析：找 ` -> `，左侧去 `■ `，右侧取到 `：` 或 `:`。

角色卡复用 `to_prompt` 单卡字段顺序（姓名/身份/当前状态/性格/四元组），标题改为「本拍角色（须遵循当前状态）」。只输出 `admitted` 顺序中能在 `core_characters` 里找到的人。

世界观：按 `\n\n` 或行首 `世界概念：` / `核心规则：` / `历史背景：` / `文化与势力：` 分段。丢弃历史/文化。概念 `truncate_chars(..., 400)`。规则最多 5 条；若 `location` 或录取名出现在某条 `- 名：描述` 中则优先保留。整段 cap 800。

预算：`ASSET_CHAR_BUDGET = 6000`。`apply_asset_budget(sections: &[String])`：sections 约定顺序为红线、大纲、世界观、角色卡、名单、关系、本章大纲、进度、前文、张力、弧光、logline、伏笔。超预算时从「前文 → 进度 → 世界观 → 大纲转折（即大纲段）」依次 `truncate_chars` 或整段清空前文，直到 ≤6000。红线、角色卡、名单不得删光（只截断到至少保留标题+一行）。

`slice_prior_prose(text: &str) -> String`：取**末** 800 字（`chars().rev().take(800)` 再反转）。空则空串。

- [ ] **Step 4: 跑测试确认通过**

```
cd src-tauri && cargo test --lib agency::continue_assets -- --nocapture
```

---

### Task 4: `render_continue_assets` 总装 + 规格 §7.1 端到端纯函数

**Files:**
- Modify: `src-tauri/src/agency/continue_assets.rs`

- [ ] **Step 1: 写失败测试（20 人点 3 个）**

```rust
use crate::domain::write_time_bundle::{CoreCharacter, WriteTimeBundle, StoryMeta, GenreCategory};

fn empty_bundle() -> WriteTimeBundle {
    WriteTimeBundle {
        contract_redlines: Some(r#"{"world_rules":"禁止时间旅行"}"#.into()),
        core_characters: vec![],
        relationship_lines: vec![],
        scene_outline: None,
        story_outline: Some("【核心冲突】皇权裂痕\n【核心冲突】皇权裂痕".into()),
        world_setting: Some("世界概念：阴阳两界。".into()),
        genre_antipatterns: vec![],
        style_slice: None,
        story_meta: StoryMeta {
            title: "镇魂".into(),
            genre: None,
            tone: None,
            pacing: None,
            description: None,
        },
        genre_category: GenreCategory::Speculative,
        narrative_phase_guidance: None,
        pending_foreshadowings: vec!["旧剑将出鞘".into()],
        overdue_foreshadowings: vec![],
        style_dna_summary: None,
        narrative_quartet: None,
        style_dna_extension: None,
        methodology_extension: None,
        genre_profile_strategy: Some("不得进本拍".into()),
        secondary_genre_profile_strategy: None,
        writing_strategy_constraints: None,
        runtime_contract: None,
        reference_scene_fewshots: vec![],
        related_entity_summaries: vec![],
        active_conflicts: None,
        character_goals: None,
        chase_debt_text: None,
        genre_reference: Some("题材表不得进".into()),
        style_blend_text: None,
        rotation_ledger_text: None,
    }
}

#[test]
fn twenty_chars_three_present_full_cards_rest_roster() {
    let mut bundle = empty_bundle();
    let names: Vec<String> = (0..20).map(|i| format!("角色{:02}", i)).collect();
    bundle.core_characters = names
        .iter()
        .map(|n| core(n))
        .collect();
    bundle.relationship_lines = vec![
        "■ 角色00 -> 角色01：社会关系=同僚 ｜ 情感=恨[0.9]".into(),
        "■ 角色00 -> 角色19：社会关系=路人 ｜ 情感=无[0.1]".into(),
    ];
    let present = vec!["角色00".into(), "角色01".into(), "角色02".into()];
    let evidence: String = names.join("、"); // 全部相关，无脏名
    let input = ContinueAssetsInput {
        bundle: &bundle,
        admitted: &present,
        roster: &build_roster(&names, &present, &evidence),
        location: None,
        next_node: "下一节点",
        chapter_outline: "本章：角色00对质",
        progress_lines: &[],
        prior_prose: "前文末段。",
        tension_lines: &[],
        arc_lines: &[],
        logline: Some("一句话"),
    };
    let out = render_continue_assets(&input);
    assert!(out.contains("情感内核：角色00的情感内核"));
    assert!(out.contains("情感内核：角色01的情感内核"));
    assert!(out.contains("情感内核：角色02的情感内核"));
    assert!(!out.contains("情感内核：角色19的情感内核"));
    assert!(!out.contains("必须严格遵循其当前状态"));
    assert!(out.contains(ROSTER_PREFIX));
    assert!(out.contains("角色19"));
    assert!(out.contains("角色00 -> 角色01"));
    assert!(!out.contains("角色00 -> 角色19"));
    assert_eq!(out.matches("【核心冲突】皇权裂痕").count(), 1);
    assert!(!out.contains("题材表不得进"));
    assert!(!out.contains("不得进本拍"));
    assert!(out.contains("旧剑将出鞘"));
    assert!(out.chars().count() <= ASSET_CHAR_BUDGET);
}
```

- [ ] **Step 2: 确认失败**（缺 `ContinueAssetsInput` / `render_continue_assets`）

- [ ] **Step 3: 实现**

```rust
pub struct ContinueAssetsInput<'a> {
    pub bundle: &'a WriteTimeBundle,
    pub admitted: &'a [String],
    pub roster: &'a [String],
    pub location: Option<&'a str>,
    pub next_node: &'a str,
    pub chapter_outline: &'a str,
    pub progress_lines: &'a [String],
    pub prior_prose: &'a str,
    pub tension_lines: &'a [String],
    pub arc_lines: &'a [String],
    pub logline: Option<&'a str>,
}

pub fn render_continue_assets(input: &ContinueAssetsInput<'_>) -> String {
    // 组装 sections（红线用 extract_redline_text；标题必须含「【⚠️ 世界观红线」与「【故事大纲」——
    // generate_chapter_outline 现网用 contains("【故事大纲") 短路，不得改掉这个子串。）
    // 风格/方法论/题材表/few-shot：不推进 sections。
    // 伏笔：pending 最多 3 + overdue 最多 1，标题保持 Bundle 习惯即可。
    // 最后 apply_asset_budget。
    // log::info admitted/roster/chars/truncated。
}
```

红线渲染调用 `crate::creative_engine::write_time_bundle::extract_redline_text`（已是 `pub(crate)`，agency 同 crate 可调）。

- [ ] **Step 4: 测试通过**

---

### Task 5: 接线 `build_writer_context_from_db`

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs`（`build_writer_context_from_db` 约 5394–5478）
- GitNexus：`impact({target:"build_writer_context_from_db", direction:"upstream"})`，记录 blast radius。若 HIGH/CRITICAL 先报用户。

- [ ] **Step 1: 先改现有测试契约（红）**

`test_build_continue_writer_context`（`agency/tests.rs` 约 1511）：单角色「阿苔」+ 近场正文，筛选后仍应含阿苔完整卡、红线、大纲、logline、第一章正文。不必改断言。

新增同文件测试（会红，直到接线完成）：

```rust
#[test]
fn build_writer_context_omits_orphan_and_full_dumps() {
    // 建故事：角色 阿苔（正文出现）+ 周奕辰（表内、正文/大纲均无）
    // 故事大纲堆叠两段相同【核心冲突】
    // 三场前文各 1500 字
    let ctx = build_writer_context_from_db(&pool, &story.id);
    assert!(ctx.contains("阿苔"));
    assert!(!ctx.contains("周奕辰"));
    assert_eq!(ctx.matches("【核心冲突】").count() <= 1 || ctx.matches("【核心冲突】皇权").count() == 1, true);
    assert!(!ctx.contains("【登场角色（必须严格遵循"));
    // 不再叠三场：第二/第三场独有标记不得出现
    assert!(!ctx.contains("第二场独有标记XYZ"));
}
```

`test_build_continue_writer_context_includes_emotional_attrs` / `relationships`：无 BeatCard 时录取前 8 人，甲乙都在前 8，断言保持。

- [ ] **Step 2: 跑测试确认新测试失败**

```
cd src-tauri && cargo test --lib build_writer_context_omits_orphan -- --nocapture
```

- [ ] **Step 3: 实现 load + 默认录取前 8**

```rust
pub(crate) struct ContinueContextParts {
    pub bundle: WriteTimeBundle,
    pub table_names: Vec<String>,
    pub scenes: Vec<crate::db::Scene>,
    pub tensions: Vec<crate::agency::emotional_ledger::InterpersonalTension>,
    pub arcs: Vec<crate::agency::emotional_ledger::EmotionalArc>,
    pub logline: Option<String>,
}

pub(crate) fn load_continue_context_parts(pool: &DbPool, story_id: &str) -> ContinueContextParts {
    // 现 build_writer_context_from_db 的 load 部分：bundle / tensions / arcs / logline / scenes
}

pub(crate) fn default_admitted(parts: &ContinueContextParts) -> Vec<String> {
    parts.table_names.iter().take(ADMITTED_CAP).cloned().collect()
}

pub(crate) fn evidence_blob(parts: &ContinueContextParts) -> String {
    // 近 5 场 content + characters_present join + story_outline
}

pub(crate) fn render_parts(
    parts: &ContinueContextParts,
    admitted: &[String],
    chapter_outline: &str,
    next_node: &str,
    location: Option<&str>,
    current_content: Option<&str>,
) -> String {
    let roster = build_roster(&parts.table_names, admitted, &evidence_blob(parts));
    let latest = parts.scenes.iter().max_by_key(|s| s.sequence_number)
        .and_then(|s| s.content.as_deref());
    let prior = slice_prior_prose(current_content.or(latest).unwrap_or(""));
    // tensions/arcs：只留录取名参与的，各 truncate 400
    render_continue_assets(&ContinueAssetsInput { ... })
}

pub(crate) fn build_writer_context_from_db(pool: &DbPool, story_id: &str) -> String {
    let parts = load_continue_context_parts(pool, story_id);
    let admitted = default_admitted(&parts);
    render_parts(&parts, &admitted, "", "", None, None)
}
```

`table_names` 从 `bundle.core_characters` 取即可，不必再查角色表。

- [ ] **Step 4: 相关测试通过**

```
cd src-tauri && cargo test --lib build_writer_context -- --nocapture
cd src-tauri && cargo test --lib test_build_continue_writer_context -- --nocapture
```

Expected: 旧注入测试 + 新 orphan 测试全绿。

---

### Task 6: 重排 `write_beat_once` + 大纲 LLM 角色变量

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs` `write_beat_once`（约 3694–3740）、`generate_chapter_outline`（约 3491–3612）
- GitNexus：`impact` 这两个符号。

- [ ] **Step 1: 写失败测试（规格 §7.6）**

在 `continue_assets.rs` 或 `coordinator.rs` tests 测 **组装函数**（避免真 LLM）：

抽出 `pub(crate) fn assemble_continue_user_prompt(...)` 供 `write_beat_once` 调用。测试：

```rust
#[test]
fn assembled_user_prompt_omits_non_admitted_emotional_core() {
    // 20 角色，current_content 只点 角色00/01/02
    // compile_beat_card 需要 DB：用 create_test_pool 建故事+20角色+正文含三名
    // 调用 assemble（或 write_beat_once 的同步半截）
    let user = assemble_continue_user_prompt(&pool, &story.id, "续写", &content, "");
    assert!(user.contains("情感内核：角色00") || user.contains("角色00"));
    assert!(!user.contains("情感内核：角色19的情感内核") && !user.contains("角色19的情感内核"));
}
```

`assemble_continue_user_prompt` 签名（同步，0 LLM）：

```rust
pub(crate) fn assemble_continue_user_prompt(
    pool: &DbPool,
    story_id: &str,
    instruction: &str,
    current_content: &str,
    chapter_outline: &str,
) -> Result<(String, crate::agency::beat_card::SceneBeatCard), AppError>
```

内部：`load_continue_context_parts` → `compile_beat_card` → `merge_admitted(present from card purpose「末段已在场」, parties, names_in(outline+instruction+next_node), card rest)` → `render_parts` → `render_writer_user_prompt`。

- [ ] **Step 2: 确认失败**

- [ ] **Step 3: 接线 write_beat_once**

替换现顺序：

```
assets_ctx = build_continue_writer_context   // 删掉这次全量倾倒
generate_chapter_outline(..., &assets_ctx)
compile_beat_card
render_writer_user_prompt(bundle, card, ...)
```

改为：

```
let parts = db { load_continue_context_parts }
let card = db { compile_beat_card }
let v1 = card.cast names
let roster_for_outline = build_roster(...)
let chars_var = admitted summaries（姓名+性格+目标一行，仅 v1）+ roster 行
let chapter_outline = generate_chapter_outline(..., characters_override: chars_var)
  // 短路：仍要求故事大纲存在。改 spawn_blocking 内：story_outline 空则 return ""。
  // 不要再用 assets_ctx.contains("【故事大纲")——此时尚未渲染全文。
  // characters vars.insert 用 characters_override，禁止 CharacterRepository 全表拼接。
let admitted = merge_admitted(present, parties, names_in(outline+instruction+next_node), rest)
let assets = render_parts(..., chapter_outline, card.next_outline_node, card.setting_location, Some(current_content))
let user = render_writer_user_prompt(&assets, &card, instr, current_content)
```

`generate_chapter_outline` 增加参数 `characters_override: String`。调用点：`write_beat_once` 与 `write_chapter`。world/progress 仍可查库（进度已是 3×200，可接受）；`story_outline` vars 改为 `condense_story_outline(db_outline, "")`，避免大纲 LLM 也被 blob 撑爆。

- [ ] **Step 4: 测试通过**

```
cd src-tauri && cargo test --lib assemble_continue_user_prompt assembled_user_prompt agency::continue_assets -- --nocapture
```

再跑可能调用 `generate_chapter_outline` 旧签名的测试：

```
cd src-tauri && cargo test --lib agency:: -- --nocapture
```

修编译错误直到绿。

---

### Task 7: `write_chapter` 共用筛选资产

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs` `write_chapter`（约 3849–3880）

- [ ] **Step 1: 把 `assets_ctx = build_continue_writer_context` 换成与 `write_beat_once` 相同的 `assemble` 产出的 assets 段**（不要 BeatCard 头尾——tool_loop task 不是 `render_writer_user_prompt`）。至少：`load` + 默认或 BeatCard 准入 + `render_parts`。优先 compile_beat_card（用最新场 content），与单章路径同一准入。

```rust
let (assets_ctx, _card) = self.db({
    let content = /* latest scene content or "" */;
    assemble_continue_user_prompt(...) // 若返回的是完整 user prompt，则再拆一个只返回 assets 的函数
})
```

更干净：`assemble` 返回 `(assets, card)`，`write_beat_once` 再 `render_writer_user_prompt`；`write_chapter` 只用 `assets`。

- [ ] **Step 2: 跑 `test_write_chapter_wrong_key_fails_loudly` 与 `test_continue_prose_fallback_failure_does_not_enter_tool_loop`**

```
cd src-tauri && cargo test --lib test_write_chapter_wrong_key test_continue_prose_fallback -- --nocapture
```

Expected: PASS（签名未变）。

---

### Task 8: 全量契约 + 文档回写（不 bump、不 commit）

- [ ] **Step 1:**

```
cd src-tauri && cargo test --lib
```

Expected: ≥1354 + 本计划新增，0 fail。`cargo +nightly fmt`。`python3 scripts/architecture_guard.py`。

- [x] **Step 2:** 设计文档状态已改为 `已发版 v0.42.0（真机 §8 未跑）`。`ROADMAP.md` 已知债务保留：ingest 大纲无界、跨故事脏角色不删、ContextPrioritizer 未接、§13 探针、本地连接 60s×2。

- [ ] **Step 3:** 发版时（用户说提交/推送）再 bump v0.42.0 + docs of record。本 Task 不改 Cargo.toml。

真机验收（规格 §8）需 LLM，登记为未关闭，不得宣称「上下文已智能选取」。

---

## Spec coverage

| 规格 | Task |
|---|---|
| §3 准入并集 + cap 8 优先级 | T1 `merge_admitted` + T6 接线 |
| §3 名单 + 脏名 | T2 |
| §4 红线/大纲/世界/卡/名单/关系/进度/前文/张力/弧光/logline/伏笔/Background 不进 | T3–T4 |
| §4 6000 预算 | T3 `apply_asset_budget` |
| §5 编排顺序 | T6 |
| §5 generate_chapter_outline 角色变量 | T6 |
| §5 无卡录取前 8 | T5 |
| §6 降级/日志 | T4 `log::info`；空表=空卡空名单（自然） |
| §7.1–7.5 | T1–T4 |
| §7.6 流程 | T6 |
| §2 不改 to_prompt | 全任务禁止改该函数 |
| write_chapter 共用 | T7 |

## 自检

- 无 TBD。关系规则=两端都在录取集。预算=6000。名单 cap=40。前文=末 800、1 场。
- `ContinueAssetsInput` 字段名在 T4 定义，T5–T7 沿用。
- `generate_chapter_outline` 短路改为读 DB 大纲是否为空，避免依赖尚未渲染的 `【故事大纲】` 子串；渲染侧仍使用该标题以兼容 `test_build_continue_writer_context`。
