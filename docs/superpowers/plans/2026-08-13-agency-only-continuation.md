# Agency 三角色唯一续写路径 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 创世与幕前/幕后续写只走 Agency 三角色；幕前续写同章追加并强制戏剧换场；删除 TimeSliced/TriShot 续写引擎。

**Architecture:** `smart_execute` 续写与 `agency_continue_chapter` 都进入 `AgencyCoordinator::run_continue(PersistMode)`。Append 把增量写入当前 `scenes` 行；NextChapter 才建新行。Producer 用 Rust 编译 SceneBeatCard；LeadWriter 单次 `complete()`；Editor 后台质检。WriteTimeBundle 成为唯一上下文编译器。

**Tech Stack:** Tauri 2 / Rust / rusqlite / React 18。设计文档：`docs/plans/2026-08-13-agency-only-continuation-design.md`。

**验证基线（开始前跑一次记下数字）：** `cd src-tauri && cargo test --lib`；`cd src-frontend && npx tsc --noEmit && npx vitest run`。不得回归。

---

## File map

| 文件 | 职责 |
|---|---|
| Create `src-tauri/src/agency/persist.rs` | `PersistMode`、Append 落库、拍计数读写 |
| Create `src-tauri/src/agency/beat_card.rs` | `SceneBeatCard` 纯 Rust 编译 + prompt 渲染 |
| Modify `src-tauri/src/agency/mod.rs` | 注册新模块 |
| Modify `src-tauri/src/agency/coordinator.rs` | `run_continue` 吃 PersistMode；单次主创；后台编辑；Append 装配 |
| Modify `src-tauri/src/agency/commands.rs` | 幕后续写仍 NextChapter |
| Modify `src-tauri/src/commands/orchestrator.rs` | 续写分支改调 Agency；增加 `scene_id` 参数 |
| Modify `src-tauri/src/domain/write_time_bundle.rs` | `CoreCharacter` 情感四元组 |
| Modify `src-tauri/src/creative_engine/write_time_bundle.rs` | `to_prompt` 情感 + 关系段 |
| Modify `src-tauri/src/creative_engine/expansion/{mod,ledger,debt}.rs` | asset_history 读写兼容 beats 对象；按名匹配；按拍计债务 |
| Modify `src-tauri/src/memory/asset_bridge.rs` | ingest 同时写 `characters_present` |
| Modify `src-tauri/src/planner/{mod,executor}.rs` | 续写不再 TimeSliced/beat 链 |
| Modify `src-frontend/src/services/api/intent.ts` | `smartExecute()` 封装透传 `scene_id`（直接 invoke 点在此，不在 FrontstageApp） |
| Modify `src-frontend/src/frontstage/FrontstageApp.tsx` | 两处 `smartExecute({...})` 调用点（约 :3513/:4505）传当前 `sceneId` |
| Modify `src-frontend/src/frontstage/components/RichTextEditor.tsx` | 内联建议（:893）/ 划词改写（:1003）：均传 `selected_text`，禁止走 Agency Append |
| Modify `src-tauri/src/agency/eval_harness.rs` | `run_continue` 新签名 |
| Modify `src-frontend/src/pages/settings/GeneralSettings.tsx` | 去掉续写 generation_mode 语义 |
| Modify `src-frontend/src/pages/settings/UnifiedModelManager.tsx` | `plan_mode`（约 :493）标注已废弃或隐藏 |
| Test `src-tauri/src/agency/tests.rs` 与各模块 `#[cfg(test)]` | 契约测试 |

改写/审稿的 planner Full/Fast **不动**。

**漏网入口（实施时必改，否则续写/改写会串路）：**

| 文件 | 原因 |
|---|---|
| `src-frontend/src/frontstage/components/RichTextEditor.tsx` :893 / :1003 | 另两处 `smartExecute`：内联建议（:893）/ 划词改写（:1003），均传 `selected_text`，**禁止**走 Agency Append |
| `src-tauri/src/agency/eval_harness.rs` :256 | `run_continue(&run_id, &story_id, chapter)` 旧三参，Task 2 必须改 |
| `src-tauri/src/agency/coordinator.rs` `run_continue_batch` | 仍调 `write_chapter`；Task 3 只改单章续写默认路径，batch 改为调 `write_beat_once` 或暂时保留 tool_loop 但必须编译通过 |

`python3 scripts/verify-ipc-manifest.py` **只核对命令名**，不核对参数。`scene_id` 必须同时加进 Tauri 命令签名与 `intent.ts` 透传，否则前端字段会被静默丢弃。

---

### Task 1: PersistMode + Append 落库纯函数

**Files:**
- Create: `src-tauri/src/agency/persist.rs`
- Modify: `src-tauri/src/agency/mod.rs`（加 `pub mod persist;`）
- Test: 同文件 `#[cfg(test)]`

- [ ] **Step 1: 写失败测试**

