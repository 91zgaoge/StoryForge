# 续写质量闭合 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 堵住 Agency 续写编译器空转，使角色出场、情节推进、前后文因果在 P0–P3 后由契约测试保护；真机八拍未跑前不得宣称三症状已修复。

**Architecture:** 不改路由与 `PersistMode`。P0 修债务/节点/冲突/写回/回退；P1 别名与场次；P2 新增 `beat_state.rs` 状态网 + 一次探针重试；P3 八拍 mock 契约。热路径不为选资产加 LLM。

**Tech Stack:** Rust / rusqlite 测试池。设计：`docs/plans/2026-08-15-continue-quality-closure-design.md`。

**验证基线：** `cd src-tauri && cargo test --lib`。无前端逻辑变更则不强制 vitest；每阶段结束跑 `cargo test --lib` + `python3 scripts/architecture_guard.py`。

**提交纪律：** 用户未说「提交」前不 `git commit`。勾掉各 Task 的 Commit 步。

**禁止：** 改 `WriteTimeBundle::to_prompt()`；接 `ContextPrioritizer`；热路径 LLM 选 key；改 `ExpansionDebt::quota_text` 原文（planner 仍用「章」）；宣称 §13 真机探针已过；恢复 TimeSliced/TriShot。

**GitNexus：** 改下列符号前 `impact({target, direction:"upstream"})`，HIGH/CRITICAL 先告知用户再改：`compile_beat_card`、`persist_append_with_card`、`write_beat_once`、`render_continue_assets`、`writer_prose_fallback`、`ending_anchor`、`render_writer_user_prompt`、`merge_progress_line`（若变成 pub）、`touch_refresh_beats`。

---

## File map

| 文件 | 职责 |
|---|---|
| Modify `src-tauri/src/agency/persist.rs` | `beat_refresh_flags`、进度行累积、事实写回、债务按旗标刷新、P2 位置回写 |
| Modify `src-tauri/src/agency/beat_card.rs` | `quota_text_for_beats`、冲突看阵容、下一节点不回绕、P1 阵容规则、末句降权、prompt 插状态网 |
| Modify `src-tauri/src/agency/continue_assets.rs` | 别名点名、地点 shift、冲突/目标段 |
| Create `src-tauri/src/agency/beat_state.rs` | BeatState + probe（P2） |
| Modify `src-tauri/src/agency/mod.rs` | `pub mod beat_state;`（P2） |
| Modify `src-tauri/src/agency/coordinator.rs` | 续写回退；传入本场地点；P2 探针重试 |
| Modify `src-tauri/src/agency/tests.rs` | 八拍契约（P3 收口） |
| Test 各模块 `#[cfg(test)]` | 设计 §10 |

漏网：`planner/executor.rs` 的 `quota_text()` **不改**。`writer_prose_fallback` 创世调用保留。`agency/tools.rs` 的 `bundle.to_prompt()` 不改。

阶段交付：P0 完成后幕前续写可发；P1/P2/P3 依次叠加。不要把 P2 文件在 P0 就创建空壳。

---

## P0 — 编译器修洞

### Task 1: 债务旗标纯函数 + 进度行累积

**Files:**
- Modify: `src-tauri/src/agency/persist.rs`

- [ ] **Step 1: 写失败测试**

在 `persist.rs` 的 `#[cfg(test)]` 末尾追加（函数尚不存在，应编译失败）：

```rust
    #[test]
    fn beat_refresh_flags_conflict_only_when_parties_in_increment_or_verbs() {
        let flags = beat_refresh_flags(
            "两人继续喝茶聊天。",
            &["阿岩".into()],
            &["阿岩".into(), "林雪".into()],
            Some("雨巷"),
            Some("雨巷"),
            &["阿岩".into(), "林雪".into()],
            &[],
        );
        assert!(!flags.conflict);
        assert!(!flags.cast);
        assert!(!flags.location);
        assert!(!flags.foreshadow);
    }

    #[test]
    fn beat_refresh_flags_conflict_when_both_parties_named() {
        let flags = beat_refresh_flags(
            "阿岩逼视林雪，林雪没有退。",
            &["阿岩".into(), "林雪".into()],
            &["阿岩".into()],
            Some("雨巷"),
            Some("雨巷"),
            &["阿岩".into(), "林雪".into()],
            &[],
        );
        assert!(flags.conflict);
        assert!(flags.cast);
    }

    #[test]
    fn beat_refresh_flags_conflict_verb_without_both_names() {
        let flags = beat_refresh_flags(
            "对峙已经无法再拖。",
            &["阿岩".into()],
            &["阿岩".into()],
            None,
            None,
            &["阿岩".into(), "林雪".into()],
            &[],
        );
        assert!(flags.conflict);
    }

    #[test]
    fn merge_progress_line_appends_distinct_nodes() {
        let once = merge_progress_line(None, "夜宴破裂");
        assert_eq!(once, "进度：夜宴破裂");
        let twice = merge_progress_line(Some(&once), "密诏败露");
        assert!(twice.contains("进度：夜宴破裂"));
        assert!(twice.contains("进度：密诏败露"));
        let dup = merge_progress_line(Some(&twice), "密诏败露");
        assert_eq!(dup.matches("进度：密诏败露").count(), 1);
    }
```

把现有 `fn merge_progress_line` 从私有改为 `pub(crate)`，以便测试与 `compile_next_node` 语义对齐。现有覆盖逻辑必须改掉，本测试会在 Step 3 后失败如果仍覆盖。

- [ ] **Step 2: 跑测试确认失败**

```
cd src-tauri && cargo test --lib agency::persist::tests::beat_refresh_flags_conflict_only_when_parties_in_increment_or_verbs -- --nocapture
```

Expected: compile error `cannot find function beat_refresh_flags`。

