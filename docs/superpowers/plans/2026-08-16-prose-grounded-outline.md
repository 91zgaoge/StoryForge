# 正文为大纲真相源 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 有正文时禁止按书名发明角色/大纲；大纲情节按创作方法论（默认 `scene_structure`）从正文归纳并往下推；管理 Agent 熔断改为后台续跑且不挡住续写；高峰未完不得换场换主角。

**Architecture:** 新增 `agency/prose_ground.rs` 纯函数门闩（0 LLM）。`ensure_assets` 在有正文时走提取+归纳，不再同步标题发明 tool_loop；`producer_out.aborted` salvage + `spawn_producer_resume`。`compile_next_node` / `probe_increment` 用方法论下一拍 + 场外开篇缺口。`materialize_assets` 落库前按正文过滤姓名。

**Tech Stack:** Rust (Tauri 2 / rusqlite)、现有 IngestPipeline、AgencyCoordinator、vitest 不改除非前端 toast。

**Spec:** `docs/plans/2026-08-16-prose-grounded-outline-design.md`

---

## File map

| File | Responsibility |
|---|---|
| Create: `src-tauri/src/agency/prose_ground.rs` | 正文阈值、姓名门闩、大纲接地、默认方法论 id、方法论下一拍文案 |
| Modify: `src-tauri/src/agency/mod.rs` | `pub mod prose_ground` |
| Modify: `src-tauri/src/agency/coordinator.rs` | `ensure_assets` / `ensure_story_outline` / 熔断不 Err / `spawn_producer_resume` |
| Modify: `src-tauri/src/agency/materialize.rs` | 角色/大纲落库前过滤 |
| Modify: `src-tauri/src/agency/beat_card.rs` | `compile_next_node` 未接地当空 + 方法论下一拍 |
| Modify: `src-tauri/src/agency/beat_state.rs` | 场外开篇探针 |
| Modify: `src-tauri/src/agency/continue_assets.rs` | roster 剔除未接地名 |
| Modify: `src-tauri/src/agency/tools.rs` | `story_info` 附开篇摘录 |
| Modify: `src-tauri/src/agency/tests.rs` | 集成契约 |
| Modify: docs of record | 仅发版任务，不在 P0 提交 |

---

### Task 1: `prose_ground` 纯函数门闩

**Files:**
- Create: `src-tauri/src/agency/prose_ground.rs`
- Modify: `src-tauri/src/agency/mod.rs`

- [ ] **Step 1: Write failing tests in `prose_ground.rs` `mod tests`**