在 `persist.rs` 先只放测试（类型尚未存在，应编译失败）：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::create_test_pool;
    use crate::db::repositories::{CreateStoryRequest, SceneRepository, StoryRepository};

    fn long_increment() -> String {
        "续写增量正文。".repeat(30) // 7 * 30 = 210 字，满足 ≥200 落库门槛
    }

    fn seed_story_with_scene(pool: &crate::db::DbPool) -> (String, String) {
        let story = StoryRepository::new(pool.clone())
            .create(CreateStoryRequest {
                title: "追加测试".into(),
                description: None,
                genre: Some("玄幻".into()),
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: None,
                reference_book_id: None,
            })
            .unwrap();
        let scene_repo = SceneRepository::new(pool.clone());
        let mut conn = pool.get().unwrap();
        let tx = conn.transaction().unwrap();
        let scene = scene_repo
            .create_in_tx(&tx, &story.id, 1, Some("第一章"))
            .unwrap();
        scene_repo
            .update_in_tx(
                &tx,
                &scene.id,
                &crate::db::repositories::SceneUpdate {
                    content: Some("旧文开头。".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        tx.commit().unwrap();
        (story.id, scene.id)
    }

    #[test]
    fn append_does_not_create_new_scene_row() {
        let pool = create_test_pool().unwrap();
        let (story_id, scene_id) = seed_story_with_scene(&pool);
        let out = persist_append(
            &pool,
            &PersistMode::Append {
                scene_id: scene_id.clone(),
            },
            "旧文开头。",
            &long_increment(),
        )
        .unwrap();
        assert_eq!(out.scene_id, scene_id);
        let scenes = SceneRepository::new(pool.clone())
            .get_by_story(&story_id)
            .unwrap();
        assert_eq!(scenes.len(), 1);
        let expected = format!("旧文开头。\n\n{}", long_increment());
        assert_eq!(scenes[0].content.as_deref(), Some(expected.as_str()));
    }

    #[test]
    fn append_missing_scene_id_is_err() {
        let pool = create_test_pool().unwrap();
        let err = persist_append(
            &pool,
            &PersistMode::Append {
                scene_id: "no-such".into(),
            },
            "旧",
            "新",
        )
        .unwrap_err();
        assert!(err.to_string().contains("请先打开一个章节") || err.to_string().contains("不存在"));
    }
}
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cd src-tauri && cargo test --lib persist::tests::append_does_not_create_new_scene_row -- --nocapture`

Expected: 编译失败（`PersistMode` / `persist_append` 未定义）

- [ ] **Step 3: 最小实现**

```rust
//! 续写落库模式：Append（当前章）与 NextChapter（新章）的纯数据 + Append 写库。

use crate::db::DbPool;
use crate::error::AppError;

#[derive(Debug, Clone)]
pub enum PersistMode {
    Append { scene_id: String },
    NextChapter { chapter_number: i32 },
}

#[derive(Debug, Clone)]
pub struct AppendPersistOutcome {
    pub scene_id: String,
    pub chapter_number: i32,
    pub full_content: String,
}

/// 将 current_content + increment 写入已有 scene。禁止 create 新行。
pub fn persist_append(
    pool: &DbPool,
    mode: &PersistMode,
    current_content: &str,
    increment: &str,
) -> Result<AppendPersistOutcome, AppError> {
    let PersistMode::Append { scene_id } = mode else {
        return Err(AppError::from("persist_append 只接受 Append"));
    };
    let repo = crate::db::repositories::SceneRepository::new(pool.clone());
    let scene = repo
        .get_by_id(scene_id)
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::validation_failed("请先打开一个章节", Some("no_scene")))?;
    let cleaned_old = current_content.trim();
    let cleaned_inc = increment.trim();
    if cleaned_inc.chars().count() < 200 {
        return Err(AppError::from("续写增量过短，拒绝落库"));
    }
    let full = if cleaned_old.is_empty() {
        cleaned_inc.to_string()
    } else {
        format!("{cleaned_old}\n\n{cleaned_inc}")
    };
    repo.update(
        scene_id,
        &crate::db::repositories::SceneUpdate {
            content: Some(full.clone()),
            ..Default::default()
        },
    )
    .map_err(AppError::from)?;
    Ok(AppendPersistOutcome {
        scene_id: scene.id,
        chapter_number: scene.sequence_number,
        full_content: full,
    })
}
```

`SceneRepository::update` 若签名不是 `(id, &SceneUpdate)`，按 `scene_repository.rs` 现有 `update` / `update_in_tx` 对齐（优先 `update_in_tx` 包一层事务）。增量 <200 字与设计「熔断不丢稿 ≥200」一致。

单测用纯文本即可。生产路径（Task 4）幕前必须传 **HTML**（`getHTML()`），否则 `persist_append` 会用 `getText()` 覆写整章、毁掉 TipTap 标记。Task 1 不处理 HTML 包 `<p>`；Task 4 接入时：`full = current_html + "<p>" + html_escape(increment) + "</p>"`（旧文已是 HTML 时），纯文本旧文仍用 `\n\n`。

`mod.rs` 增加 `pub mod persist;`。

- [ ] **Step 4: 跑测试确认通过**

Run: `cd src-tauri && cargo test --lib persist:: -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/agency/persist.rs src-tauri/src/agency/mod.rs
git commit -m "$(cat <<'EOF'
feat: 续写 PersistMode::Append 同章落库不新建章

EOF
)"
```

---

### Task 2: run_continue 接受 PersistMode；Append 走 persist_append

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs`（`AgencyContinueResult`、`run_continue`、`run_continue_inner`）
- Modify: `src-tauri/src/agency/commands.rs`（幕后仍 NextChapter）
- Modify: `src-tauri/src/agency/tests.rs`（现有 `run_continue("rc-1", &story.id, 2)` 全部改为 NextChapter；实测 12 处：:1110/:1188/:1463/:2145/:2175/:2240/:2386/:2422/:2934/:3118/:3214/:3246）
- Modify: `src-tauri/src/agency/eval_harness.rs` :256（旧三参，漏改则编译失败）

**本任务选定策略（不要再写依赖 LLM 的 coordinator 测试）：** 只改签名 + NextChapter 包装；「Append 不建章」由 Task 1 `persist_append` 覆盖；「协调器 Append 不建章」推迟到 Task 12，用 `persist_append_with_card` 绕过 LLM。

- [ ] **Step 1: 写失败测试**

本任务不追加 `test_run_continue_append_keeps_scene_count`（无 mock LLM 必失败）。只改现有调用点，编译失败即红：

并改现有 `run_continue("id", story, 2)` 调用点为：

```rust
.run_continue(
    "rc-1",
    &story.id,
    PersistMode::NextChapter { chapter_number: 2 },
    "",
    None,
)
```

- [ ] **Step 2: 跑测试确认失败（签名不对）**

Run: `cd src-tauri && cargo test --lib agency::tests -- --nocapture`

Expected: 编译失败

- [ ] **Step 3: 改签名并接上**

`AgencyContinueResult` 增加：

```rust
pub increment: String, // 本拍增量，供幕前 appendAiContent
```

`run_continue` 改为：

```rust
pub async fn run_continue(
    &self,
    run_id: &str,
    story_id: &str,
    persist: crate::agency::persist::PersistMode,
    instruction: &str,
    current_content: Option<&str>,
) -> Result<AgencyContinueResult, AppError>
```

`run_continue_inner` 同样接收 `persist` / `instruction` / `current_content`。

装配处（今日 `handle_gate` 里 `create_in_tx`）：

```rust
match &persist {
    PersistMode::Append { .. } => {
        let increment = cleanup_prose_for_persist(&draft.content, story_id).await;
        let outcome = crate::agency::persist::persist_append(
            &self.pool,
            &persist,
            current_content.unwrap_or(""),
            &increment,
        )?;
        // finish_run completed；spawn_editor_qc；返回 AgencyContinueResult {
        //   scene_id: outcome.scene_id, chapter_number: outcome.chapter_number,
        //   increment, revised: false, verdict: EditorVerdict::pending(),
        // }
    }
    PersistMode::NextChapter { chapter_number } => {
        // 保留现有 create+update 单事务
    }
}
```

