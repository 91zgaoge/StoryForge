# 按正文重写生产资产 Implementation Plan

> **For agentic workers:** 已批准并落地 v0.53.0。对照 `docs/plans/2026-08-21-asset-refresh-from-prose-design.md`。

**Goal:** 幕前「将故事大纲按照现有正文重新写过」只更新 `story_outlines`，纸面不变；同一路由能按指令改角色 / 世界观 / 当前场大纲。

**Architecture:** 分类后置纠正 + 0 LLM 靶解析 → `smart_execute` 在 Append 之前走 `run_asset_refresh`（Producer 一次 JSON）→ 接地过滤后按靶 persist → `result_kind=asset_refresh`。

**Tech Stack:** Tauri 2 / Rust / React；复用 `prose_ground`、`asset_bridge::is_refinable`、`merge_current_scene_outline`、`cap_story_outline_content`、幕前 `audit_report` 分流。

---

### Task 1: 靶解析纯函数

**Files:**
- Modify: `src-tauri/src/agency/asset_refresh.rs`（新建）
- Test: 同文件 `#[cfg(test)]`

- [ ] **Step 1: 写失败测试**

```rust
#[test]
fn parse_targets_story_outline_only() {
    let t = parse_asset_refresh_targets("将故事大纲按照现有正文重新写过");
    assert_eq!(t, vec![AssetRefreshTarget::StoryOutline]);
}

#[test]
fn parse_targets_characters_and_world() {
    let t = parse_asset_refresh_targets("把角色和世界观按正文重写");
    assert!(t.contains(&AssetRefreshTarget::Characters));
    assert!(t.contains(&AssetRefreshTarget::World));
    assert!(!t.contains(&AssetRefreshTarget::StoryOutline));
}

#[test]
fn parse_targets_all_assets() {
    let t = parse_asset_refresh_targets("按正文重写全部设定");
    assert_eq!(t.len(), 4);
}

#[test]
fn parse_targets_empty_when_unspecified() {
    assert!(parse_asset_refresh_targets("帮我看看").is_empty());
}
```

- [ ] **Step 2: `cargo test --lib parse_targets_story_outline_only` 必须红**
- [ ] **Step 3: 实现 `AssetRefreshTarget` + `parse_asset_refresh_targets`（关键词表见设计 §3）**
- [ ] **Step 4: 测试绿**

---

### Task 2: 分类后置纠正，禁止当续写

**Files:**
- Modify: `src-tauri/src/intent.rs`（`build_classification_prompt` 示例 + `parse_classification_json` 后置）
- Modify: `src-tauri/src/creative_engine/asset_capability_manifest.rs`（`AssetTaskType::AssetRefresh`）

- [ ] **Step 1: 测试**

```rust
#[test]
fn asset_refresh_instruction_is_not_continuation_or_prose() {
    let mut c = WritingIntentClassification {
        is_continuation: true,
        is_prose_request: true,
        task_type: AssetTaskType::Continuation,
        ..WritingIntentClassification::conservative_fallback()
    };
    apply_asset_refresh_override(
        &mut c,
        "将故事大纲按照现有正文重新写过",
    );
    assert!(!c.is_continuation);
    assert!(!c.is_prose_request);
    assert_eq!(c.task_type, AssetTaskType::AssetRefresh);
}

#[test]
fn continuation_override_does_not_force_prose_when_asset_refresh() {
    // 现有：is_continuation || is_new_novel => is_prose=true
    // 本作业必须排在该纠正之后或从中豁免
}
```

- [ ] **Step 2: 提示词增加示例**：「将故事大纲按照现有正文重新写过」→ `is_new_novel=false, is_continuation=false, task_type=asset_refresh, is_prose=false`
- [ ] **Step 3: `task_type` JSON 枚举加入 `asset_refresh`**
- [ ] **Step 4: `cargo test --lib asset_refresh_instruction` 绿**
- [ ] **Step 5: `planner/mod.rs` `should_force_correct_to_writer`：AssetRefresh 对 outline_planner 不改 writer**

---

### Task 3: persist 接地与四靶写入

**Files:**
- Modify: `src-tauri/src/agency/asset_refresh.rs`
- Reuse: `prose_ground::name_in_prose`、`observe::merge_current_scene_outline`、`asset_bridge` 合并、`cap_story_outline_content`