- [ ] **Step 3: 最小实现**

在 `persist.rs` 的 `touch_refresh_beats` 上方加入：

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct RefreshFlags {
    pub conflict: bool,
    pub cast: bool,
    pub location: bool,
    pub foreshadow: bool,
}

const CONFLICT_VERBS: &[&str] = &["对峙", "反转", "代价", "冲突", "对打", "逼迫", "加压"];

pub(crate) fn beat_refresh_flags(
    increment: &str,
    matched_names: &[String],
    prev_present: &[String],
    prev_location: Option<&str>,
    new_location: Option<&str>,
    conflict_parties: &[String],
    foreshadow_needles: &[String],
) -> RefreshFlags {
    let mut names: Vec<String> = matched_names.to_vec();
    names.sort();
    let mut prev: Vec<String> = prev_present.to_vec();
    prev.sort();
    let conflict_named = conflict_parties.len() >= 2
        && conflict_parties
            .iter()
            .all(|p| matched_names.iter().any(|n| n == p));
    let conflict_verb = CONFLICT_VERBS.iter().any(|v| increment.contains(v));
    RefreshFlags {
        conflict: conflict_named || conflict_verb,
        cast: names != prev,
        location: match (new_location.map(str::trim).filter(|s| !s.is_empty()), prev_location) {
            (Some(n), Some(p)) => n != p,
            (Some(_), None) => true,
            _ => false,
        },
        foreshadow: foreshadow_needles
            .iter()
            .any(|n| !n.is_empty() && increment.contains(n.as_str())),
    }
}
```

重写 `merge_progress_line`：

```rust
fn merge_progress_line(existing: Option<&str>, node: &str) -> String {
    let node = node.trim();
    let line = format!("进度：{node}");
    if node.is_empty() {
        return existing.unwrap_or("").to_string();
    }
    match existing.map(str::trim).filter(|s| !s.is_empty()) {
        None => line,
        Some(e) if e.contains(&line) => e.to_string(),
        Some(e) => {
            let merged = format!("{e}\n{line}");
            const CAP: usize = 2000;
            if merged.chars().count() <= CAP {
                merged
            } else {
                let mut kept = String::new();
                let mut progress: Vec<&str> = Vec::new();
                for raw in e.lines() {
                    if raw.trim_start().starts_with("进度：") {
                        progress.push(raw);
                    } else if kept.is_empty() {
                        kept.push_str(raw);
                    } else {
                        kept.push('\n');
                        kept.push_str(raw);
                    }
                }
                progress.push(&line);
                while progress.len() > 1 {
                    let candidate = if kept.is_empty() {
                        progress[1..].join("\n")
                    } else {
                        format!("{kept}\n{}", progress[1..].join("\n"))
                    };
                    if candidate.chars().count() <= CAP {
                        return candidate;
                    }
                    progress.remove(0);
                }
                if kept.is_empty() {
                    line
                } else {
                    format!("{kept}\n{line}")
                }
            }
        }
    }
}
```

把 `touch_refresh_beats` 签名扩为四旗标（加 `foreshadow: bool`），true 时写对应 `last_*_beat = append_beats`。

- [ ] **Step 4: 跑测试确认通过**

```
cd src-tauri && cargo test --lib agency::persist::tests -- --nocapture
```

Expected: PASS（含既有 Append 测试）。

- [ ] **Step 5: Commit** — 跳过（等用户说提交）。

---

### Task 2: persist 按旗标刷新债务 + 写回事实出场

**Files:**
- Modify: `src-tauri/src/agency/persist.rs`

- [ ] **Step 1: 写失败测试**

```rust
    #[test]
    fn two_appends_without_conflict_realization_leave_last_conflict_zero() {
        let pool = create_test_pool().unwrap();
        let (story_id, scene_id) = seed_story_with_scene(&pool);
        let card = dummy_card();
        persist_append_with_card(&pool, &scene_id, "旧文。", &long_increment(), &card).unwrap();
        persist_append_with_card(&pool, &scene_id, "旧文。", &long_increment(), &card).unwrap();
        let conn = pool.get().unwrap();
        let beats = crate::creative_engine::expansion::read_beat_counters(&conn, &story_id);
        assert_eq!(beats.append_beats, 2);
        assert_eq!(beats.last_conflict_beat, 0);
        let debt = crate::creative_engine::expansion::debt::ExpansionDebt::from_beats(&beats);
        assert!(debt
            .triggered()
            .contains(&crate::creative_engine::expansion::debt::QuotaItem::ConflictEscalation));
    }

    #[test]
    fn writeback_present_from_increment_names_not_card_plan() {
        let pool = create_test_pool().unwrap();
        let (_story_id, scene_id) = seed_story_with_scene(&pool);
        let mut card = dummy_card();
        card.cast = vec![
            CastMember { name: "阿岩".into(), purpose: "计划".into() },
            CastMember { name: "林雪".into(), purpose: "计划".into() },
            CastMember { name: "幽灵".into(), purpose: "沉寂回归".into() },
        ];
        let inc = format!("阿岩看了林雪一眼。{}", "续写增量正文。".repeat(28));
        persist_append_with_card(&pool, &scene_id, "旧文。", &inc, &card).unwrap();
        let scene = SceneRepository::new(pool.clone())
            .get_by_id(&scene_id)
            .unwrap()
            .unwrap();
        assert!(scene.characters_present.contains(&"阿岩".into()));
        assert!(scene.characters_present.contains(&"林雪".into()));
        assert!(!scene.characters_present.contains(&"幽灵".into()));
    }

    #[test]
    fn writeback_keeps_old_present_when_increment_has_no_names() {
        let pool = create_test_pool().unwrap();
        let (_story_id, scene_id) = seed_story_with_scene(&pool);
        {
            let repo = SceneRepository::new(pool.clone());
            repo.update(
                &scene_id,
                &crate::db::repositories::SceneUpdate {
                    characters_present: Some(vec!["阿岩".into()]),
                    ..Default::default()
                },
            )
            .unwrap();
        }
        persist_append_with_card(
            &pool,
            &scene_id,
            "旧文。",
            &long_increment(),
            &dummy_card(),
        )
        .unwrap();
        let scene = SceneRepository::new(pool.clone())
            .get_by_id(&scene_id)
            .unwrap()
            .unwrap();
        assert_eq!(scene.characters_present, vec!["阿岩".to_string()]);
    }