`commands.rs` `agency_continue_chapter`：

```rust
coordinator
    .run_continue(
        &rid,
        &story_id,
        PersistMode::NextChapter { chapter_number },
        "续写下一章",
        None,
    )
```

全文件 `rg 'run_continue\(' src-tauri` 改齐（含 `eval_harness.rs`）。`increment` 字段所有 `AgencyContinueResult {` 字面量补 `increment: String::new()` 或真实增量。`run_continue_batch` 本任务不改路径，只保证仍能编译（仍调现有 `write_chapter` / `handle_gate`）。

- [ ] **Step 4: 跑测试**

Run: `cd src-tauri && cargo test --lib agency:: -- --nocapture`

Expected: 现有 continue 测试 PASS；新签名编译通过

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/agency
git commit -m "$(cat <<'EOF'
feat: run_continue 支持 Append/NextChapter 两种落库

EOF
)"
```

---

### Task 3: 续写主创单次 complete + 编辑后台化

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs`（`write_chapter` / `run_continue_inner`）

契约：Append 与 NextChapter 在资产已注入后，主创走 `writer_prose_fallback` 同类单次 `complete()`，不再默认 tool_loop。编辑走已有 `spawn_editor_qc`，`run_continue` 在装配后立刻 `finish_run`，不等质检。

- [ ] **Step 1: 写失败测试**

**不要**再写 `EditorVerdict::pending()` 单测——`agency/tests.rs` 已有 `test_editor_verdict_pending_defaults`，会立刻绿、测不到本任务。改为断言 `AgencyContinueResult` 字面量在装配后 `verdict.verdict == "pending"`（可在无 LLM 的 persist 装配测，或推迟到 Task 12）。

本任务路径改动：`write_chapter` 开头改为调单次 complete（复制 `writer_prose_fallback` 的 `llm.complete`，task 文本用 `instruction` + `assets_ctx`）。tool_loop 仅当 `complete` 结果 `< 200` 字时再试一次 fallback（已有）。`run_continue_batch` 改为调 `write_beat_once`（或暂时保留 tool_loop，但必须编译通过）。

- [ ] **Step 2: 跑测试**

Run: `cd src-tauri && cargo test --lib test_editor_verdict_pending -- --nocapture`

- [ ] **Step 3: 改 `run_continue_inner`**

将步骤 3「质量门 + handle_gate」改为：

1. `write_beat_once`（单次 complete，上下文暂用现有 `build_continue_writer_context`）
2. 按 PersistMode 装配（Task 2）
3. `self.spawn_editor_qc(run_id, story_id, premise, &draft)`
4. **不要**再 `evaluate_gate` 同步等待
5. 外层 `run_continue` 已有的 `finish_run(completed)` 保持在 Ok 之后立刻执行

`write_chapter` 的 tool_loop 保留为 `write_beat_once` 失败后的最后手段（设计 §4）。

- [ ] **Step 4: 跑 agency 测试**

Run: `cd src-tauri && cargo test --lib agency:: -- --nocapture`

Expected: PASS（依赖 LLM 的测试若本来 skip/mock 则不变）

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/agency/coordinator.rs src-tauri/src/agency/tests.rs
git commit -m "$(cat <<'EOF'
feat: 续写主创单次生成、编辑审计后台化