- [ ] **Step 1: 测试（内存 pool）**

```rust
#[test]
fn persist_story_outline_does_not_touch_scene_content() { /* 写大纲前后 scenes.content 相等 */ }

#[test]
fn persist_drops_names_absent_from_prose() { /* 金敏秀不进大纲/角色 */ }

#[test]
fn persist_preserves_user_created_emotional_core() { /* source=user_created */ }

#[test]
fn persist_refines_ingest_character() { /* source=ingest 可改 */ }

#[test]
fn persist_scene_outline_keeps_handwritten_prefix() { /* 前缀 + 【当前场大纲】 */ }
```

- [ ] **Step 2: 红 → 实现 `persist_asset_refresh(pool, story_id, scene_id, targets, parsed)`**
- [ ] **Step 3: 故事大纲走 cap；角色/世界按 `is_refinable`；场景用 `merge_current_scene_outline`**
- [ ] **Step 4: 测试绿**

---

### Task 4: `run_asset_refresh` + `smart_execute` 入口

**Files:**
- Modify: `src-tauri/src/commands/orchestrator.rs`（Append 判断之前）
- Modify: `src-tauri/src/agency/coordinator.rs` 或 `asset_refresh.rs` 异步 LLM
- Modify: `src-tauri/src/agency/persist.rs` 若需 `should_agency_append_continue` 文档化「asset_refresh 不得为 continuation」

- [ ] **Step 1: 测试** `should_agency_append_continue(false, None) == false`；集成：mock LLM 返回大纲 JSON，断言 result_kind 与表

```rust
#[test]
fn asset_refresh_result_kind_is_not_prose() {
    let r = PlanExecutionResult {
        success: true,
        steps_completed: 1,
        final_content: Some("已按正文重写故事大纲".into()),
        messages: vec![],
        error: None,
        result_kind: Some("asset_refresh".into()),
    };
    assert_eq!(r.result_kind.as_deref(), Some("asset_refresh"));
}
```

- [ ] **Step 2: `smart_execute`：`targets` 非空且（分类为 AssetRefresh 或关键词覆盖）→ `run_asset_refresh`，return，不进 Append**
- [ ] **Step 3: 无正文 / 无故事 / 有 blocking creative run → AppError 可读中文**
- [ ] **Step 4: Producer `complete_json`，失败不写库**
- [ ] **Step 5: 相关 `cargo test --lib` 绿**

---

### Task 5: 幕前不进纸面

**Files:**
- Modify: `src-frontend/src/frontstage/FrontstageApp.tsx`（两处 `audit_report` 旁）
- Modify: `src-frontend/src/services/api/intent.ts` 注释
- Test: `src-frontend/src/frontstage/__tests__/FrontstageApp.asset-refresh.test.tsx`（仿 `FrontstageApp.audit-report.test.tsx`）

- [ ] **Step 1: vitest：mock `result_kind: 'asset_refresh'`，编辑器文本不变，出现成功 toast 文案**
- [ ] **Step 2: 红 → 实现 early return + `invalidateQueries`（story outline / characters / world / scenes）**
- [ ] **Step 3: `npx vitest run` 该文件绿；`npx tsc --noEmit`**

---

### Task 6: 对照设计核验直到可上线

- [ ] `cargo test --lib` 全绿
- [ ] `npx tsc --noEmit` / `npx vitest run` / `python3 scripts/architecture_guard.py` / `cargo +nightly fmt`
- [ ] 设计 §8 十条探针逐条 executed，缺口就改
- [ ] bump **v0.53.0** 四源 + `FALLBACK_VERSION` + README / CHANGELOG / AGENTS / PROJECT_STATUS / ROADMAP / ARCHITECTURE / TESTING / USER_GUIDE
- [ ] 推送双远程 + tag + `gh run list` 盯到 macOS/Windows/Linux 全绿

**USER_GUIDE 一句：** 在幕前输入「将故事大纲按照现有正文重新写过」会按已写章节重写幕后故事大纲，不会往正文里加字。角色 / 世界观 / 场景大纲同理，把名字换进指令即可。手改过的角色卡默认不覆盖。正在续写时请等结束。