合约：书名不能把费迪南送进正文是苏会山的故事。

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const PROSE: &str = "知启纪元八百四十七年。大奉帝国西北边陲重镇，黑崎州城。\
第二代镇北王苏会山端坐大堂。大少爷苏亦铁红装肃立。";

    #[test]
    fn substantial_prose_threshold() {
        assert!(!has_substantial_prose("短"));
        assert!(has_substantial_prose(&"字".repeat(200)));
    }

    #[test]
    fn title_inventions_dropped_when_absent_from_prose() {
        let names = ["费迪南三世", "艾拉", "苏会山", "苏亦铁"];
        let kept = filter_names_to_prose(&names, PROSE);
        assert!(kept.contains(&"苏会山".into()));
        assert!(kept.contains(&"苏亦铁".into()));
        assert!(!kept.iter().any(|n| n.contains("费迪南")));
        assert!(!kept.iter().any(|n| n == "艾拉"));
    }

    #[test]
    fn ferdinand_outline_is_not_grounded() {
        let outline = "第一卷·灰烬低语。费迪南三世为撑烟火节加征火药税。艾拉偷入工坊。";
        let candidates = ["费迪南三世", "艾拉", "苏会山"];
        assert!(!outline_is_grounded(outline, PROSE, &candidates));
    }

    #[test]
    fn su_family_outline_is_grounded() {
        let outline = "【转折点】景亲王送女，苏会山在镇北王府大堂迎亲。";
        let candidates = ["苏会山", "费迪南三世"];
        assert!(outline_is_grounded(outline, PROSE, &candidates));
    }

    #[test]
    fn default_methodology_is_scene_structure() {
        assert_eq!(DEFAULT_METHODOLOGY_ID, "scene_structure");
        assert_eq!(resolve_methodology_id(None), "scene_structure");
        assert_eq!(resolve_methodology_id(Some("hero_journey")), "hero_journey");
        assert_eq!(
            resolve_methodology_id(Some("custom_foo")),
            "custom_foo"
        );
    }

    #[test]
    fn scene_structure_next_beat_after_disaster_stays_in_shot() {
        let shot = "公主短刃扎进苏会山胸口。苏会山头脸崩裂，气绝。苏亦铁跪在红毡上。";
        let present = ["苏亦铁", "曹元佩"];
        let node = methodology_next_node("scene_structure", shot, &present);
        assert!(node.contains("苏亦铁") || node.contains("反应"));
        assert!(!node.contains("费迪南"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```
cd src-tauri && cargo test --lib agency::prose_ground -- --nocapture
```

Expected: compile fail `prose_ground` 模块不存在。

- [ ] **Step 3: Implement `prose_ground.rs` and register in `mod.rs`**

```rust
//! 有正文时资产/大纲必须接地。0 LLM。
//! 设计：docs/plans/2026-08-16-prose-grounded-outline-design.md

use crate::agency::continue_assets::{match_character_names, strip_editor_markup};

pub const DEFAULT_METHODOLOGY_ID: &str = "scene_structure";
pub const SUBSTANTIAL_PROSE_CHARS: usize = 200;
pub const STORY_INFO_PROSE_CHARS: usize = 800;

pub fn has_substantial_prose(text: &str) -> bool {
    strip_editor_markup(text).chars().count() >= SUBSTANTIAL_PROSE_CHARS
}

pub fn resolve_methodology_id(existing: Option<&str>) -> &str {
    match existing.map(str::trim).filter(|s| !s.is_empty()) {
        Some(id) => id,
        None => DEFAULT_METHODOLOGY_ID,
    }
}

pub fn filter_names_to_prose(names: &[impl AsRef<str>], prose: &str) -> Vec<String> {
    let plain = strip_editor_markup(prose);
    match_character_names(names, &plain)
}

pub fn name_in_prose(name: &str, prose: &str) -> bool {
    !filter_names_to_prose(&[name], prose).is_empty()
}

/// `candidate_names` 中出现在大纲里的姓名必须也出现在正文。
/// 大纲未点任何候选名 → 视为接地（避免空大纲误杀）。
pub fn outline_is_grounded(outline: &str, prose: &str, candidate_names: &[impl AsRef<str>]) -> bool {
    let mentioned: Vec<String> = match_character_names(candidate_names, outline);
    if mentioned.is_empty() {
        return true;
    }
    mentioned.iter().all(|n| name_in_prose(n, prose))
}

pub fn methodology_next_node(methodology_id: &str, shot: &str, present: &[impl AsRef<str>]) -> String {
    let names = present
        .iter()
        .map(|n| n.as_ref())
        .filter(|n| !n.is_empty())
        .collect::<Vec<_>>()
        .join("、");
    let cast = if names.is_empty() {
        "本场仍在场者".to_string()
    } else {
        names
    };
    let disaster = ["气绝", "刺", "死", "崩裂", "败", "灾难", "短刃"]
        .iter()
        .any(|s| shot.contains(s));
    match methodology_id {
        "scene_structure" if disaster => format!(
            "末句已是灾难。用场景结构写本场{cast}的反应、困境与决定，不得换场、不得换主角。"
        ),
        "scene_structure" => format!(
            "按场景结构推进：目标→冲突→灾难或反应→困境→决定。只写本场{cast}，不得另起开篇。"
        ),
        _ => format!("在硬约束内把当前冲突推进一步，只写本场{cast}，不得原地复述末句。"),
    }
}
```

`mod.rs` 在 `pub mod persist;` 旁加 `pub mod prose_ground;`。

- [ ] **Step 4: Run tests**

```
cd src-tauri && cargo test --lib agency::prose_ground -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```
git add src-tauri/src/agency/prose_ground.rs src-tauri/src/agency/mod.rs
git commit -m "feat: 正文姓名门闩纯函数（续写大纲不得按书名发明）"
```

---

### Task 2: 空方法论落库 `scene_structure`

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs`（`ensure_assets` 开头或新 `ensure_methodology_default`）
- Test: `src-tauri/src/agency/tests.rs`

- [ ] **Step 1: Failing test**

`test_ensure_methodology_default_writes_scene_structure_when_empty`：建故事不设 methodology，调用 helper，`get_by_id` 为 `scene_structure` / step 1。

`test_ensure_methodology_default_does_not_override_hero_journey`：已有 `hero_journey` 保持不变。

Helper 签名：

```rust
pub(crate) fn persist_default_methodology_if_empty(pool: &DbPool, story_id: &str) -> Result<(), AppError>
```

用 `StoryRepository::update` 且仅当 `story.methodology_id` 空。`methodology_step` 空时写 `1`。

- [ ] **Step 2: Implement + pass `cargo test --lib test_ensure_methodology_default -- --nocapture`**
- [ ] **Step 3: Commit** `feat: 未选定创作方法论时落库场景结构规范`

---

### Task 3: `materialize_assets` 落库前过滤

**Files:**
- Modify: `src-tauri/src/agency/materialize.rs`
- Test: 同文件 `mod tests` 或 `agency/tests.rs`

- [ ] **Step 1: Failing test**

种子故事 + 场景正文含苏会山、不含费迪南。黑板两条 character（苏会山 / 费迪南三世）。`materialize_assets` 后 `SELECT name FROM characters` 只有苏会山。

有正文为空（无 scenes）时两条都落库（创世路径）。

- [ ] **Step 2: Implement**

在 `materialize_assets` 开头读该 story 全部 `scenes.content` concat。`has_substantial_prose` 为真时，`character` 分支在 INSERT/UPDATE 前 `if !name_in_prose(&name, &prose) { continue; }`。`outline` 分支：收集本批 + 库内角色名作 `candidate_names`，`!outline_is_grounded` 则 skip。

- [ ] **Step 3: `cargo test --lib materialize -- --nocapture` 相关新测 PASS**
- [ ] **Step 4: Commit** `feat: 资产落库拒绝正文未出现的角色名`

---

### Task 4: `ensure_assets` 有正文不走标题发明；熔断不 Err

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs` `ensure_assets`（约 3051–3171）
- Test: `src-tauri/src/agency/tests.rs`

- [ ] **Step 1: Failing tests**

1. `test_ensure_assets_with_prose_does_not_require_producer_loop`：故事有 ≥200 字含苏会山的场景、角色表空、MockLlm 若被 producer tool_loop 调用则 panic。`ensure_assets` 返回 Ok。允许 0 或 1 次提取/大纲 complete（按 mock 计数断言 tool_loop 轮次为 0）。

2. `test_ensure_assets_producer_abort_returns_ok`：无正文或走旧补齐路径时，mock producer aborted；`ensure_assets` 仍 `Ok(())`，不包含「资产补齐未完成」。

实现要点：

- 先 `persist_default_methodology_if_empty`。
- `load_story_prose(pool, story_id)` concat scenes。
- 若 `has_substantial_prose`：`character_count==0` 时 **不要** 进入 `run_role_with_llm_and_budget` 标题补齐。改为：尝试 `list_items_for_story` materialize（已有门闩）；仍空则本拍继续（提取可 Task 5 再接）。`has_outline` 假或未接地 → 调改造后的 `ensure_story_outline`（Task 5）。
- 无论有无正文：`if producer_out.aborted` 改为 salvage materialize + `spawn_producer_resume`（Task 6 可先 no-op 函数）+ `Ok(())`。

- [ ] **Step 2: Implement until tests pass**
- [ ] **Step 3: Commit** `fix: 有正文时禁止按书名发明资产且管理熔断不挡住续写`

---

### Task 5: 从正文 + 方法论归纳大纲

**Files:**
- Modify: `coordinator.rs` `ensure_story_outline`
- Modify: `tools.rs` `StoryInfoTool`
- Test: `agency/tests.rs`

- [ ] **Step 1: Failing tests**

`test_ensure_story_outline_from_prose_uses_scene_structure_not_problem`：有正文苏家、mock complete 捕获 system/user。user 含「苏会山」或正文摘录；system 来自 `methodology_scene_structure`（或 user 含「目标→冲突→灾难」）。不得把「帝国的烟火」当情节前提唯一来源而无正文。

`test_ensure_story_outline_rejects_ungrounded_llm_output`：mock 返回费迪南大纲；落库后 `get_by_story` 仍空或仍为旧接地文，不得变成费迪南。

`test_story_info_includes_prose_excerpt`：场景有正文时 `story_info` 输出含正文片段。

- [ ] **Step 2: Implement**

`ensure_story_outline` user prompt：

- 有正文：`slice_prior_prose` 或开篇 600+近文 1800；system = `resolve_prompt_default(map_methodology_to_prompt_id(...))` fallback 场景结构全文；指令「只归纳已有正文；往下发展必须用该方法论且不得发明未出场主角」。
- 无正文：保留现 PROBLEM 路径。
- 落库前 `outline_is_grounded`。

`story_info`：query scenes concat，取前 `STORY_INFO_PROSE_CHARS`，追加「禁止发明下列正文未出现的姓名」。

- [ ] **Step 3: Tests PASS**
- [ ] **Step 4: Commit** `feat: 有正文时按创作方法论从章节归纳大纲`

---

### Task 6: `spawn_producer_resume`

**Files:**
- Modify: `coordinator.rs`（仿 `spawn_editor_qc`）

- [ ] **Step 1: Test** `test_spawn_producer_resume_noop_in_test_env`：`app_handle=None` 调用不 panic、不发 LLM。

- [ ] **Step 2: Implement**

```rust
fn spawn_producer_resume(&self, run_id: &str, story_id: &str) {
    let Some(app) = self.app_handle.clone() else {
        log::info!("agency: 测试环境跳过后台管理补齐 (run={})", run_id);
        return;
    };
    // spawn: BACKGROUND_LLM_SEMAPHORE → 读正文 → 提取/ensure_story_outline/ensure_world
    // 门闩；emit Producer start/done 后台补齐；独立 300s
}
```

`ensure_assets` aborted 臂调用它。

- [ ] **Step 3: Commit** `feat: 管理 Agent 熔断后后台续跑资产补齐`

---

### Task 7: `compile_next_node` 未接地当空 + 方法论下一拍

**Files:**
- Modify: `beat_card.rs` `compile_next_node`
- Existing tests: `compile_next_node_skips_book_beats_without_present_cast` 必须仍绿
- New: `compile_next_node_ignores_ungrounded_book_outline`

- [ ] **Step 1: Failing test**

种子：正文/current_content 为苏会山酒盏；书大纲为费迪南三卷（含或不含「苏会山」字样）。`compile_next_node` 不得含「费迪南」。应含方法论留场文案或苏会山。

若大纲含「苏会山在席间」且接地，可返回该句，但不得返回「费迪南得知苏会山遇刺」。

- [ ] **Step 2: Implement**

读 methodology_id（空则 `scene_structure`）。`outline_is_grounded` 假则 `outline=""`。候选句循环保留；全失败则 `methodology_next_node(id, shot_window, &present)`。

- [ ] **Step 3: `cargo test --lib compile_next_node -- --nocapture` PASS**
- [ ] **Step 4: Commit** `fix: 未接地书大纲不得充当续写下一节点`

---

### Task 8: 场外开篇探针 + roster 剔除未接地名

**Files:**
- Modify: `beat_state.rs` `probe_increment`
- Modify: `continue_assets.rs` 渲染 roster 处
- Test: `beat_state.rs` tests

- [ ] **Step 1: Failing test**

```rust
#[test]
fn probe_rejects_offshot_pov_opening() {
    // card.cast = 苏亦铁、曹元佩；increment 以费迪南三世开篇
    // 角色表候选通过 probe 的 offshot 检测：增量首 80 字点名费迪南且未点名在场者
}
```

需要让 `probe_increment` 知道「场外已登记名」。最小改动：增加参数 `roster_names: &[String]` 或把场外名放进 `BeatState`。**不要**破坏现有调用方语义。推荐 `BeatState` 增 `offshot: Vec<String>`（compile 时 = 表内名 − present），探针：无 `NewScene` 时，增量前 80 字 `match_character_names(&offshot)` 非空且 `match_character_names(&present)` 空 → gap `增量以场外角色开篇`。

`render_continue_assets`：构造 roster 前 `filter_names_to_prose`。

- [ ] **Step 2: Implement；更新 `probe_increment` 所有调用点（coordinator 一处 + 单测）**
- [ ] **Step 3: Commit** `fix: 续写探针拦截场外开篇并从名单剔除未接地名`

---

### Task 9: 热路径提前提取（角色表空且有正文）

**Files:**
- Modify: `coordinator.rs` `ensure_assets`

仅当 `has_substantial_prose && character_count==0 && app_handle.is_some()`：对 concat 正文调 `IngestPipeline::ingest`（timeout 建议 60s），失败 warn 继续。测试环境 skip（无 LLM）。契约用 mock 或纯「不调用 producer loop」已在 Task 4 覆盖。本任务加：ingest 成功后角色名须过门闩（ingest 走 asset_bridge，若 bridge 不过门闩则在 bridge 的 character upsert 复用 `name_in_prose`——**若 blast 过大则只在 ensure_assets 之后 DELETE 本 run 插入的未接地行禁止**；优先在 `asset_bridge` 角色写入点加 4 行过滤，impact `sync` 角色函数后改）。

- [ ] **Step 1: 对 `asset_bridge` 写角色处跑 GitNexus `impact`；HIGH 则只在 agency 侧过滤。**
- [ ] **Step 2: 测试 `filter` 已在 Task 1；此处加 `test_asset_bridge_skips_name_not_in_scene` 若改 bridge。**
- [ ] **Step 3: Commit** `feat: 续写前从已有正文提取角色且拒绝未接地名`

---

### Task 10: 核验与发版文档

- [ ] `cd src-tauri && cargo test --lib` 全绿
- [ ] `cd src-frontend && npx tsc --noEmit && npx vitest run` 与 `npm run format:check`
- [ ] `python3 scripts/architecture_guard.py`
- [ ] `cargo +nightly fmt`
- [ ] bump **v0.49.0** 四源 + `landing` `FALLBACK_VERSION` + docs of record（README / CHANGELOG / AGENTS / PROJECT_STATUS / ROADMAP / ARCHITECTURE / TESTING / USER_GUIDE）
- [ ] 对照设计 §11 六条契约：每条标明 executed 测试名。未跑真机不得写「已修复唱反调」。
- [ ] Commit + tag + push（仅当用户要求推送；本计划实现阶段先本地提交）

---

## Spec coverage

| 设计条款 | Task |
|---|---|
| 姓名门闩 / 大纲接地 | 1, 3 |
| 默认 scene_structure | 2, 5 |
| 有正文禁止标题发明 | 4 |
| 熔断不 Err + 后台续跑 | 4, 6 |
| 方法论归纳大纲 | 5 |
| story_info 正文摘录 | 5 |
| 脏大纲不注入 / next node | 7 |
| 留场探针 / roster | 8 |
| 提取提前 | 9 |
| 验收发版 | 10 |

## 非目标（计划内不要做）

恢复 TimeSliced、接 ContextPrioritizer、改 `to_prompt()`、DELETE 费迪南脏行、宣称真机已修复。