EOF
)"
```

---

### Task 4: smart_execute 续写改调 Agency Append

**Files:**
- Modify: `src-tauri/src/commands/orchestrator.rs`（`smart_execute` / `smart_execute_inner` 加 `scene_id: Option<String>` **和** `selected_text: Option<String>`；外层超时包装必须把两参传到 inner）
- Modify: `src-frontend/src/services/api/intent.ts`（`SmartExecuteRequest` + `smartExecute()` 透传 `scene_id`）
- Modify: `src-frontend/src/frontstage/FrontstageApp.tsx`（两处续写调用传 `scene_id`）
- Modify: `src-frontend/src/frontstage/components/RichTextEditor.tsx`（:893 内联建议 / :1003 划词改写：传 `selected_text`，**不**传 scene_id 走 Append；后端见下方硬门）
- Modify: 所有 `smartExecute` / `smart_execute` 测试 mock
- Run: `python3 scripts/verify-ipc-manifest.py`（只核命令名；参数靠 tsc + 命令签名）

- [ ] **Step 1: 写失败测试**

在 `orchestrator.rs` 或现有 orchestrator 测试模块：

```rust
#[test]
fn continuation_requires_scene_id_for_append() {
    let err = resolve_persist_mode(true, None, false).unwrap_err();
    assert!(err.to_string().contains("请先打开一个章节"));
    let ok = resolve_persist_mode(true, Some("s1".into()), false).unwrap();
    match ok {
        crate::agency::persist::PersistMode::Append { scene_id } => {
            assert_eq!(scene_id, "s1")
        }
        _ => panic!("expected Append"),
    }
    let next = resolve_persist_mode(true, None, true).unwrap();
    match next {
        crate::agency::persist::PersistMode::NextChapter { chapter_number } => {
            assert_eq!(chapter_number, 0); // 占位；幕后自己算 MAX+1，不走此函数填真实章号
        }
        _ => panic!("expected NextChapter"),
    }
}
```

把 `resolve_persist_mode` 放在 `orchestrator.rs` 底部或 `agency/persist.rs`：

```rust
pub fn resolve_persist_mode(
    is_continuation: bool,
    scene_id: Option<String>,
    explicit_next_chapter: bool,
) -> Result<PersistMode, AppError> {
    if !is_continuation {
        return Err(AppError::from("resolve_persist_mode 仅用于续写"));
    }
    if explicit_next_chapter {
        return Ok(PersistMode::NextChapter { chapter_number: 0 }); // 0 = 调用方事后填 MAX+1
    }
    let sid = scene_id.filter(|s| !s.is_empty()).ok_or_else(|| {
        AppError::validation_failed("请先打开一个章节", Some("no_scene"))
    })?;
    Ok(PersistMode::Append { scene_id: sid })
}
```

`chapter_number: 0` 仅占位；幕后命令自己算 MAX+1，不走此函数。

- [ ] **Step 2: 跑测试失败**

Run: `cd src-tauri && cargo test --lib continuation_requires_scene_id -- --nocapture`

- [ ] **Step 3: smart_execute_inner 续写分支**

在 `is_bootstrap_intent` 块之后、现有「Phase 3 加载场景 / PlanExecutor」之前插入：

```rust
if classification.is_continuation
    && selected_text.as_deref().map(str::trim).unwrap_or("").is_empty()
{
    // 硬门：有 selected_text = 划词改写/内联建议，必须留 PlanExecutor Full，禁止 Append。
    // 今日后端 PlanContext.selected_text 恒为 None（前端已传、命令未收）——本任务一并接入，否则改写会被误分类成续写后吞进 Agency。
    let persist = crate::agency::persist::resolve_persist_mode(true, scene_id.clone(), false)?;
    let run_id = Uuid::new_v4().to_string();
    let coordinator = AgencyCoordinator::new(app_handle.clone(), pool.clone());
    let story_id = current_story_id.clone().ok_or_else(|| {
        AppError::validation_failed("请先在左侧选择或创建一个作品", Some("no_story_selected"))
    })?;
    let result = coordinator
        .run_continue(
            &run_id,
            &story_id,
            persist,
            &user_input,
            current_content.as_deref(),
        )
        .await?;
    return Ok(crate::planner::PlanExecutionResult {
        success: true,
        steps_completed: 1,
        final_content: Some(result.increment),
        messages: vec!["续写完成".into()],
        error: None,
        result_kind: None,
    });
}
```

注意：`current_scene_id` 今日在 Phase 3 才加载（本分支之前不存在该变量）——**不做**「最新有内容场景」回退（设计 §3.2 禁止猜测），前端必须传 `scene_id`，缺失即报「请先打开一个章节」。`explicit_next_chapter` 本期恒为 `false`（幕前不接「明确下一章」入口，新章只由幕后 `agency_continue_chapter` 与现有自动分章产生）；`resolve_persist_mode` 保留该参数仅为契约完整，测试照写。

`smart_execute` 与 `smart_execute_inner` 增加参数 `scene_id: Option<String>` 与 `selected_text: Option<String>`。PlanContext 构造处把 `selected_text: None` 改为命令入参（今日 :782 硬编码 None）。

前端改动点：`intent.ts` 的 `smartExecute()`（约 :83）封装签名加 `scene_id?: string` 并透传（`selected_text` 已透传）；`FrontstageApp.tsx` 两处 `smartExecute({...})`（约 :3513 / :4505）传入：

```ts
scene_id: useFrontstageStore.getState().sceneId ?? undefined,
current_content: editorRef.current?.getHTML() ?? editorRef.current?.getText(),
```

续写两处（FrontstageApp :3513 / :4505）今日传的是 `getText()`。必须改成 `getHTML()`，与 `flushSceneSave` 同源，避免 persist_append 把整章 HTML 压成纯文本。`RichTextEditor.tsx` 内联建议（:893）与划词改写（:1003）**已经**传 `getHTML()` + `selected_text`、不传 `scene_id`；不要改成 Append，也不要再「补」getHTML。

搜 `smart_execute` / `smartExecute` 改齐测试 mock 参数。

- [ ] **Step 4: 验证**

Run:

```
python3 scripts/verify-ipc-manifest.py
cd src-tauri && cargo test --lib continuation_requires_scene_id
cd src-frontend && npx tsc --noEmit
```

Expected: 命令名仍在清单；Rust 测试 PASS；tsc 通过。**不要**把 ipc-manifest 绿当成 `scene_id` 已接线。

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/orchestrator.rs src-tauri/src/agency/persist.rs \
  src-frontend/src/services/api/intent.ts \
  src-frontend/src/frontstage/FrontstageApp.tsx \
  src-frontend/src/frontstage/components/RichTextEditor.tsx
git commit -m "$(cat <<'EOF'
feat: 幕前续写改走 Agency Append 唯一路径

EOF
)"
```

---

### Task 5: Bundle 角色卡补情感四元组 + 关系段

**Files:**
- Modify: `src-tauri/src/domain/write_time_bundle.rs`（`CoreCharacter`）
- Modify: `src-tauri/src/creative_engine/write_time_bundle.rs`（`load_sync` 映射、`to_prompt`）
- Test: `write_time_bundle.rs` 现有 `to_prompt_*` 测试旁

- [ ] **Step 1: 写失败测试**

```rust
#[test]
fn to_prompt_includes_emotional_fields_and_relationships() {
    let mut bundle = empty_bundle(); // 复用文件内已有 helper
    bundle.core_characters = vec![CoreCharacter {
        name: "沈炼".into(),
        identity: None,
        physical_state: None,
        mental_state: None,
        location: None,
        personality: Some("隐忍".into()),
        emotional_core: Some("压抑的悲愤".into()),
        emotional_trigger: Some("被背叛".into()),
        emotional_wound: Some("师父之死".into()),
        emotional_need: Some("讨回公道".into()),
    }];
    bundle.relationship_lines = vec![
        "沈炼 -> 顾长夜：社会关系=同僚 ｜ 情感=仇恨[0.9]".into(),
    ];
    let prompt = bundle.to_prompt();
    assert!(prompt.contains("情感内核：压抑的悲愤"));
    assert!(prompt.contains("情感创伤：师父之死"));
    assert!(prompt.contains("角色情感关系"));
    assert!(prompt.contains("顾长夜"));
}
```

`empty_bundle` 若构造体缺新字段，测试会逼你补字段。注意：`write_time_bundle.rs` 测试模块**没有**现成 `empty_bundle`（现有同名 helper 在 `prompt_synthesis/manifest.rs` 与 `prompt_synthesis/mod.rs` 的测试里），需在本文件测试模块新建一个。

- [ ] **Step 2: 跑测试失败**

Run: `cd src-tauri && cargo test --lib to_prompt_includes_emotional_fields -- --nocapture`

- [ ] **Step 3: 实现**

`CoreCharacter` 增加四个 `Option<String>`。`WriteTimeBundle` 增加 `pub relationship_lines: Vec<String>`。

`load_sync` 映射 `c.emotional_*`；关系用 `CharacterRelationshipRepository::get_by_story`，渲染格式对齐 `build_writer_context_from_db` 的 `■ {} -> {}` 行。