```

`dummy_card()` 抽现有 `append_writeback_sets_characters_present_names` 里的卡。该旧测试的增量必须改成点名「阿岩」「林雪」，否则与新契约冲突。

- [ ] **Step 2: 跑测试确认失败**

```
cd src-tauri && cargo test --lib agency::persist::tests::two_appends_without_conflict_realization_leave_last_conflict_zero -- --nocapture
```

Expected: `last_conflict_beat` 不是 0（今日每次写卡都刷新）。

- [ ] **Step 3: 改 `persist_append_inner`**

在写 `SceneUpdate` 之前：

1. 用 `CharacterRepository::get_by_story` 取名。
2. `matched` = `crate::agency::continue_assets::names_in_text(&table_names, increment)`（P1 再换成 `match_character_names`）。
3. `flags = beat_refresh_flags(...)`，`conflict_parties` 来自 card，`prev_present` / `prev_location` 来自已读 scene，`new_location` P0 用 `card.setting_location`。
4. `characters_present`：`matched` 非空则 `Some(matched)`，否则 `None`（让 repository 不覆盖）。确认 `SceneUpdate` 的 `None` 表示不改列——读 `SceneRepository::update`：若 `None` 跳过该列，则符合；若 `None` 写成空数组，则改为显式保留 `scene.characters_present`。
5. `character_conflicts`：仅 `flags.conflict` 时写 `card_conflicts(...)`。
6. `touch_refresh_beats(pool, story_id, flags.conflict, flags.cast, flags.location, flags.foreshadow)`。
7. `foreshadow_needles`：P0 可传空切片（伏笔针在 P2 状态网补）。债务缺口仍会因 `last_foreshadow_beat==0` 增长。

- [ ] **Step 4: 跑 persist 测试**

```
cd src-tauri && cargo test --lib agency::persist -- --nocapture
```

Expected: PASS。

- [ ] **Step 5: Commit** — 跳过。

---

### Task 3: 配额中文 + 下一节点不回绕 + 冲突看阵容

**Files:**
- Modify: `src-tauri/src/agency/beat_card.rs`

- [ ] **Step 1: 写失败测试**

```rust
    #[test]
    fn quota_text_for_beats_uses_pai_not_debug_enum() {
        let debt = crate::creative_engine::expansion::ExpansionDebt {
            conflict: 2,
            scene: 0,
            character: 0,
            foreshadow: 0,
        };
        let text = quota_text_for_beats(&debt).expect("quota");
        assert!(text.contains("本拍扩张任务"));
        assert!(text.contains("必须"));
        assert!(!text.contains("ConflictEscalation"));
        assert!(!text.contains("本章扩张任务"));
    }

    #[test]
    fn compile_next_node_does_not_rewind_to_first_sentence() {
        let pool = create_test_pool().unwrap();
        let sid = seed_story_minimal(&pool);
        StoryOutlineRepository::new(pool.clone())
            .create(&sid, "开篇灵堂。钟楼破阵。龙脉重封。", None, 3, None)
            .unwrap();
        let scenes = SceneRepository::new(pool.clone()).get_by_story(&sid).unwrap();
        if let Some(sc) = scenes.first() {
            SceneRepository::new(pool.clone())
                .update(
                    &sc.id,
                    &crate::db::repositories::SceneUpdate {
                        outline_content: Some(
                            "进度：开篇灵堂。\n进度：钟楼破阵。\n进度：龙脉重封。".into(),
                        ),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
        let node = compile_next_node(&pool, &sid);
        assert!(!node.starts_with("开篇灵堂"), "rewound: {node}");
        assert!(node.contains("把当前冲突推进一步") || node.contains("不得原地复述"));
    }

    #[test]
    fn compile_conflict_ignores_offstage_enemy() {
        let pool = create_test_pool().unwrap();
        let sid = seed_three_chars_one_silent(&pool);
        let chars = CharacterRepository::new(pool.clone())
            .get_by_story(&sid)
            .unwrap();
        let cast = vec![CastMember {
            name: "林雪".into(),
            purpose: "末段已在场，承接行动".into(),
        }];
        let mv = compile_conflict(&chars, &cast, &pool, &sid, "林雪");
        assert!(!mv.parties.iter().any(|p| p == "顾长夜"));
        assert!(!mv.action.contains("顾长夜"));
    }
```

`seed_story_minimal` 目前可能没有 scene。`compile_next_node` 测试若无 scene，先 `create_in_tx` 一场再写 outline。`compile_conflict` 今日是四参数，测试按新五参数（加 `cast`）写。`quota_text_for_beats` 尚未存在。

把 `compile_next_node` / `compile_conflict` 改为 `pub(crate)`。

- [ ] **Step 2: 跑测试确认失败**

```
cd src-tauri && cargo test --lib agency::beat_card::tests::compile_next_node_does_not_rewind_to_first_sentence -- --nocapture
```

Expected: 返回「开篇灵堂」或编译失败。

- [ ] **Step 3: 实现**

`quota_text_for_beats`：

```rust
pub fn quota_text_for_beats(
    debt: &crate::creative_engine::expansion::ExpansionDebt,
) -> Option<String> {
    debt.quota_text().map(|s| {
        s.replace("【本章扩张任务（硬性要求，必须落实）】", "【本拍扩张任务（硬性要求，必须落实）】")
            .replace(" 章——本章", " 拍——本拍")
            .replace(" 章无更新——本章", " 拍无更新——本拍")
            .replace(" 章无动静——本章", " 拍无动静——本拍")
    })
}
```

`SceneBeatCard` 加 `expansion_quota_text: Option<String>`。所有结构体字面量补 `expansion_quota_text: None` 或 `quota_text_for_beats(...)`。`render_full`：有 text 则追加该段，**删除** `format!("{:?}", i)`。

`compile_next_node`：候选 `chars().count() >= 8`；覆盖键 `take(20)`；全部覆盖返回 `"在硬约束内把当前冲突推进一步，不得原地复述末句。"`。删掉 `candidates.first()` 兜底。

`compile_conflict(..., cast: &[CastMember])`：`cast_names` 集合；敌对边仅当 src 名与 tgt 名都在集合内。否则逾期伏笔（`ForeshadowingTracker` 若在 beat_card 引入会跨层——不要新依赖：用已有 `ExpansionDebt` 不够。为免 `beat_card` 依赖 foreshadow 模块，P0 场内无边时用降级句，`parties` = cast 前 2 名。）

`compile_beat_card` 在 truncate 之后调 `compile_conflict(&chars, &cast, ...)`；`expansion_quota_text: quota_text_for_beats(&debt)`。先 `let debt = ExpansionDebt::compute(...)`，`triggered()` 与 text 同源。

- [ ] **Step 4: 跑 beat_card 测试**

```
cd src-tauri && cargo test --lib agency::beat_card -- --nocapture
```

Expected: PASS。修复所有 `SceneBeatCard {` 缺字段。

- [ ] **Step 5: Commit** — 跳过。

---

### Task 4: 第三次编译出现冲突配额（端到端 P0 契约）

**Files:**
- Modify: `src-tauri/src/agency/beat_card.rs` tests 或 `persist.rs` tests

- [ ] **Step 1: 写失败测试**

```rust
    #[test]
    fn third_compile_after_two_idle_appends_has_conflict_quota_zh() {
        let pool = create_test_pool().unwrap();
        let sid = seed_three_chars_one_silent(&pool);
        let scenes = SceneRepository::new(pool.clone()).get_by_story(&sid).unwrap();
        let scene_id = scenes.last().unwrap().id.clone();
        let card = compile_beat_card(&pool, &sid, "阿岩站在雨里。").unwrap();
        persist_append_with_card(&pool, &scene_id, "阿岩站在雨里。", &long_increment(), &card)
            .unwrap();
        persist_append_with_card(&pool, &scene_id, "阿岩站在雨里。", &long_increment(), &card)
            .unwrap();
        let card3 = compile_beat_card(&pool, &sid, "阿岩站在雨里。").unwrap();
        let text = card3.expansion_quota_text.clone().unwrap_or_default();
        let full = card3.render_full();
        assert!(
            text.contains("冲突") || full.contains("本拍扩张任务"),
            "quota missing: {full}"
        );
        assert!(!full.contains("ConflictEscalation"));
    }
```

`beat_card` 测试要 `use crate::agency::persist::persist_append_with_card`。若循环依赖，把测试放 `agency/tests.rs`。

- [ ] **Step 2–4:** 失败则查 Task 2 是否仍在「写卡就 refresh」。通过后跑：

```
cd src-tauri && cargo test --lib third_compile_after_two_idle_appends -- --nocapture
```

- [ ] **Step 5: Commit** — 跳过。

---

### Task 5: 续写过短回退走 continue 组装

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs`
- Test: `src-tauri/src/agency/tests.rs` 或 coordinator 可测纯函数

不要改 `writer_prose_fallback` 本体（创世仍用）。抽纯函数避免测 async：

- [ ] **Step 1: 写失败测试**

在 `coordinator.rs` 或 `tests.rs`：

```rust
    #[test]
    fn continue_short_retry_user_keeps_beat_card_not_genesis_premise() {
        let user = "【本章节拍任务】\n阵容：阿岩\n【续写硬锚点";
        let retry = crate::agency::coordinator::continue_short_retry_user(user);
        assert!(retry.contains("【本章节拍任务】"));
        assert!(retry.contains("只输出小说正文"));
        assert!(!retry.contains("故事前提："));
    }
```

若 `continue_short_retry_user` 不存在会编译失败。

- [ ] **Step 2: 确认失败**

```
cd src-tauri && cargo test --lib continue_short_retry_user_keeps_beat_card -- --nocapture
```

- [ ] **Step 3: 实现并改 `write_beat_once`**

```rust
pub(crate) fn continue_short_retry_user(user: &str) -> String {
    format!(
        "只输出小说正文，承接末句，落实节拍任务，禁止分析/提纲/创世开篇。\n\n{user}"
    )
}
```

`write_beat_once` 在 `text.chars().count() < 200` 时：

```rust
        let retry_user = continue_short_retry_user(&user);
        if let Ok(retry) = llm
            .complete(&system, &retry_user, TaskType::CreativeWriting, 8192)
            .await
        {
            let retry = crate::agents::orchestrator::sanitize_novel_output(retry.trim());
            if retry.chars().count() >= 200 {
                text = retry;
            }
        }
        if text.chars().count() >= 200 {
            // 走既有 board.write 成功臂
        }
        return Err(AppError::from(format!(
            "agency: write_beat_once 过短（{} 字符），续写回退仍失败 run={}",
            text.chars().count(),
            run_id
        )));
```

删除此分支对 `writer_prose_fallback` 的调用。`write_chapter` 里续写熔断同样改为 `continue_short_retry_user` + 同一 user（若该路径已有 assembled user）；没有则拼 `render_writer_user_prompt` 后再 retry。创世 `writer_prose_fallback` 调用点不动。

`continue_short_retry_user` 若在 `coordinator.rs` 的 `impl` 外，tests 模块用 `crate::agency::coordinator::continue_short_retry_user`。可能需 `pub(crate)` 且 tests 是子模块——`agency/tests.rs` 是 `mod tests` 在 agency 下，可 `use super::coordinator::continue_short_retry_user`。核对 `coordinator` 是否 `pub mod`：是。函数放在 `coordinator.rs` 文件层 `pub(crate) fn`。

- [ ] **Step 4:**

```
cd src-tauri && cargo test --lib continue_short_retry_user -- --nocapture
cd src-tauri && cargo test --lib agency:: -- --nocapture
```

Expected: PASS。创世 prose fallback 测试仍绿。

- [ ] **Step 5: Commit** — 跳过。

---

### Task 6: P0 回归闸门

- [ ] **Step 1: 跑全库**

```
cd src-tauri && cargo test --lib
python3 scripts/architecture_guard.py
```

Expected: 全绿。修复 `SceneBeatCard` 缺字段、`touch_refresh_beats` 参数个数、NextChapter 装配若也调用 `touch_refresh_beats` 则同样改用 `beat_refresh_flags`（搜 `touch_refresh_beats` 与 `scene_update_from_card`）。

NextChapter：`scene_fields_from_card` 今日无增量文本。对 NextChapter，用新章 `content` 当 increment 调 `beat_refresh_flags`。在 `handle_gate` / NextChapter 装配点搜 `scene_update_from_card` 并补旗标刷新。

- [ ] **Step 2: Commit** — 跳过。

P0 完成定义：设计 §10 契约 1–6 有测试且绿；幕前续写路径可运行。

---

## P1 — 身份与场次

### Task 7: 别名点名

**Files:**
- Modify: `src-tauri/src/agency/continue_assets.rs`

- [ ] **Step 1: 写失败测试**

```rust
    #[test]
    fn aliases_for_two_char_name_includes_a_prefix() {
        let a = aliases_for("沈砚");
        assert!(a.contains(&"沈砚".into()));
        assert!(a.contains(&"阿砚".into()));
        assert!(!a.iter().any(|s| s.chars().count() == 1));
    }

    #[test]
    fn match_character_names_alias_and_no_single_char() {
        let names = vec!["沈砚".into(), "白芷".into()];
        let hit = match_character_names(&names, "阿砚握着罗盘，白芷在侧。");
        assert!(hit.contains(&"沈砚".into()));
        assert!(hit.contains(&"白芷".into()));
        let miss = match_character_names(&names, "白雪落在石阶上。");
        assert!(!miss.contains(&"白芷".into()));
    }

    #[test]
    fn match_character_names_exact_name_wins_over_alias() {
        let names = vec!["沈砚".into(), "阿砚".into()];
        let hit = match_character_names(&names, "阿砚先开口。");
        assert_eq!(hit, vec!["阿砚".to_string()]);
    }
```

- [ ] **Step 2: 确认失败**

```
cd src-tauri && cargo test --lib agency::continue_assets::tests::match_character_names_alias_and_no_single_char -- --nocapture
```

- [ ] **Step 3: 实现**

```rust
pub fn aliases_for(name: &str) -> Vec<String> {
    let name = name.trim();
    if name.is_empty() {
        return vec![];
    }
    let chars: Vec<char> = name.chars().collect();
    let mut out = vec![name.to_string()];
    if let Some(last) = chars.last() {
        if chars.len() >= 2 {
            let nick = format!("阿{last}");
            if nick != name {
                out.push(nick);
            }
        }
    }
    if chars.len() >= 3 {
        let last2: String = chars[chars.len() - 2..].iter().collect();
        if last2 != name {
            out.push(last2);
        }
    }
    out.retain(|s| s.chars().count() >= 2);
    out.sort_by_key(|s| std::cmp::Reverse(s.chars().count()));
    out.dedup();
    out
}

pub fn match_character_names(names: &[impl AsRef<str>], text: &str) -> Vec<String> {
    let canonical: Vec<String> = names
        .iter()
        .map(|n| n.as_ref().trim().to_string())
        .filter(|n| !n.is_empty())
        .collect();
    let exact: std::collections::HashSet<&str> =
        canonical.iter().map(|s| s.as_str()).collect();
    let mut pairs: Vec<(usize, String, String)> = Vec::new();
    for canon in &canonical {
        for alias in aliases_for(canon) {
            if alias != *canon && exact.contains(alias.as_str()) {
                continue;
            }
            pairs.push((alias.chars().count(), alias, canon.clone()));
        }
    }
    pairs.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut hit = Vec::new();
    for (_, alias, canon) in pairs {
        if text.contains(&alias) && !hit.iter().any(|n| n == &canon) {
            hit.push(canon);
        }
    }
    hit
}
```

`present_in_text` 改用 `match_character_names`。`names_in_text` 保留给不需要别名的调用；`write_beat_once` 的 `names_in_text` 与 persist 的 matched 改为 `match_character_names`。

- [ ] **Step 4:**

```
cd src-tauri && cargo test --lib agency::continue_assets -- --nocapture
```

- [ ] **Step 5: Commit** — 跳过。

---

### Task 8: 沉寂/张力入场门闩 + 本场地点参数

**Files:**
- Modify: `src-tauri/src/agency/beat_card.rs`
- Modify: `src-tauri/src/agency/coordinator.rs`（`compile_beat_card` 调用传本场地点）

- [ ] **Step 1: 改现有测试并加新测试**

`beat_card_cast_includes_silent_character_when_three_exist`：正文只有阿岩与顾长夜时，**无 CharacterMove 则林雪可以不在 cast**。另测：把 `append_beats=3, last_cast_refresh_beat=0` 写入后编译，林雪在 cast 且 purpose 含「入场」。

`compile_beat_card` 签名增加 `current_scene_location: Option<&str>`。所有调用点补参数：Append 传当前 scene 的 `setting_location`；测试传 `None` 或 `"雨巷"`。

张力：仅当 `source` 或 `target` 已在 present 时加入另一端。

- [ ] **Step 2: 跑失败**

```
cd src-tauri && cargo test --lib agency::beat_card -- --nocapture
```

Expected: 旧测试「必含林雪」失败。

- [ ] **Step 3: 按设计 §6.2 / §6.3 改 `compile_beat_card`**

`setting_location` 用参数 `current_scene_location`，不再 `get_by_story` 倒序第一条非空。

`write_beat_once` 的 db 闭包：Append 时用 `SceneRepository::get_by_id` 取地点再 `compile_beat_card(..., loc.as_deref())`。需要把 `scene_id` 传入 `write_beat_once` 或从 `current_content` 无法得 id——`run_continue` 已有 persist scene_id，把它传入 `write_beat_once`。

查 `write_beat_once` 调用点（约 `run_continue` Append 臂），加 `scene_id: Option<&str>`。

- [ ] **Step 4:** beat_card + coordinator 相关测试 PASS。

- [ ] **Step 5: Commit** — 跳过。

---

### Task 9: 地点 shift

**Files:**
- Modify: `src-tauri/src/agency/continue_assets.rs`
- Modify: `src-tauri/src/agency/persist.rs`

- [ ] **Step 1: 测试**

```rust
    #[test]
    fn detect_location_shift_picks_last_known_place_in_increment() {
        let known = vec!["雨巷".into(), "钟楼".into()];
        let got = detect_location_shift(&known, Some("雨巷"), "他们离开雨巷，潜入钟楼底层。");
        assert_eq!(got.as_deref(), Some("钟楼"));
        assert!(detect_location_shift(&known, Some("雨巷"), "继续对话。").is_none());
    }
```

- [ ] **Step 2–3: 实现**

```rust
pub fn detect_location_shift(
    known: &[String],
    prev: Option<&str>,
    increment: &str,
) -> Option<String> {
    let mut last: Option<(usize, String)> = None;
    for loc in known {
        let loc = loc.trim();
        if loc.is_empty() {
            continue;
        }
        if let Some(idx) = increment.rfind(loc) {
            if last.as_ref().map(|(i, _)| idx >= *i).unwrap_or(true) {
                last = Some((idx, loc.to_string()));
            }
        }
    }
    let n = last.map(|(_, s)| s)?;
    match prev.map(str::trim).filter(|s| !s.is_empty()) {
        Some(p) if p == n => None,
        _ => Some(n),
    }
}
```

`persist_append_inner`：`known` = 本故事 scenes 地点 ∪ card.setting_location；`new_location = detect_location_shift(...).or(prev)`；写回仅当 shift `Some`。`beat_refresh_flags` 的 `new_location` 用 shift 结果。

- [ ] **Step 4:** continue_assets + persist 测试 PASS。

- [ ] **Step 5: Commit** — 跳过。

---

### Task 10: 活跃冲突 / 角色目标进 `render_continue_assets`

**Files:**
- Modify: `src-tauri/src/agency/continue_assets.rs`

- [ ] **Step 1: 测试**

在 `render_continue_assets` 测试夹具把 `character_goals: Some("阿岩：讨回公道".into())`，`admitted = ["阿岩"]`，断言输出含「讨回公道」。`active_conflicts: Some("皇权裂痕".into())` 断言含「皇权裂痕」。预算仍 ≤6000。

- [ ] **Step 2–3:** 在 `parts` 数组里，关系段之后插入过滤后的 conflicts/goals 段（`truncate_chars(..., 400)`）。goals 按行过滤含录取名。超预算仍 `apply_asset_budget`，红线/卡/名单 `true` 不可删光。

- [ ] **Step 4:**

```
cd src-tauri && cargo test --lib agency::continue_assets -- --nocapture
```

- [ ] **Step 5: Commit** — 跳过。

P1 完成定义：设计 §10 契约 7–10 绿。

---

## P2 — 拍级状态网

### Task 11: `BeatState` 编译与渲染

**Files:**
- Create: `src-tauri/src/agency/beat_state.rs`
- Modify: `src-tauri/src/agency/mod.rs`

- [ ] **Step 1: 测试（文件先写测试）**

```rust
//! 拍级状态网。设计：docs/plans/2026-08-15-continue-quality-closure-design.md §7
pub fn compile_beat_state(...) -> BeatState { unimplemented!() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beat_state_includes_next_node_and_overdue() {
        let st = compile_beat_state(
            &["沈砚".into(), "白芷".into()],
            Some("钟楼"),
            "子时前破金煞",
            &["五阵未破".into()],
            "沈砚握着罗盘。必须在子时前动手，否则龙脉裂口。",
            &["进度：灵堂托梦"],
        );
        assert!(st.present.contains(&"沈砚".into()));
        assert!(st.locations.iter().any(|(n, l)| n == "沈砚" && l == "钟楼"));
        assert!(st.threads.iter().any(|t| t.text.contains("金煞") || t.text.contains("子时")));
        assert!(st.threads.iter().any(|t| t.text.contains("五阵")));
        assert!(st.threads.len() <= 5);
        let full = st.render_full();
        assert!(full.contains("【本拍状态网】"));
        assert!(full.contains("未决"));
    }
}
```

- [ ] **Step 2: 确认失败**（unimplemented 或缺模块）。`mod.rs` 加 `pub mod beat_state;`。

- [ ] **Step 3: 实现**

```rust
#[derive(Debug, Clone)]
pub struct OpenThread {
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct BeatState {
    pub present: Vec<String>,
    pub locations: Vec<(String, String)>,
    pub threads: Vec<OpenThread>,
}

impl BeatState {
    pub fn render_full(&self) -> String {
        let loc = self
            .locations
            .iter()
            .map(|(n, l)| format!("{n}={l}"))
            .collect::<Vec<_>>()
            .join("；");
        let mut lines = vec![
            "【本拍状态网】".into(),
            format!("在场：{}", self.present.join("、")),
            format!("地点：{loc}"),
        ];
        if self.threads.is_empty() {
            lines.push("未决：承接当前冲突，不得原地复述末句。".into());
        } else {
            let t = self
                .threads
                .iter()
                .enumerate()
                .map(|(i, th)| format!("{}. {}", i + 1, th.text))
                .collect::<Vec<_>>()
                .join(" ");
            lines.push(format!("未决：{t}"));
        }
        lines.push(
            "必须承接未决，禁止忘掉已在场者，禁止把未决线程当已解决除非本拍写明解决。".into(),
        );
        lines.join("\n")
    }

    pub fn render_tail_summary(&self) -> String {
        let thread = self
            .threads
            .first()
            .map(|t| t.text.chars().take(40).collect::<String>())
            .unwrap_or_default();
        format!(
            "【状态摘要】在场：{}｜未决：{}",
            self.present.join("、"),
            thread
        )
    }
}

pub fn compile_beat_state(
    present: &[String],
    location: Option<&str>,
    next_node: &str,
    overdue: &[String],
    tail: &str,
    progress_lines: &[String],
) -> BeatState {
    let loc = location.unwrap_or("").trim();
    let locations = if loc.is_empty() {
        vec![]
    } else {
        present
            .iter()
            .map(|n| (n.clone(), loc.to_string()))
            .collect()
    };
    let mut threads: Vec<OpenThread> = Vec::new();
    let push = |threads: &mut Vec<OpenThread>, raw: &str| {
        let t: String = raw.chars().take(80).collect();
        let t = t.trim().to_string();
        if t.is_empty() {
            return;
        }
        if threads.iter().any(|x| x.text == t) {
            return;
        }
        if threads.len() < 5 {
            threads.push(OpenThread { text: t });
        }
    };
    push(&mut threads, next_node);
    for o in overdue {
        push(&mut threads, o);
    }
    for p in progress_lines {
        if threads.len() >= 5 {
            break;
        }
        if p.contains("进度：") {
            push(&mut threads, p.trim());
        }
    }
    const SIGNALS: &[&str] = &["必须", "之前", "否则", "子时", "七日"];
    for sent in tail.split(['。', '！', '？', '\n']) {
        if threads.len() >= 5 {
            break;
        }
        if SIGNALS.iter().any(|s| sent.contains(s)) {
            push(&mut threads, sent);
        }
    }
    BeatState {
        present: present.to_vec(),
        locations,
        threads,
    }
}
```

- [ ] **Step 4:**

```
cd src-tauri && cargo test --lib agency::beat_state -- --nocapture
```

- [ ] **Step 5: Commit** — 跳过。

---

### Task 12: prompt 插入状态网 + 末句降权

**Files:**
- Modify: `src-tauri/src/agency/beat_card.rs`
- Modify: `src-tauri/src/agency/coordinator.rs`（`render_writer_user_prompt` 调用）

- [ ] **Step 1:** 改 `ending_anchor` 测试：断言不含「最高优先级」；含「句法衔接」或「节拍任务与状态网」。

改 `render_writer_user_prompt` 签名增加 `state: Option<&BeatState>`。顺序：卡全文、状态网全文、bundle、指令、卡摘要、状态摘要、ending。

更新 `writer_prompt_order_is_card_then_body_then_summary_then_ending`。

- [ ] **Step 2–4:** 实现文案与顺序。所有 `render_writer_user_prompt` 调用补 `Some(&state)` 或 `None`（测试可 `None` 则跳过状态段）。`write_beat_once` 在 render 前 `compile_beat_state`。逾期伏笔从 `parts.bundle.overdue_foreshadowings` 来。

- [ ] **Step 5: Commit** — 跳过。

---

### Task 13: `probe_increment` + 一次重试

**Files:**
- Modify: `src-tauri/src/agency/beat_state.rs`
- Modify: `src-tauri/src/agency/coordinator.rs`

- [ ] **Step 1: 测试**

```rust
    #[test]
    fn probe_reports_missing_cast_and_unshifted_location() {
        let card = SceneBeatCard { /* 阿岩+林雪，配额 NewScene+ConflictEscalation */ ... };
        let state = BeatState {
            present: vec!["阿岩".into(), "林雪".into()],
            locations: vec![("阿岩".into(), "雨巷".into())],
            threads: vec![],
        };
        let probe = probe_increment(
            "他叹了口气，继续喝茶。",
            &card,
            &state,
            &[QuotaItem::NewScene, QuotaItem::ConflictEscalation],
        );
        assert!(!probe.gaps.is_empty());
        assert!(probe.gaps.join("").contains("在场") || probe.named_cast < 2);
    }
```

- [ ] **Step 3: 实现 `BeatProbe { named_cast, gaps }`**，规则见设计 §7.3。加压词复用 persist 的 `CONFLICT_VERBS`——不要跨模块复制三次：抽到 `continue_assets.rs` 的 `pub fn has_conflict_verb(text: &str) -> bool`，persist 与 probe 共用。

`write_beat_once` 在首次 complete ≥200 后跑 probe；`gaps` 非空则第二次 complete，user 追加：

```
【缺口（必须在正文里补上，不要解释）】
- ...
```

两次结果：`probe.gaps.len()` 更小者胜；平手更长者胜。仍有缺口 `log::warn`。过短路径不套探针重试（设计 §7.3）。

- [ ] **Step 4:** beat_state + `cargo test --lib agency::` PASS。

- [ ] **Step 5: Commit** — 跳过。

---

### Task 14: 回写 `character_states.location`

**Files:**
- Modify: `src-tauri/src/agency/persist.rs`
- Inspect: `src-tauri/src/memory/ingest.rs` `persist_character_states` 是否整行 REPLACE location

- [ ] **Step 1: 测试**

种子角色「阿岩」，persist 增量点名阿岩且 `setting_location`/`shift` 为「钟楼」。断言 `CharacterRepository::get_character_state` 的 `location == Some("钟楼")`。

- [ ] **Step 2–3:** persist 成功后对 matched names `update_character_state` 只设 location。读 ingest：若 `INSERT OR REPLACE` 覆盖 location，改为仅当旧 location 为空才写（设计 §7.4）。**改 ingest 前 impact `persist_character_states`。**

- [ ] **Step 4:** persist 测试 + ingest 既有测试 PASS。

- [ ] **Step 5: Commit** — 跳过。

P2 完成定义：设计 §10 契约 11–14 绿。

---

## P3 — 八拍验收

### Task 15: `eight_beat_append_quality_contract`

**Files:**
- Modify: `src-tauri/src/agency/tests.rs`

- [ ] **Step 1: 写测试**（可先用真实 `compile_beat_card` + `persist_append_with_card` 循环 8 次，不必起 coordinator LLM）：

```rust
#[test]
fn eight_beat_append_quality_contract() {
    let pool = create_test_pool().unwrap();
    // 3 角色 + 仇敌 + 大纲三句 + 1 scene「雨巷」
    // 8 次增量：0–1 只喝茶；2 阿岩对峙顾长夜；3–4 对话；5 进入钟楼（known 地点先写入另一 scene 或 known 列表）；6–7 点名林雪入场
    let mut content = "阿岩站在雨巷。".to_string();
    let scene_id = /* ... */;
    for i in 0..8 {
        let inc = increments[i]; // 每段 ≥200
        let card = compile_beat_card(&pool, &sid, &content).unwrap();
        if i == 2 {
            assert!(
                card.expansion_quota_text
                    .as_deref()
                    .unwrap_or("")
                    .contains("冲突")
                    || card.render_full().contains("本拍扩张任务"),
                "beat 3 must carry conflict quota"
            );
        }
        persist_append_with_card(&pool, &scene_id, &content, inc, &card).unwrap();
        content.push_str(inc);
    }
    let scenes = SceneRepository::new(pool.clone()).get_by_story(&sid).unwrap();
    assert_eq!(scenes.len(), 1);
    let present = &scenes[0].characters_present;
    assert!(present.iter().any(|n| n == "阿岩"));
    assert!(!present.iter().any(|n| n == "路人甲"));
}
```

`increments` 写成 8 个 ≥200 字的字符串常量。第 5 段含「钟楼」且故事里已有 known 地点「钟楼」（可在 seed 时给一场历史 scene 地点钟楼，或把钟楼写入第一场旧 `setting_location` 历史——`detect_location_shift` 的 known 来自全部 scenes。seed 第二场地点钟楼但 Append 只改第一场：已知表仍含钟楼）。

断言 `compile_next_node` 在进度累积后不回绕：读最后一次 card 的 `next_outline_node`。

P2 之后：对 `render_writer_user_prompt` 抽一次断言含「本拍状态网」、不含「最高优先级」。

- [ ] **Step 2–4:** 跑到 PASS。失败则回到对应阶段补旗标/地点 known。

```
cd src-tauri && cargo test --lib eight_beat_append_quality_contract -- --nocapture
cd src-tauri && cargo test --lib
python3 scripts/architecture_guard.py
```

- [ ] **Step 5: Commit** — 跳过。

### Task 16: 真机清单（文档，不写代码）

在 `docs/plans/2026-08-15-continue-quality-closure-design.md` §8.2 已有指标。实施结束时更新 `ROADMAP.md` 已知债务：保留「真机 8 次幕前续写未跑，不得宣称三症状已修复」；勾掉已由 mock 覆盖的「债务按拍无测试」。

**不要**在 README/CHANGELOG 写「角色丢失已修复」，除非用户跑完真机并授权。

发版推送时才 bump 版本并同步 docs of record（本计划不发版）。

---

## 规格覆盖自检

| 设计条款 | 任务 |
|---|---|
| §5.1 债务旗标 | Task 1–2, 4 |
| §5.2 配额中文 | Task 3 |
| §5.3 节点不回绕 | Task 3 |
| §5.4 冲突看阵容 | Task 3 |
| §5.5 事实写回 | Task 2 |
| §5.6 续写回退 | Task 5 |
| §6.1 别名 | Task 7 |
| §6.2 入场门闩 | Task 8 |
| §6.3 地点 | Task 8–9 |
| §6.4 冲突/目标段 | Task 10 |
| §7.1–7.2 状态网/末句 | Task 11–12 |
| §7.3 探针 | Task 13 |
| §7.4 位置回写 | Task 14 |
| §8.1 八拍 mock | Task 15 |
| §8.2 真机 | Task 16（文档） |
| §10 契约 1–15 | 各 Task 测试 |

无 TBD。类型名：`RefreshFlags`、`BeatState`、`OpenThread`、`BeatProbe`、`quota_text_for_beats`、`continue_short_retry_user`、`match_character_names`、`detect_location_shift`、`probe_increment` 前后一致。