**张力/弧光段必须并入（防回归），但禁止 `creative_engine` → `agency`：** 现行 `build_writer_context_from_db`（coordinator.rs:5217-5226）注入了 `emotional_ledger` 的 `load_tensions`/`render_tensions_for_prompt` 与 `load_arcs`/`render_arcs_for_prompt`。`creative_engine/write_time_bundle.rs` 今日 **零** `crate::agency` 引用；`load_sync` **不要**调用 `emotional_ledger`（层倒置，architecture_guard 虽不禁但会把编译器绑死在 agency）。

正确做法：Task 5 只把情感四元组 + `relationship_lines` 放进 Bundle；Task 6 的 `build_writer_context_from_db` 在 `to_prompt()` **之后**由 coordinator 追加张力/弧光段（agency 已依赖 ledger）。`build_progression_anchor` 随 TimeSliced 删除（Task 11）后，这条 agency 拼接是张力段唯一入口。

`to_prompt` 在角色段每行追加非空情感字段；角色段后插入：

```rust
if !self.relationship_lines.is_empty() {
    sections.push(format!(
        "【角色情感关系（真实情感，可与表面关系不一致）】\n{}\n要求：言行须与情感关系一致。",
        self.relationship_lines.join("\n")
    ));
}
```

所有 `WriteTimeBundle {` 字面量补 `relationship_lines: vec![]` 与四个情感 `None`。

- [ ] **Step 4: 跑测试**

Run: `cd src-tauri && cargo test --lib write_time_bundle:: -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/domain/write_time_bundle.rs src-tauri/src/creative_engine/write_time_bundle.rs
git commit -m "$(cat <<'EOF'
feat: WriteTimeBundle 注入情感四元组与角色关系

EOF
)"
```

---

### Task 6: 续写上下文改用 Bundle 编译器（删第二套真源）

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs`（`build_continue_writer_context` / `build_writer_context_from_db`）
- Modify: `src-tauri/src/creative_engine/context_prioritizer.rs` 调用点（在编译器内）

- [ ] **Step 1: 写失败测试**

```rust
#[test]
fn continue_context_contains_wound_from_bundle() {
    let pool = create_test_pool().unwrap();
    let story_id = seed_story_with_character_wound(&pool, "沈炼", "师父之死");
    let ctx = crate::agency::coordinator::build_writer_context_from_db(&pool, &story_id);
    assert!(ctx.contains("师父之死"), "续写上下文必须含情感创伤");
    assert!(ctx.contains("世界观红线") || ctx.contains("登场角色") || ctx.contains("沈炼"));
}
```

种子：建故事 + 角色行写入 `emotional_wound`。`build_writer_context_from_db` 改为内部 `WriteTimeBundle::load_sync(...).to_prompt()`。

- [ ] **Step 2: 跑测试失败**（今日 Agency 函数已含创伤则本测试可能已绿——若已绿，断言再加 `relationship_lines` 或红线段，确保走 Bundle）

若今日函数已含创伤，改断言 `ctx.contains("【登场角色")`（Bundle 标题）以锁定切换。**另加一条防回归断言**：种子再加一条带 `emotional_bond` 的角色关系，断言 `ctx.contains("情感张力驱动")`——确认 Task 6 由 coordinator 在 `to_prompt()` 后拼接的张力段仍输出（现行 `build_writer_context_from_db` 有此注入，丢失即回归；张力不进 Bundle）。

- [ ] **Step 3: 实现**

```rust
pub(crate) fn build_writer_context_from_db(pool: &DbPool, story_id: &str) -> String {
    let mut s = match WriteTimeBundle::load_sync(pool, story_id, 1, None, None, None) {
        Ok(b) => b.to_prompt(),
        Err(e) => {
            log::warn!("continue compiler bundle 失败: {e}");
            String::new()
        }
    };
    // 张力/弧光留在 agency 层拼接，不让 creative_engine 依赖 agency
    // load_tensions / load_arcs 返回 Vec，不是 Result
    let tensions = crate::agency::emotional_ledger::load_tensions(pool, story_id);
    let rendered = crate::agency::emotional_ledger::render_tensions_for_prompt(&tensions);
    if !rendered.is_empty() {
        s.push_str("\n\n");
        s.push_str(&rendered);
    }
    let arcs = crate::agency::emotional_ledger::load_arcs(pool, story_id);
    let rendered = crate::agency::emotional_ledger::render_arcs_for_prompt(&arcs);
    if !rendered.is_empty() {
        s.push_str("\n\n");
        s.push_str(&rendered);
    }
    s
}
```

`chapter_number` 能拿到就传入，拿不到用 1。禁止保留旧的手写字段清单作为并行真源。

Prioritizer：在 `to_prompt` 之后不要再拆一遍。阶段 3 BeatCard 注入时再用 `ContextChunk`。本任务只合一 Bundle。（设计 §6.3 的 Critical/High/Normal 分级排序**本期不实施**：BeatCard 双锚（Task 9）已承担 Critical 优先级职责，设计文档该条视为降级，Task 12 文档同步时注明。）

- [ ] **Step 4: 跑测试**

Run: `cd src-tauri && cargo test --lib continue_context_contains_wound -- --nocapture`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/agency/coordinator.rs
git commit -m "$(cat <<'EOF'
refactor: 续写上下文改用 WriteTimeBundle 唯一编译器

EOF
)"
```

---

### Task 7: 拍计数 + 按拍扩张债务

**Files:**
- Modify: `src-tauri/src/agency/persist.rs`（读写 `stories.asset_history_json` 的拍字段）
- Modify: `src-tauri/src/creative_engine/expansion/mod.rs`（**必改**：`read_asset_history`/`append_asset_history` 是该列现有唯一读写者，整体重写 JSON；不兼容新对象格式会把 `beats` 字段抹掉）
- Modify: `src-tauri/src/creative_engine/expansion/debt.rs`
- Modify: `src-tauri/src/creative_engine/expansion/ledger.rs`（按 **角色名** 匹配 `characters_present`）

`asset_history_json` 今日是 `[{chapter, ids}]`。扩展为对象：

```json
{
  "assets": [{ "chapter": 1, "ids": ["beat_card.x"] }],
  "beats": {
    "append_beats": 3,
    "last_conflict_beat": 1,
    "last_cast_refresh_beat": 0,
    "last_location_beat": 2,
    "last_foreshadow_beat": 0
  }
}
```

读写必须兼容旧数组：旧数组 → `{ "assets": <旧>, "beats": { append_beats:0, ...} }`。

- [ ] **Step 1: 写失败测试**

```rust
#[test]
fn beat_debt_triggers_conflict_after_two_appends_without_conflict() {
    let beats = BeatCounters {
        append_beats: 2,
        last_conflict_beat: 0,
        last_cast_refresh_beat: 0,
        last_location_beat: 0,
        last_foreshadow_beat: 0,
    };
    let debt = ExpansionDebt::from_beats(&beats);
    assert!(debt.conflict >= 2);
    let items = debt.triggered();
    assert!(items.contains(&QuotaItem::ConflictEscalation));
}

#[test]
fn ledger_matches_character_present_by_name() {
    // seed scenes.characters_present = ["林雪"]（名字不是 id）
    // 林雪 last_seen 应非 0
}
```

`from_beats`：`conflict = append_beats - last_conflict_beat`（`last==0` 且 `append_beats>0` 且角色表非空 → 视为已停滞 `append_beats`，**不再**当旧书零干扰）。

- [ ] **Step 2: 跑测试失败**

Run: `cd src-tauri && cargo test --lib beat_debt_triggers_conflict -- --nocapture`

- [ ] **Step 3: 实现**

`BeatCounters` 放 `persist.rs`。`increment_append_beat(pool, story_id)` 在每次成功 Append/NextChapter 后调用。回流冲突/换场/阵容时更新对应 `last_*_beat = append_beats`。

`expansion/mod.rs` 兼容改造（与 beats 读写同 Task 落地）：`read_asset_history` 接受旧数组与新对象两种形态（旧数组 → `{assets: <旧>, beats: 全 0}`）；`append_asset_history` 改写时保留既有 `beats` 字段，不得整体覆盖抹掉。

`ledger.rs`：`characters_present` JSON 既可能是 id 也可能是名（存量脏数据：`creation_commands.rs:226` 写 id、`story_system/auto_contract.rs:951` 写名，schema 注释说 id——写入口径统一不在本期范围，记入 Task 12 文档「已知遗留」）。`last_seen` 的 key 用 **name**：先建 `id→name` map，present 里每项若是 id 则译名，否则当名。

`debt.rs`：`compute` 若 `beats.append_beats > 0` 则 `from_beats`，否则保留章级逻辑给从未 Append 的旧书。

- [ ] **Step 4: 跑测试**

Run: `cd src-tauri && cargo test --lib expansion:: -- --nocapture`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/agency/persist.rs src-tauri/src/creative_engine/expansion
git commit -m "$(cat <<'EOF'
feat: 扩张债务改按续写拍数计并按角色名匹配出场

EOF
)"
```

---

### Task 8: SceneBeatCard 编译器（阵容/冲突/情感/下一节点）

**Files:**
- Create: `src-tauri/src/agency/beat_card.rs`
- Modify: `src-tauri/src/agency/mod.rs`

- [ ] **Step 1: 写失败测试**

```rust
#[test]
fn beat_card_cast_includes_silent_character_when_three_exist() {
    let pool = create_test_pool().unwrap();
    let sid = seed_three_chars_one_silent(&pool); // 阿岩每章在场，林雪 3 拍未出场，顾长夜在场
    let card = compile_beat_card(&pool, &sid, "阿岩站在雨里。顾长夜冷笑。").unwrap();
    assert!(card.cast.len() >= 3 && card.cast.len() <= 8);
    assert!(card.cast.iter().any(|c| c.name == "林雪"));
    assert!(!card.conflict_move.action.is_empty());
    assert!(!card.emotion_beat.summary.is_empty());
    assert!(!card.next_outline_node.is_empty());
}

#[test]
fn beat_card_never_empty_slots() {
    let pool = create_test_pool().unwrap();
    let sid = seed_story_minimal(&pool); // 1 个角色，无关系无大纲
    let card = compile_beat_card(&pool, &sid, "他走了。").unwrap();
    assert!(!card.conflict_move.action.is_empty());
    assert!(!card.emotion_beat.summary.is_empty());
    assert!(!card.next_outline_node.is_empty());
}
```

- [ ] **Step 2: 跑测试失败**

Run: `cd src-tauri && cargo test --lib beat_card:: -- --nocapture`

- [ ] **Step 3: 实现 `compile_beat_card`**

类型：

```rust
pub struct CastMember { pub name: String, pub purpose: String }
pub struct ConflictMove {
    pub action: String, // 加压|反转|代价显现 + 双方 + 赌注
    pub parties: Vec<String>,
}
pub struct EmotionBeat { pub summary: String }
pub struct SceneBeatCard {
    pub cast: Vec<CastMember>,
    pub conflict_move: ConflictMove,
    pub emotion_beat: EmotionBeat,
    pub next_outline_node: String,
    pub expansion_quota: Vec<crate::creative_engine::expansion::debt::QuotaItem>,
    pub setting_location: Option<String>,
}

impl SceneBeatCard {
    pub fn render_full(&self) -> String { /* 【本章节拍任务】四块 */ }
    pub fn render_tail_summary(&self) -> String {
        format!(
            "【节拍摘要】上场：{}｜冲突：{}｜情感：{}｜推进：{}",
            self.cast.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join("、"),
            self.conflict_move.action.chars().take(40).collect::<String>(),
            self.emotion_beat.summary.chars().take(40).collect::<String>(),
            self.next_outline_node.chars().take(40).collect::<String>(),
        )
    }
}
```

编译规则按设计 §5，全部确定性、0 LLM。降级句写死在函数内：

- 冲突空：`"{主角} 必须在本拍与阻力正面对峙，不得只靠对话过渡。"`
- 情感空：`本拍必须让 {主角} 的需求受阻并露出情绪代价。`
- 节点空：`在硬约束内把当前冲突推进一步，不得原地复述末句。`

`expansion_quota` 来自 Task 7 `debt.triggered()`。

- [ ] **Step 4: 跑测试通过**

Run: `cd src-tauri && cargo test --lib beat_card:: -- --nocapture`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/agency/beat_card.rs src-tauri/src/agency/mod.rs
git commit -m "$(cat <<'EOF'
feat: SceneBeatCard 纯 Rust 编译本拍阵容冲突情感与大纲节点

EOF
)"
```

---

### Task 9: 主创 prompt 双锚注入 BeatCard + 末句锚点顺序

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs`（`write_beat_once`）
- Modify: `src-tauri/src/agents/orchestrator.rs` 仅 **复用** `build_ending_anchor`（已存在，`pub(crate)`，取末 2 句）。

依赖事实（已核实）：`architecture_guard` 只强制 `db`/`domain` 两条禁令，**并不禁止 agency→agents**（`agency/gate.rs`、`agency/graders.rs` 现已 import `crate::agents`）。因此直接 `use crate::agents::orchestrator::build_ending_anchor` 合规。但为控制 coordinator 对编排器的耦合面，选定仍为：**复制 20 行末句锚点到 `agency/beat_card.rs` 的 `fn ending_anchor`**（与 orchestrator 逻辑相同：取末 2 句），并加单测。

- [ ] **Step 1: 写失败测试**

```rust
#[test]
fn writer_prompt_order_is_card_then_body_then_summary_then_ending() {
    let card = SceneBeatCard {
        cast: vec![CastMember { name: "林雪".into(), purpose: "回归质问".into() }],
        conflict_move: ConflictMove { action: "加压：当众揭穿".into(), parties: vec!["林雪".into(), "阿岩".into()] },
        emotion_beat: EmotionBeat { summary: "林雪伤口=被抛弃".into() },
        next_outline_node: "夜宴破裂".into(),
        expansion_quota: vec![],
        setting_location: Some("夜宴".into()),
    };
    let prompt = render_writer_user_prompt(
        "【红线】不可飞天",
        &card,
        "往下写",
        "他推开门。",
    );
    let i_card = prompt.find("【本章节拍任务】").unwrap();
    let i_sum = prompt.find("【节拍摘要】").unwrap();
    let i_end = prompt.find("必须从上述末句").unwrap_or(prompt.find("末句").unwrap());
    assert!(i_card < i_sum);
    assert!(i_sum < i_end);
    assert!(prompt.contains("林雪"));
}
```

- [ ] **Step 2: 跑测试失败**

Run: `cd src-tauri && cargo test --lib writer_prompt_order -- --nocapture`

- [ ] **Step 3: `render_writer_user_prompt` + 接入 `write_beat_once`**

```rust
pub fn render_writer_user_prompt(
    bundle_prompt: &str,
    card: &SceneBeatCard,
    instruction: &str,
    current_content: &str,
) -> String {
    format!(
        "{card_full}\n\n{bundle}\n\n【本次创作指令】\n{instruction}\n\n\
         须在节拍任务硬约束内落实指令核心意图。\n\n{card_tail}\n\n{ending}",
        card_full = card.render_full(),
        bundle = bundle_prompt,
        instruction = instruction,
        card_tail = card.render_tail_summary(),
        ending = ending_anchor(current_content),
    )
}
```

`write_beat_once`：`compile_beat_card` → `build_writer_context_from_db` → `render_writer_user_prompt` → `llm.complete`。

- [ ] **Step 4: 跑测试通过**

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/agency/beat_card.rs src-tauri/src/agency/coordinator.rs
git commit -m "$(cat <<'EOF'
feat: 主创续写 prompt 头尾双锚注入 SceneBeatCard

EOF
)"
```

---

### Task 10: 写后回流出场/冲突/地点/进度

**Files:**
- Modify: `src-tauri/src/agency/persist.rs`（`persist_append` 增加 card 元数据）
- Modify: `src-tauri/src/agency/coordinator.rs` NextChapter `SceneUpdate`
- Modify: `src-tauri/src/memory/asset_bridge.rs`（`sync_scene_outline` 同时写 `characters_present`）

- [ ] **Step 1: 写失败测试**

```rust
#[test]
fn append_writeback_sets_characters_present_names() {
    let pool = create_test_pool().unwrap();
    let (story_id, scene_id) = seed_story_with_scene(&pool);
    let card = SceneBeatCard {
        cast: vec![
            CastMember { name: "阿岩".into(), purpose: "守门".into() },
            CastMember { name: "林雪".into(), purpose: "质问".into() },
        ],
        conflict_move: ConflictMove {
            action: "加压".into(),
            parties: vec!["阿岩".into(), "林雪".into()],
        },
        emotion_beat: EmotionBeat { summary: "怒".into() },
        next_outline_node: "夜宴破裂".into(),
        expansion_quota: vec![],
        setting_location: Some("夜宴厅".into()),
    };
    persist_append_with_card(&pool, &scene_id, "旧文。", &long_increment(), &card).unwrap();
    let scene = SceneRepository::new(pool.clone()).get_by_id(&scene_id).unwrap().unwrap();
    assert!(scene.characters_present.contains(&"阿岩".into()));
    assert!(scene.characters_present.contains(&"林雪".into()));
    assert_eq!(scene.setting_location.as_deref(), Some("夜宴厅"));
    assert!(scene.outline_content.unwrap_or_default().contains("夜宴破裂"));
}
```

ingest 测试：`sync_scene_outline` 在 `characters_present` 列为空时写入 delta 里的名字数组（不是只写大纲文本）。

- [ ] **Step 2: 跑测试失败**

- [ ] **Step 3: 实现**

`persist_append_with_card`：`SceneUpdate` 填 `content`、`characters_present: Some(names)`、`setting_location`、`character_conflicts`（name→id 查 `characters` 表，查不到则 `character_a_id` 用名字字符串——列是 TEXT，账本已按名匹配）、`outline_content`：若现有为空或 source 为机器，则追加一行 `进度：{next_outline_node}`，**不覆盖**用户长大纲（若 `outline_content` 已有且不含「进度：」则 append 一行）。

成功后 `increment_append_beat`；若 conflict/cast/location 有更新则写对应 `last_*_beat`。

`asset_bridge.rs` `sync_scene_outline` 的 UPDATE 增加 `characters_present = ?`，值为 `serde_json::to_string(&so.characters_present)`，守卫与 outline 相同（空或机器 source）。

- [ ] **Step 4: 跑测试通过**

Run: `cd src-tauri && cargo test --lib append_writeback_sets_characters_present -- --nocapture`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/agency/persist.rs src-tauri/src/agency/coordinator.rs src-tauri/src/memory/asset_bridge.rs
git commit -m "$(cat <<'EOF'
feat: 续写回流出场冲突地点与进度指针

EOF
)"
```

---

### Task 11: 删除续写 TimeSliced / TriShot / beat 链

**Files:**
- Modify: `src-tauri/src/planner/executor.rs`（`execute_writer` 路由）
- Modify: `src-tauri/src/planner/mod.rs`（`sanitize_plan_for_prose_request` 续写不再重建 beat_planner→writer）
- Modify: `src-frontend/src/pages/settings/GeneralSettings.tsx`（`generation_mode` 选项 :511-534）
- Modify: `src-frontend/src/pages/settings/UnifiedModelManager.tsx`（`plan_mode` 选项约 :493——**不在** GeneralSettings）
- Modify: `src-tauri/src/config/settings.rs` 注释（`generation_mode` 仅改写）

- [ ] **Step 1: 写失败测试**

在 `planner/mod.rs` 测试：

```rust
#[test]
fn continuation_sanitize_does_not_insert_beat_planner() {
    let mut plan = /* 单 writer 步 */;
    let ctx = PlanContext { intent_classification: Some(cls_continuation()), ..empty() };
    PlanGenerator::sanitize_plan_for_prose_request(
        &mut plan,
        ctx.intent_classification.as_ref(),
        &ctx,
        "beat",
    );
    assert!(plan.steps.iter().all(|s| s.capability_id != "beat_planner"));
    assert_eq!(plan.steps[0].capability_id, "writer");
}
```

`execute_writer`：若仍被误调且 `is_continuation`，应 `Err` 而不是 TimeSliced：

```rust
if plan_context.intent_classification.as_ref().map(|c| c.is_continuation).unwrap_or(false) {
    return Err(AppError::from(
        "续写必须走 AgencyCoordinator，禁止 PlanExecutor TimeSliced/TriShot",
    ));
}
```

创世同理 `is_new_novel`。

- [ ] **Step 2: 跑测试失败**（今日续写会插入 beat_planner）

- [ ] **Step 3: 实现删除语义**

- `sanitize_plan_for_prose_request`：`is_continuation` 时塌缩为 **单 writer**（不再 beat 链）。创世不应进入 PlanExecutor（已由 smart_execute 分流）；若进入同样拒绝。
- `execute_writer`：continuation/new_novel 直接 Err。
- `generation_mode` UI：选项只留「改写：自动（有选中文本走完整质检）/ 快速」。删除 `time_sliced`、`tri_shot` 作为续写说明。`plan_mode`（在 `UnifiedModelManager.tsx` 约 :493，不在 GeneralSettings）标注已废弃或隐藏。
- **不要**本任务删除 `execute_time_sliced` 函数体（改写若仍引用 Fast/Full 即可）。确认 `rg execute_time_sliced` 仅剩定义与测试后，下一提交再删函数。本任务先断路由。

- [ ] **Step 4: 验证**

```
cd src-tauri && cargo test --lib planner::
cd src-frontend && npx tsc --noEmit && npx vitest run
python3 scripts/architecture_guard.py
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/planner src-tauri/src/config src-frontend/src/pages/settings
git commit -m "$(cat <<'EOF'
refactor: 续写切断 TimeSliced/TriShot/beat 链，只留 Agency

EOF
)"
```

---

### Task 12: 路由契约 + 回归 + 文档状态

**Files:**
- Test: `src-tauri/src/agency/tests.rs` 补 `test_run_continue_append_keeps_scene_count`（若 Task 2 因 LLM 未做，此处用 persist+writeback 集成、writer 用注入的现成 draft 绕过 LLM：在测试里直接 `persist_append_with_card`）
- Modify: `docs/plans/2026-08-13-agency-only-continuation-design.md` 状态改为「实施中/已实施」
- 不在本任务 bump 版本、不推送（除非用户明确要求发版）

- [ ] **Step 1: 集成测试清单（全部要绿）**

1. Append 不增加 `scenes` 行数  
2. NextChapter 增加一行  
3. BeatCard 三人含沉寂  
4. 回流 `characters_present` 为名  
5. 两拍无冲突 → 第三拍 quota 含冲突  
6. prompt 含 `emotional_wound`  
7. 现有 `run_genesis` 测试不回退  
8. `cargo test --lib` 全量；`npx tsc --noEmit`；`npx vitest run`；`cargo +nightly fmt`；`python3 scripts/architecture_guard.py`
9. **情感张力不回归**：故事存在带 `emotional_bond` 的关系时，续写上下文含 `情感张力驱动` 段（emotional_ledger 由 coordinator 在 `to_prompt()` 后拼接承接，Task 6）
10. **文档同步**：设计正文三处必改——① §3.2「旧文以 current_content 为准」补「HTML（getHTML）」；② §3.2「允许格式化差异」**改写勿删**（两端均 HTML 后，persist 与 `flushSceneSave` 仍可能因 TipTap 规范化/`<p>` 包裹产生标记差异，后写覆盖前写不视为错误；禁止的是 persist 用 `getText()` 把整章压成纯文本）；③ §10「熔断 ≥200 必写回」与 Task 1「<200 拒写」对齐为同一门槛的两面（<200 垃圾稿拒落库，≥200 熔断也必写回）。另两处非正文改写、只加注：④ §6.3 ContextPrioritizer 分级排序本期降级（BeatCard 双锚承担 Critical）；⑤ `characters_present` 写入口径（id vs 名字混杂）记「已知遗留」。

- [ ] **Step 2: 跑全量**

```
cd src-tauri && cargo +nightly fmt && cargo test --lib
cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check
python3 scripts/architecture_guard.py
```

- [ ] **Step 3: 修到全绿**

- [ ] **Step 4: Commit**

```bash
git add -u
git commit -m "$(cat <<'EOF'
test: 三角色唯一续写路径契约与回归

EOF
)"
```

---

## Spec coverage（自检）

| 设计章节 | 任务 |
|---|---|
| §3 唯一路径 / smart_execute | Task 4, 11 |
| §3.1 戏剧换场 vs 切章 | Task 8 换场硬任务；Task 1–2 不建章 |
| §3.2 PersistMode | Task 1–2 |
| §3.3 并发 finish_run + 后台编辑 | Task 3 |
| §4 三角色职责 | Task 3, 8–9 |
| §5 SceneBeatCard | Task 8–9 |
| §6 Bundle 编译器 | Task 5–6（情感四元组+关系在 Bundle；张力/弧光由 coordinator 在 `to_prompt()` 后拼接，禁止 creative_engine→agency；§6.3 Prioritizer 分级本期降级） |
| §7 回流 + 账本名匹配 + 按拍债务 | Task 7, 10 |
| §8 删除 TimeSliced/TriShot | Task 11 |
| §9 前端 scene_id / increment | Task 4 |
| §10 无 scene_id 错误 / ≥200 落库 | Task 1, 4 |
| §11 测试契约 | Task 8, 10, 12 |
| 改写/审稿不动 | Task 11 只断续写路由 |

无 TBD。`PersistMode`、`SceneBeatCard`、`BeatCounters` 名称前后一致。
