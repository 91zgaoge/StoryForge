# 智能创作资产融合深度重构 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 打通智能创作流程对已有创作资产（方法论/写作策略/提示词/追读力/冲突引擎）的融合运用，解决续写人物少、场景单一、世界构建简单、冲突不足、推进慢五大痛点。

**Architecture:** 一次性交付三块改造：①续写链路——Full 路径段落组装逻辑下沉为共享函数（`agents/writer_assets.rs`），TimeSliced 默认路径补齐活跃冲突/角色目标/追读力/体裁参考/风格混合注入，冲突约束语义化，prompt 改为阶段感知的扩张-收敛平衡导向；②计划结构——续写默认 `beat_planner → writer` 两步链（失败自动降级单 writer，`plan_mode` 可回退）；③资产贯通与创世——推荐资产透传写回、方法论 PromptRegistry 动态解析+step 自动推进、stories.strategy_json 持久化向导选择、向导提示词融合体裁画像/方法论/四元组、删除空转死提示词。

**Tech Stack:** Rust（Tauri 后端，`src-tauri/`）、React+TS 前端（`src-frontend/`）、SQLite 迁移（`src-tauri/src/db/migrations/`）、PromptRegistry（`resources/prompts/**/*.md`）。

**设计文档:** `docs/plans/2026-08-04-asset-fusion-deep-restructure-design.md`（已批准）

## Global Constraints

- 提交信息：中文 conventional commit（如 `feat: ...` / `fix: ...` / `chore: ...`），仓库有 pre-commit 格式检查钩子。
- Rust 测试命令：`cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib <filter>`；测试写在 lib 内 `#[cfg(test)] mod tests`。
- 前端测试：`npx vitest run <file>`（工作目录 `src-frontend/`）；类型检查 `npx tsc --noEmit`。
- 所有 prompt 改动走 PromptRegistry（`resources/prompts/**/*.md` + frontmatter），前端「提示词」面板可覆盖，禁止硬编码新 prompt 文本。
- 新注入段落一律「有资产才渲染、缺失记 debug/warn 跳过」，资产缺失不得导致生成失败。
- `WriteTimeBundle` 结构体字段定义在 `domain/write_time_bundle.rs`，`load_sync`/`to_prompt` 在 `creative_engine/write_time_bundle.rs`——新增字段必须改前者并在所有字面量构造点补字段。
- 数据库迁移自动按编号扫描执行，无需注册；最新编号 V118，本计划新增 V119。
- YAGNI：不改创世 2.0 agency 多代理流程；不恢复已 DROP 的 beat_cards/story_engines/pressure_relationships 表；不动模型网关与 LLM 适配层。

---

# 第一部分：续写链路资产贯通（Task 1-4）

# StoryMoss 资产融合重构实施计划 · 第 1 部分：续写链路资产贯通（Task 1-4）

> 设计依据：`docs/plans/2026-08-04-asset-fusion-deep-restructure-design.md` 第一节「续写链路资产贯通」。
> 本部分只覆盖设计第一节；第二~五节由其他部分承接。
> 所有代码片段均基于 2026-08-05 读到的真实当前源码改编，行号锚点为当前工作区行号。
> 仓库约定：Rust 测试在被测文件内 `#[cfg(test)] mod tests`；提交信息用中文 conventional commit。
> 测试命令统一为 `cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib <filter>`。

## 读源码确认的关键事实（全部分共享）

- `WriteTimeBundle` 结构体定义在 `src-tauri/src/domain/write_time_bundle.rs:13-64`；`load_sync` 与 `to_prompt()` 在 `src-tauri/src/creative_engine/write_time_bundle.rs`（`impl WriteTimeBundle`，35 行起）。结构体字面量共 8 处需要随新字段同步更新：`creative_engine/write_time_bundle.rs:433`（生产）、同文件 1058/1098/1139/1219（测试）、`agents/orchestrator.rs:4953`（测试 helper `progression_bundle`）、`creative_engine/prompt_synthesis/mod.rs:48`（测试）、`creative_engine/prompt_synthesis/manifest.rs:374`（测试）。
- 快照数据源：`crate::canonical_state::CanonicalStateManager::new(pool.clone()).get_snapshot_sync(story_id)`（`canonical_state/manager.rs:36`），返回 `CanonicalStateSnapshot`（`canonical_state/mod.rs:13`），含 `story_context.active_conflicts: Vec<Conflict>`（`conflict_type/parties/stakes`，mod.rs:64-68）与 `character_states: Vec<CharacterStateSnapshot>`（`name/current_location/current_emotion/active_goal/secrets_known/secrets_unknown/arc_progress: f32`，mod.rs:34-43）。
- `WritingStrategy`（`config/settings.rs:164-170`）：`run_mode: String`、`conflict_level: i32`（0-100）、`pace: String`、`ai_freedom: String`；已实现 `Default`（settings.rs:176）。代码库中**不存在** `conflict_intensity`/`pacing_factor` 概念，Task 1 共享函数的 f64 参数语义为新增约定：conflict_intensity 沿用 0-100 刻度与 service.rs 既有 80/60/40/20 阈值；pacing_factor 由 `pace` 字符串映射（fast=1.5 / slow=0.5 / 其他=1.0），≥1.2 判快、≤0.8 判慢。
- 模板：`writer_chase_debt`（vars: debt_count, debts）与 `writer_reading_power_goal`（vars: hook_type, hook_strength, foreshadowing_list, micropayoff_count）均为 Registry 内置提示词（`prompts/registry.rs:775-776`），解析走 `crate::prompts::registry::resolve_prompt(pool, id)`（用户覆盖优先，433-444）回退 `resolve_prompt_default(id)`（447-451），渲染用 `crate::prompts::engine::TemplateEngine::render_with_conditions`。
- `AgentTask.parameters: HashMap<String, serde_json::Value>`（`domain/agent_types.rs:59`）。
- `PlanContext.style_dna_info: Option<String>`（`planner/mod.rs:88`）已由 `commands/orchestrator.rs:612-651` 填好：blend 时为 `"风格混合 [name]: comp1:70%, comp2:30%"`，无 blend 时回退 `"风格DNA ID: xxx"`；commands 层在 690/763 行已把它塞进 PlanContext，**无需改 commands 层**。
- `build_progression_anchor`（`agents/orchestrator.rs:3765-3852`）有两个调用方：TimeSliced（1046-1052，此处 bundle.to_prompt() 已进入 prompt，属重复注入）与 TriShot（1662-1668，仅 `!synthesis.is_fallback` 时调用，此时 synthesized_prompt **不含** bundle 段落，不是重复）。因此去重必须按调用方区分，不能一刀切删除段落，否则 TriShot 的大纲/世界观将彻底丢失。
- 测试基建：`crate::db::create_test_pool()`（`db/connection.rs:15`，内存 SQLite + 全量迁移）；种子模式参考 `canonical_state/tests.rs`（`StoryRepository::create(CreateStoryRequest{...})`、`CharacterRepository::create(CreateCharacterRequest{...})`、`SceneRepository::create(&story_id, seq, Some(title))` + `SceneRepository::update(&id, &SceneUpdate{ ..Default::default() })`）；活跃冲突可通过 `SceneUpdate.character_conflicts: Option<Vec<CharacterConflict>>`（`db/models.rs:160-165`，字段 `character_a_id/character_b_id/conflict_nature/stakes`）注入，快照聚合逻辑见 `canonical_state/manager.rs:310-346`。
- `agents/mod.rs` 模块注册在 16-29 行（`pub(crate) mod trim_utils;` 之后追加）。

---

## Task 1：共享段落组装函数下沉（writer_assets.rs）

**Files:**
- Create: `src-tauri/src/agents/writer_assets.rs`（新文件，含实现 + `#[cfg(test)] mod tests`）
- Modify: `src-tauri/src/agents/mod.rs:29`（注册模块）
- Modify: `src-tauri/src/agents/service.rs:1879-1935`（策略约束块改调共享函数）、`1966-1971`（体裁参考表改调共享函数）、`2336-2412`（活跃冲突/角色状态块改调共享函数）
- Test: 同 Create 文件内 tests

**Interfaces:**
- Produces（`agents/writer_assets.rs`，均 `pub(crate)`）：
  - `pub(crate) fn format_active_conflicts(pool: &crate::db::DbPool, story_id: &str, budget_chars: usize) -> Option<String>`
  - `pub(crate) fn format_character_goals(pool: &crate::db::DbPool, story_id: &str, per_char_budget: usize) -> Option<String>`
  - `pub(crate) fn format_genre_reference_tables(profile: &crate::db::GenreProfile, budget_chars: usize) -> Option<String>`
  - `pub(crate) fn writing_constraints_semantic_text(conflict_intensity: f64, pacing_factor: f64) -> String`
  - `pub(crate) fn pace_to_factor(pace: &str) -> f64`（辅助：fast=1.5 / slow=0.5 / 其他=1.0）
- Consumes：`crate::canonical_state::CanonicalStateManager::get_snapshot_sync`、`crate::db::GenreProfile`、`crate::config::settings::WritingStrategy`（调用方）。
- 语义约定：`budget_chars` 传 `usize::MAX` 等价不截断（service.rs 原行为无截断，传 MAX 保证行为不变）；`format_*` 无资产或快照加载失败返回 `None`（记 warn/debug，不 panic）。

- [ ] **Step 1: 在 agents/mod.rs 注册模块并创建只含失败测试的 writer_assets.rs**

`src-tauri/src/agents/mod.rs` 第 29 行 `pub(crate) mod trim_utils;` 后追加一行：

```rust
pub(crate) mod writer_assets;
```

新建 `src-tauri/src/agents/writer_assets.rs`，先只写测试（实现留空，验证 TDD 红灯）：

```rust
//! Writer 资产段落组装共享函数
//!
//! 从 Full 路径（`agents/service.rs` 的 `build_writer_prompt`）下沉的段落组装
//! 逻辑，供 Full 路径与 TimeSliced（WriteTimeBundle）双路复用。
//! 全部为同步函数：内部只做本地 SQLite 聚合，异步调用方须用
//! `tokio::task::spawn_blocking` 包裹。

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        connection::create_test_pool,
        repositories::{CharacterRepository, SceneRepository, SceneUpdate, StoryRepository},
        CharacterConflict, CreateCharacterRequest, CreateStoryRequest,
    };

    fn block_on<F>(f: F) -> F::Output
    where
        F: std::future::Future,
    {
        tokio::runtime::Runtime::new().unwrap().block_on(f)
    }

    fn seed_story(pool: &crate::db::DbPool) -> crate::db::Story {
        StoryRepository::new(pool.clone())
            .create(CreateStoryRequest {
                title: "测试故事".to_string(),
                description: None,
                genre: Some("奇幻".to_string()),
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: None,
                reference_book_id: None,
            })
            .unwrap()
    }

    // ---- format_active_conflicts ----

    #[test]
    fn format_active_conflicts_with_conflicts() {
        let pool = create_test_pool().unwrap();
        let story = seed_story(&pool);
        let scene_repo = SceneRepository::new(pool.clone());
        let scene = scene_repo.create(&story.id, 1, Some("第一章")).unwrap();
        scene_repo
            .update(
                &scene.id,
                &SceneUpdate {
                    character_conflicts: Some(vec![CharacterConflict {
                        character_a_id: "张三".to_string(),
                        character_b_id: "李四".to_string(),
                        conflict_nature: "杀父之仇".to_string(),
                        stakes: "家族存亡".to_string(),
                    }]),
                    ..Default::default()
                },
            )
            .unwrap();
        let text = format_active_conflicts(&pool, &story.id, 600).expect("有资产应输出段落");
        assert!(text.contains("【当前活跃冲突】"));
        assert!(text.contains("杀父之仇"));
        assert!(text.contains("家族存亡"));
        assert!(text.contains("张三"));
    }

    #[test]
    fn format_active_conflicts_empty_returns_none() {
        let pool = create_test_pool().unwrap();
        let story = seed_story(&pool);
        assert!(format_active_conflicts(&pool, &story.id, 600).is_none());
        // 故事不存在（快照聚合失败）同样返回 None，不 panic
        assert!(format_active_conflicts(&pool, "no-such-story", 600).is_none());
    }

    // ---- format_character_goals ----

    #[test]
    fn format_character_goals_with_states() {
        let pool = create_test_pool().unwrap();
        let story = seed_story(&pool);
        let char_repo = CharacterRepository::new(pool.clone());
        let character = char_repo
            .create(CreateCharacterRequest {
                story_id: story.id.clone(),
                name: "张三".to_string(),
                background: Some("主角".to_string()),
                personality: None,
                goals: None,
                appearance: None,
                gender: None,
                age: None,
                source: None,
                is_auto_generated: None,
            })
            .unwrap();
        let manager = crate::canonical_state::CanonicalStateManager::new(pool.clone());
        block_on(manager.update_character_state(
            &story.id,
            &character.id,
            crate::canonical_state::CharacterStateSnapshot {
                character_id: character.id.clone(),
                name: "张三".to_string(),
                current_location: Some("京城".to_string()),
                current_emotion: None,
                active_goal: Some("复仇".to_string()),
                secrets_known: vec![],
                secrets_unknown: vec!["身世之谜".to_string()],
                arc_progress: 0.5,
            },
        ))
        .unwrap();
        let text = format_character_goals(&pool, &story.id, 200).expect("有资产应输出段落");
        assert!(text.contains("【角色当前状态】"));
        assert!(text.contains("目标: 复仇"));
        assert!(text.contains("未知秘密: 身世之谜"));
        assert!(text.contains("弧光进度: 50%"));
    }

    #[test]
    fn format_character_goals_empty_returns_none() {
        let pool = create_test_pool().unwrap();
        let story = seed_story(&pool);
        // 故事无角色 → character_states 为空 → None
        assert!(format_character_goals(&pool, &story.id, 200).is_none());
    }

    // ---- format_genre_reference_tables ----

    fn test_profile(
        reference_tables: Option<String>,
        typical_structure: Option<String>,
    ) -> crate::db::GenreProfile {
        crate::db::GenreProfile {
            id: "gp1".to_string(),
            genre_name: "玄幻".to_string(),
            canonical_name: "Fantasy".to_string(),
            aliases_json: None,
            core_tone: None,
            pacing_strategy: None,
            anti_patterns_json: None,
            reference_tables_json: reference_tables,
            typical_structure_json: typical_structure,
            reader_promise: None,
            recommended_style_dna_ids: None,
            recommended_methodology_id: None,
            recommended_skill_ids: None,
            min_quality_tier: None,
            is_builtin: true,
            created_at: chrono::Local::now(),
        }
    }

    #[test]
    fn format_genre_reference_tables_with_assets() {
        let profile = test_profile(
            Some("境界体系：炼气-筑基-金丹".to_string()),
            Some("起承转合四幕".to_string()),
        );
        let text = format_genre_reference_tables(&profile, 800).expect("有资产应输出段落");
        assert!(text.contains("元素参考表："));
        assert!(text.contains("炼气-筑基-金丹"));
        assert!(text.contains("典型结构参考："));
        assert!(text.contains("起承转合四幕"));
    }

    #[test]
    fn format_genre_reference_tables_empty_returns_none() {
        let profile = test_profile(None, None);
        assert!(format_genre_reference_tables(&profile, 800).is_none());
        let blank = test_profile(Some("  ".to_string()), None);
        assert!(format_genre_reference_tables(&blank, 800).is_none());
    }

    // ---- writing_constraints_semantic_text ----

    #[test]
    fn writing_constraints_semantic_conflict_tiers() {
        assert!(writing_constraints_semantic_text(90.0, 1.0).contains("冲突强度：极高"));
        assert!(writing_constraints_semantic_text(70.0, 1.0).contains("冲突强度：高。"));
        assert!(writing_constraints_semantic_text(50.0, 1.0).contains("冲突强度：中等"));
        assert!(writing_constraints_semantic_text(30.0, 1.0).contains("冲突强度：低。"));
        assert!(writing_constraints_semantic_text(10.0, 1.0).contains("冲突强度：极低"));
    }

    #[test]
    fn writing_constraints_semantic_pacing_tiers() {
        assert!(writing_constraints_semantic_text(50.0, 1.5).contains("叙事节奏：快"));
        assert!(writing_constraints_semantic_text(50.0, 0.5).contains("叙事节奏：慢"));
        assert!(writing_constraints_semantic_text(50.0, 1.0).contains("叙事节奏：均衡"));
    }

    #[test]
    fn pace_to_factor_mapping() {
        assert_eq!(pace_to_factor("fast"), 1.5);
        assert_eq!(pace_to_factor("slow"), 0.5);
        assert_eq!(pace_to_factor("normal"), 1.0);
    }
}
```

- [ ] **Step 2: 跑测试确认失败（编译错误即红灯）**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib writer_assets
```

预期输出（编译失败，函数尚未实现）：

```text
error[E0425]: cannot find function `format_active_conflicts` in module `super`
  --> src/agents/writer_assets.rs:...
error[E0425]: cannot find function `format_character_goals` in module `super`
error[E0425]: cannot find function `format_genre_reference_tables` in module `super`
error[E0425]: cannot find function `writing_constraints_semantic_text` in module `super`
error[E0425]: cannot find function `pace_to_factor` in module `super`
error: aborting due to previous errors
```

- [ ] **Step 3: 实现 5 个共享函数（写入 writer_assets.rs 顶部、测试模块之前）**

```rust
/// 按字符数截断；`usize::MAX` 等价不截断（保持 Full 路径原行为）。
fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}…（已截断）", truncated)
    }
}

/// 当前活跃冲突段落（原 service.rs:2345-2357 内联逻辑下沉）。
/// 无活跃冲突或快照加载失败返回 None。
pub(crate) fn format_active_conflicts(
    pool: &crate::db::DbPool,
    story_id: &str,
    budget_chars: usize,
) -> Option<String> {
    let snapshot = crate::canonical_state::CanonicalStateManager::new(pool.clone())
        .get_snapshot_sync(story_id)
        .map_err(|e| {
            log::warn!("[writer_assets] 快照加载失败(active_conflicts, {}): {}", story_id, e)
        })
        .ok()?;
    let conflicts = &snapshot.story_context.active_conflicts;
    if conflicts.is_empty() {
        return None;
    }
    let mut lines = vec!["【当前活跃冲突】".to_string()];
    for conflict in conflicts {
        lines.push(format!(
            "- {}: 涉及 {}, 赌注: {}",
            conflict.conflict_type,
            conflict.parties.join(", "),
            conflict.stakes
        ));
    }
    Some(truncate_chars(&lines.join("\n"), budget_chars))
}

/// 角色当前状态（目标/弧光/秘密）段落（原 service.rs:2386-2411 内联逻辑下沉）。
/// 每个角色行按 per_char_budget 截断；无角色状态返回 None。
pub(crate) fn format_character_goals(
    pool: &crate::db::DbPool,
    story_id: &str,
    per_char_budget: usize,
) -> Option<String> {
    let snapshot = crate::canonical_state::CanonicalStateManager::new(pool.clone())
        .get_snapshot_sync(story_id)
        .map_err(|e| {
            log::warn!("[writer_assets] 快照加载失败(character_goals, {}): {}", story_id, e)
        })
        .ok()?;
    if snapshot.character_states.is_empty() {
        return None;
    }
    let mut lines = vec!["【角色当前状态】".to_string()];
    for cs in &snapshot.character_states {
        let mut parts = vec![format!("{}:", cs.name)];
        if let Some(ref loc) = cs.current_location {
            parts.push(format!("位置: {}", loc));
        }
        if let Some(ref emo) = cs.current_emotion {
            parts.push(format!("情绪: {}", emo));
        }
        if let Some(ref goal) = cs.active_goal {
            parts.push(format!("目标: {}", goal));
        }
        if !cs.secrets_known.is_empty() {
            parts.push(format!("已知秘密: {}", cs.secrets_known.join(", ")));
        }
        if !cs.secrets_unknown.is_empty() {
            parts.push(format!("未知秘密: {}", cs.secrets_unknown.join(", ")));
        }
        parts.push(format!("弧光进度: {:.0}%", cs.arc_progress * 100.0));
        lines.push(format!("- {}", truncate_chars(&parts.join(" "), per_char_budget)));
    }
    Some(lines.join("\n"))
}

/// 体裁元素参考表 + 典型结构段落（原 service.rs:1966-1971 内联逻辑下沉）。
/// 两者皆空返回 None。
pub(crate) fn format_genre_reference_tables(
    profile: &crate::db::GenreProfile,
    budget_chars: usize,
) -> Option<String> {
    let mut lines = Vec::new();
    if let Some(reference_tables) = &profile.reference_tables_json {
        if !reference_tables.trim().is_empty() {
            lines.push(format!("元素参考表：\n{}", reference_tables));
        }
    }
    if let Some(typical_structure) = &profile.typical_structure_json {
        if !typical_structure.trim().is_empty() {
            lines.push(format!("典型结构参考：\n{}", typical_structure));
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(truncate_chars(&lines.join("\n"), budget_chars))
    }
}

/// `WritingStrategy.pace` 字符串 → 节奏因子（fast=1.5 / slow=0.5 / 其他=1.0）。
pub(crate) fn pace_to_factor(pace: &str) -> f64 {
    match pace {
        "fast" => 1.5,
        "slow" => 0.5,
        _ => 1.0,
    }
}

/// 冲突强度 + 叙事节奏的分档语义文案
/// （原 service.rs:1888-1899 冲突五档 + 1901-1908 节奏三档下沉）。
///
/// `conflict_intensity` 沿用 `WritingStrategy.conflict_level` 的 0-100 刻度；
/// `pacing_factor` 由 `pace_to_factor` 映射，>=1.2 判快、<=0.8 判慢。
pub(crate) fn writing_constraints_semantic_text(
    conflict_intensity: f64,
    pacing_factor: f64,
) -> String {
    let conflict_line = if conflict_intensity >= 80.0 {
        "冲突强度：极高。每 500 字至少设置一次冲突或张力，保持高度紧张感。"
    } else if conflict_intensity >= 60.0 {
        "冲突强度：高。保持频繁的冲突和对抗，推动情节快速展开。"
    } else if conflict_intensity >= 40.0 {
        "冲突强度：中等。适度安排冲突，兼顾人物发展和情节推进。"
    } else if conflict_intensity >= 20.0 {
        "冲突强度：低。以人物内心和情感为主，减少外部冲突。"
    } else {
        "冲突强度：极低。以平和、抒情、描写为主，避免剧烈冲突。"
    };
    let pacing_line = if pacing_factor >= 1.2 {
        "叙事节奏：快。减少环境描写和冗余叙述，增加动作和对话，快速推进情节。"
    } else if pacing_factor <= 0.8 {
        "叙事节奏：慢。允许细腻的环境描写和心理刻画，注重氛围营造。"
    } else {
        "叙事节奏：均衡。动作与描写交替，保持适度的推进速度。"
    };
    format!("{}\n{}", conflict_line, pacing_line)
}
```

- [ ] **Step 4: service.rs 三处改为调用共享函数（行为不变，预算均传 usize::MAX）**

**4a. `service.rs:1879-1908` 策略约束块**：`strategy_lines` 改为 `Vec<String>`，冲突/节奏两组 if-else 链替换为共享函数调用。整块替换为：

```rust
        if let Some(ref ws) = strategy {
            let mut strategy_lines: Vec<String> = Vec::new();

            if ws.run_mode == "fast" {
                strategy_lines.push("运行模式：快速生成。允许较快的叙事推进，注重效率。".to_string());
            } else if ws.run_mode == "polish" {
                strategy_lines.push(
                    "运行模式：精修生成。注重文字质量，每句都需斟酌，允许较慢的推进速度。"
                        .to_string(),
                );
            }

            // 冲突强度 + 叙事节奏分档文案下沉为共享函数（TimeSliced 双路复用）
            strategy_lines.push(crate::agents::writer_assets::writing_constraints_semantic_text(
                ws.conflict_level as f64,
                crate::agents::writer_assets::pace_to_factor(&ws.pace),
            ));

            if ws.ai_freedom == "low" {
                strategy_lines.push(
                    "AI 自由度：低。严格遵循已有设定和大纲，不得偏离世界观或人物设定，\
                     不得擅自引入新元素。"
                        .to_string(),
                );
            } else if ws.ai_freedom == "high" {
                strategy_lines.push(
                    "AI 自由度：高。在保持整体方向一致的前提下，允许创新情节发展和意外转折。"
                        .to_string(),
                );
            } else {
                strategy_lines
                    .push("AI 自由度：中。遵循核心设定，但在细节和情节展开上有一定发挥空间。".to_string());
            }

            if !strategy_lines.is_empty() {
                let mut section = "【写作策略约束】\n".to_string();
                for line in strategy_lines {
                    section.push_str(&line);
                    section.push('\n');
                }
                system_chunks.push(ContextChunk::new(
                    section,
                    ContextPriority::High,
                    "writing_strategy",
                ));
            }
        }
```

（与原 1878-1936 行逐字等价，仅冲突/节奏两段改为共享函数，文案不变。）

**4b. `service.rs:1966-1971` 体裁参考表两行**替换为：

```rust
                if let Some(tables) = crate::agents::writer_assets::format_genre_reference_tables(
                    &profile,
                    usize::MAX,
                ) {
                    lines.push(tables);
                }
```

（共享函数内部已做空串过滤与 `\n` 连接，输出与原两行 push 等价。）

**4c. `service.rs:2336-2412` 快照段落**：在 `emit_and_yield("正在读取故事与场景数据...", 0.187); tokio::task::yield_now().await;`（2333-2334）之后、`let mut snapshot_parts = Vec::new();`（2336）之前插入：

```rust
        // 活跃冲突与角色状态段落复用共享函数（TimeSliced bundle 双路复用）；
        // usize::MAX = 不截断，与原内联行为一致。同步 SQLite 聚合移入 spawn_blocking。
        let pool_for_assets = self.pool.clone();
        let story_id_for_assets = ctx.story.story_id.clone();
        let (active_conflicts_text, character_goals_text) = tokio::task::spawn_blocking(move || {
            (
                crate::agents::writer_assets::format_active_conflicts(
                    &pool_for_assets,
                    &story_id_for_assets,
                    usize::MAX,
                ),
                crate::agents::writer_assets::format_character_goals(
                    &pool_for_assets,
                    &story_id_for_assets,
                    usize::MAX,
                ),
            )
        })
        .await
        .unwrap_or((None, None));
```

原 2345-2358 活跃冲突块替换为：

```rust
            if let Some(ref text) = active_conflicts_text {
                emit_and_yield("正在注入活跃冲突信息...", 0.189);
                snapshot_parts.push(text.clone());
                tokio::task::yield_now().await;
            }
```

原 2386-2411 角色状态块替换为：

```rust
            if let Some(ref text) = character_goals_text {
                emit_and_yield("正在注入角色当前状态...", 0.192);
                snapshot_parts.push(text.clone());
                tokio::task::yield_now().await;
            }
```

（push 顺序与原一致：叙事阶段 → 活跃冲突 → 待回收伏笔 → 逾期伏笔 → 角色状态；`snapshot` 变量仍用于其余段落。注意：共享函数会各自再做一次快照聚合，本地 SQLite 开销可接受，性能优化留待后续。）

- [ ] **Step 5: 跑测试确认通过 + 回归**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib writer_assets
```

预期输出：

```text
running 10 tests
test agents::writer_assets::tests::format_active_conflicts_empty_returns_none ... ok
test agents::writer_assets::tests::format_active_conflicts_with_conflicts ... ok
test agents::writer_assets::tests::format_character_goals_empty_returns_none ... ok
test agents::writer_assets::tests::format_character_goals_with_states ... ok
test agents::writer_assets::tests::format_genre_reference_tables_empty_returns_none ... ok
test agents::writer_assets::tests::format_genre_reference_tables_with_assets ... ok
test agents::writer_assets::tests::pace_to_factor_mapping ... ok
test agents::writer_assets::tests::writing_constraints_semantic_conflict_tiers ... ok
test agents::writer_assets::tests::writing_constraints_semantic_pacing_tiers ... ok
test result: ok. 10 passed
```

再跑 service 相关回归：`cargo test --lib agents::service`（应全绿；service.rs 改动为等价替换）。

- [ ] **Step 6: 提交**

```bash
cd /Users/yuzaimu/projects/StoryForge && git add src-tauri/src/agents/writer_assets.rs src-tauri/src/agents/mod.rs src-tauri/src/agents/service.rs && git commit -m "feat: 下沉 Full 路径段落组装逻辑为 writer_assets 共享函数供双路复用"
```

---

## Task 2：WriteTimeBundle 新字段与 to_prompt 段落

**Files:**
- Modify: `src-tauri/src/domain/write_time_bundle.rs:55-64`（结构体新增 5 字段）
- Modify: `src-tauri/src/creative_engine/write_time_bundle.rs:252-281`（genre_profile_strategy 块重构 + genre_reference 加载）、`433-457`（load_sync 返回字面量）、`582-803`（to_prompt 新增段落）、`1058/1098/1139/1219`（测试字面量补字段）
- Modify: `src-tauri/src/agents/orchestrator.rs:4953-4991`（`progression_bundle` 测试 helper 补字段）
- Modify: `src-tauri/src/creative_engine/prompt_synthesis/mod.rs:48`、`src-tauri/src/creative_engine/prompt_synthesis/manifest.rs:374`（测试字面量补字段）
- Test: `src-tauri/src/creative_engine/write_time_bundle.rs` 内 `#[cfg(test)] mod tests`

**Interfaces:**
- Consumes：Task 1 的 `format_active_conflicts` / `format_character_goals` / `format_genre_reference_tables`。
- Produces（`WriteTimeBundle` 新字段，均 `Option<String>`）：`active_conflicts`、`character_goals`、`chase_debt_text`、`genre_reference`、`style_blend_text`。其中 `chase_debt_text` / `style_blend_text` 由调用方（orchestrator）从 `task.parameters` 设置（Task 3 接通），`load_sync` 恒置 `None`。
- 渲染契约：`to_prompt()` 有值才输出；`style_blend_text` 存在时优先渲染 blend 段并**跳过** `style_dna_extension` 单 DNA 段（回退单 DNA）。

- [ ] **Step 1: 写失败测试（追加到 write_time_bundle.rs 的 tests 模块末尾）**

注意：测试引用的 5 个字段尚不存在，且 `bundle_with_outline`（1218 行）等 4 个测试构造器将在 Step 3 补字段；本步只加新测试，编译即红灯。

```rust
    // ---- 设计第一节：续写链路资产贯通，新段落注入 ----

    #[test]
    fn to_prompt_new_asset_sections_rendered() {
        let mut bundle = bundle_with_outline(None, None);
        bundle.active_conflicts =
            Some("【当前活跃冲突】\n- 角色冲突: 涉及 张三, 李四, 赌注: 家族存亡".to_string());
        bundle.character_goals = Some("【角色当前状态】\n- 张三: 目标: 复仇".to_string());
        bundle.chase_debt_text =
            Some("【追读力债务】\n当前有 1 条待偿还的追读力债务，需在后续章节中兑现：".to_string());
        bundle.genre_reference = Some("元素参考表：\n境界体系表".to_string());
        let prompt = bundle.to_prompt();
        assert!(prompt.contains("【当前活跃冲突】"));
        assert!(prompt.contains("家族存亡"));
        assert!(prompt.contains("【角色当前状态】"));
        assert!(prompt.contains("目标: 复仇"));
        assert!(prompt.contains("【追读力债务】"));
        assert!(prompt.contains("【体裁元素参考】"));
        assert!(prompt.contains("境界体系表"));
    }

    #[test]
    fn to_prompt_new_asset_sections_skipped_when_none() {
        let bundle = bundle_with_outline(None, None);
        let prompt = bundle.to_prompt();
        assert!(!prompt.contains("【当前活跃冲突】"));
        assert!(!prompt.contains("【角色当前状态】"));
        assert!(!prompt.contains("【追读力债务】"));
        assert!(!prompt.contains("【体裁元素参考】"));
        assert!(!prompt.contains("【风格混合"));
    }

    #[test]
    fn to_prompt_style_blend_takes_precedence_over_single_dna() {
        let mut bundle = bundle_with_outline(None, None);
        bundle.style_blend_text = Some("风格混合 [燃爽融合]: 热血:70%, 冷峻:30%".to_string());
        bundle.style_dna_extension = Some("单DNA六维内容".to_string());
        let prompt = bundle.to_prompt();
        assert!(prompt.contains("【风格混合"));
        assert!(prompt.contains("热血:70%"));
        assert!(
            !prompt.contains("【风格 DNA 六维指标】"),
            "blend 存在时不再渲染单 DNA 段"
        );
    }

    #[test]
    fn to_prompt_single_dna_rendered_when_no_blend() {
        let mut bundle = bundle_with_outline(None, None);
        bundle.style_dna_extension = Some("单DNA六维内容".to_string());
        let prompt = bundle.to_prompt();
        assert!(prompt.contains("【风格 DNA 六维指标】"));
        assert!(!prompt.contains("【风格混合"));
    }
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib write_time_bundle
```

预期输出（编译失败）：

```text
error[E0609]: no field `active_conflicts` on type `&mut WriteTimeBundle`
error[E0609]: no field `character_goals` on type `&mut WriteTimeBundle`
error[E0609]: no field `chase_debt_text` on type `&mut WriteTimeBundle`
error[E0609]: no field `genre_reference` on type `&mut WriteTimeBundle`
error[E0609]: no field `style_blend_text` on type `&mut WriteTimeBundle`
```

- [ ] **Step 3: domain 结构体新增 5 字段**

`src-tauri/src/domain/write_time_bundle.rs` 在 `related_entity_summaries` 字段（63 行）之后追加：

```rust
    /// 活跃冲突清单（writer_assets::format_active_conflicts，预算 ~600 字）
    pub active_conflicts: Option<String>,
    /// 角色目标/弧光/秘密（writer_assets::format_character_goals，每角色 ~200 字）
    pub character_goals: Option<String>,
    /// 追读力债务 + 本章追读力目标渲染文本（由 orchestrator 从 task.parameters 设置）
    pub chase_debt_text: Option<String>,
    /// 体裁元素参考表 + 典型结构（writer_assets::format_genre_reference_tables，预算 ~800 字）
    pub genre_reference: Option<String>,
    /// 风格混合 blend 文本（由 orchestrator 从 task.parameters 设置；渲染优先于单 DNA）
    pub style_blend_text: Option<String>,
```

- [ ] **Step 4: load_sync 填充新字段**

**4a.** `write_time_bundle.rs:252-281` 的 `genre_profile_strategy` 块重构为（先取 profile 复用，行为不变）：

```rust
        // v0.22.0: 加载 GenreProfile 完整策略（profile 取出后同时供 genre_reference 复用）
        let primary_genre_profile = {
            let genre_name = story.genre.as_deref().unwrap_or("");
            if genre_name.is_empty() {
                None
            } else {
                let genre_repo2 = GenreProfileRepository::new(pool.clone());
                genre_repo2.get_by_name(genre_name).ok().flatten()
            }
        };
        let genre_profile_strategy = {
            let genre_name = story.genre.as_deref().unwrap_or("");
            primary_genre_profile.as_ref().and_then(|profile| {
                let mut parts = vec![];
                if let Some(ref tone) = profile.core_tone {
                    parts.push(format!("基调：{}", tone));
                }
                if let Some(ref pacing) = profile.pacing_strategy {
                    parts.push(format!("节奏策略：{}", pacing));
                }
                if !parts.is_empty() {
                    Some(format!(
                        "【体裁画像策略（{}）】\n{}",
                        genre_name,
                        parts.join("\n")
                    ))
                } else {
                    None
                }
            })
        };
        // 设计第一节：体裁元素参考表 + 典型结构（复用 Task 1 共享函数，预算 ~800 字）
        let genre_reference = primary_genre_profile
            .as_ref()
            .and_then(|p| crate::agents::writer_assets::format_genre_reference_tables(p, 800));
```

**4b.** 在 `related_entity_summaries` 加载（356-360）之后、`Ok(WriteTimeBundle {`（433）之前插入：

```rust
        // 设计第一节：续写链路资产贯通——活跃冲突与角色目标复用 Task 1 共享函数，
        // 补齐 TimeSliced 死注入（预算 ~600 字 / 每角色 ~200 字）。
        let active_conflicts =
            crate::agents::writer_assets::format_active_conflicts(pool, story_id, 600);
        let character_goals =
            crate::agents::writer_assets::format_character_goals(pool, story_id, 200);
```

**4c.** `Ok(WriteTimeBundle {` 返回字面量（433-456）末尾 `related_entity_summaries,` 之后追加：

```rust
            active_conflicts,
            character_goals,
            chase_debt_text: None, // 由调用方（orchestrator）从 task.parameters 设置
            genre_reference,
            style_blend_text: None, // 由调用方（orchestrator）从 task.parameters 设置
```

- [ ] **Step 5: to_prompt() 新增段落**

**5a.** ⑧ 逾期伏笔块（725-735）之后插入：

```rust
        // ⑧b 活跃冲突清单（设计第一节：TimeSliced 补齐死注入）
        if let Some(ref conflicts) = self.active_conflicts {
            sections.push(conflicts.clone());
        }

        // ⑧c 角色目标/弧光/秘密
        if let Some(ref goals) = self.character_goals {
            sections.push(goals.clone());
        }

        // ⑧d 追读力债务 + 本章追读力目标（由 orchestrator 渲染后设置）
        if let Some(ref chase) = self.chase_debt_text {
            sections.push(chase.clone());
        }
```

**5b.** ⑪ 风格 DNA 块（748-751）替换为 blend 优先逻辑：

```rust
        // ⑪ 风格：优先风格混合 blend（多 DNA 融合），缺省回退单 DNA 六维指标
        if let Some(ref blend) = self.style_blend_text {
            sections.push(format!(
                "【风格混合（多风格融合，须兼顾各成分风格）】\n{}",
                blend
            ));
        } else if let Some(ref dna) = self.style_dna_extension {
            sections.push(format!("【风格 DNA 六维指标】\n{}", dna));
        }
```

**5c.** ⑬ 题材画像策略块（759-761）之后插入：

```rust
        // ⑬b 体裁元素参考表 + 典型结构（~800 字预算）
        if let Some(ref reference) = self.genre_reference {
            sections.push(format!("【体裁元素参考】\n{}", reference));
        }
```

- [ ] **Step 6: 补齐其余 7 处 WriteTimeBundle 字面量的新字段**

以下每处字面量都在末尾字段后追加同样 5 行：

```rust
            active_conflicts: None,
            character_goals: None,
            chase_debt_text: None,
            genre_reference: None,
            style_blend_text: None,
```

位置：`creative_engine/write_time_bundle.rs` 的 1058（`to_prompt_secondary_genre_strategy_rendered`）、1098（`to_prompt_includes_related_entity_summaries`）、1139（`to_prompt_redlines_appear_first`）、1219（`bundle_with_outline`）；`agents/orchestrator.rs:4953`（`progression_bundle`）；`creative_engine/prompt_synthesis/mod.rs:48`（`empty_bundle`）；`creative_engine/prompt_synthesis/manifest.rs:374`（`empty_bundle`）。

- [ ] **Step 7: 跑测试确认通过 + 回归**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib write_time_bundle
```

预期输出：既有测试全绿 + 新增 4 个测试通过，其中含：

```text
test creative_engine::write_time_bundle::tests::to_prompt_new_asset_sections_rendered ... ok
test creative_engine::write_time_bundle::tests::to_prompt_new_asset_sections_skipped_when_none ... ok
test creative_engine::write_time_bundle::tests::to_prompt_style_blend_takes_precedence_over_single_dna ... ok
test creative_engine::write_time_bundle::tests::to_prompt_single_dna_rendered_when_no_blend ... ok
```

再跑受影响回归：`cargo test --lib creative_engine::prompt_synthesis`、`cargo test --lib agents::orchestrator`（应全绿，字面量仅补字段）。

- [ ] **Step 8: 提交**

```bash
cd /Users/yuzaimu/projects/StoryForge && git add src-tauri/src/domain/write_time_bundle.rs src-tauri/src/creative_engine/write_time_bundle.rs src-tauri/src/agents/orchestrator.rs src-tauri/src/creative_engine/prompt_synthesis/mod.rs src-tauri/src/creative_engine/prompt_synthesis/manifest.rs && git commit -m "feat: WriteTimeBundle 新增活跃冲突/角色目标/追读力/体裁参考/风格混合字段与渲染段落"
```

---

## Task 3：TimeSliced 打通死注入 + 冲突约束语义化 + 风格 blend 透传

**Files:**
- Modify: `src-tauri/src/agents/writer_assets.rs`（新增 `render_chase_debt_and_reading_goal`，实现 + 测试）
- Modify: `src-tauri/src/agents/orchestrator.rs:952-962`（execute_time_sliced 四元组注入之后追加追读力/blend 读取）
- Modify: `src-tauri/src/creative_engine/write_time_bundle.rs:929-940`（`format_writing_strategy_constraints` 语义化）、`322-328`（load_sync 默认约束同步改）
- Modify: `src-tauri/src/planner/executor.rs:1196-1199`（execute_writer 注入段追加 style_blend_text 透传）
- Modify: `src-tauri/src/planner/mod.rs`（新增 `style_blend_text_for_writer` + 测试；PlanContext 定义在 68-104）
- Test: 上述各文件内 tests

**Interfaces:**
- Consumes：`planner/executor.rs:1287-1342` 已注入的 `chase_debt_count` / `chase_debts` / `reading_power_hook_type` / `reading_power_hook_strength` / `reading_power_foreshadowing_list` / `reading_power_micropayoff_count`；`PlanContext.style_dna_info`（commands/orchestrator.rs:612-651 已拼好，无需改 commands 层）。
- Produces：
  - `pub(crate) fn render_chase_debt_and_reading_goal(pool: &crate::db::DbPool, params: &std::collections::HashMap<String, serde_json::Value>) -> Option<String>`（writer_assets.rs；debt_count>0 才渲染债务段、hook_type 存在才渲染目标段，两段皆无返回 None）
  - `pub(crate) fn style_blend_text_for_writer(ctx: &PlanContext) -> Option<String>`（planner/mod.rs；仅 `"风格混合"` 前缀的 blend 文本透传，`"风格DNA ID: ..."` 回退文本不透传——bundle 已从 `story.style_dna_id` 自载单 DNA）
  - writer 参数 `style_blend_text`（executor → AgentTask.parameters → execute_time_sliced → `bundle.style_blend_text`）

- [ ] **Step 1: 写失败测试**

**1a.** writer_assets.rs tests 模块追加：

```rust
    // ---- render_chase_debt_and_reading_goal ----

    #[test]
    fn render_chase_debt_and_goal_with_assets() {
        let pool = create_test_pool().unwrap();
        let mut params = std::collections::HashMap::new();
        params.insert(
            "chase_debt_count".to_string(),
            serde_json::Value::String("2".to_string()),
        );
        params.insert(
            "chase_debts".to_string(),
            serde_json::Value::String(
                "1. 类型：钩子，当前金额：3.0，到期章节：5，来源章节：2".to_string(),
            ),
        );
        params.insert(
            "reading_power_hook_type".to_string(),
            serde_json::Value::String("身份悬念".to_string()),
        );
        params.insert(
            "reading_power_hook_strength".to_string(),
            serde_json::Value::String("high".to_string()),
        );
        params.insert(
            "reading_power_foreshadowing_list".to_string(),
            serde_json::Value::String("身世之谜".to_string()),
        );
        params.insert(
            "reading_power_micropayoff_count".to_string(),
            serde_json::Value::String("2".to_string()),
        );
        let text = render_chase_debt_and_reading_goal(&pool, &params).expect("有资产应渲染");
        assert!(text.contains("【追读力债务】"));
        assert!(text.contains("当前有 2 条待偿还的追读力债务"));
        assert!(text.contains("到期章节：5"));
        assert!(text.contains("【本章追读力目标】"));
        assert!(text.contains("身份悬念"));
        assert!(text.contains("身世之谜"));
    }

    #[test]
    fn render_chase_debt_and_goal_empty_params_returns_none() {
        let pool = create_test_pool().unwrap();
        let params = std::collections::HashMap::new();
        assert!(render_chase_debt_and_reading_goal(&pool, &params).is_none());
        // debt_count 为 "0"（executor 缺省注入值）时跳过债务段
        let mut zero = std::collections::HashMap::new();
        zero.insert(
            "chase_debt_count".to_string(),
            serde_json::Value::String("0".to_string()),
        );
        assert!(render_chase_debt_and_reading_goal(&pool, &zero).is_none());
    }

    #[test]
    fn render_chase_debt_and_goal_only_hook_type() {
        // 仅 hook_type 无债务：只渲染目标段
        let pool = create_test_pool().unwrap();
        let mut params = std::collections::HashMap::new();
        params.insert(
            "reading_power_hook_type".to_string(),
            serde_json::Value::String("（延续）".to_string()),
        );
        let text = render_chase_debt_and_reading_goal(&pool, &params).expect("应有目标段");
        assert!(!text.contains("【追读力债务】"));
        assert!(text.contains("【本章追读力目标】"));
        assert!(text.contains("（延续）"));
    }
```

**1b.** write_time_bundle.rs tests 模块追加：

```rust
    #[test]
    fn format_writing_strategy_constraints_semantic() {
        let strategy = crate::config::settings::WritingStrategy {
            run_mode: "standard".to_string(),
            conflict_level: 85,
            pace: "fast".to_string(),
            ai_freedom: "medium".to_string(),
        };
        let text = format_writing_strategy_constraints(&strategy);
        assert!(text.contains("【写作策略约束】"));
        assert!(text.contains("运行模式：standard"));
        assert!(text.contains("冲突强度：极高"));
        assert!(text.contains("叙事节奏：快"));
        assert!(text.contains("AI 自由度：medium"));
        assert!(!text.contains("冲突强度：85"), "不再输出裸数字冲突强度");
    }
```

**1c.** planner/mod.rs tests 模块追加（字面量字段与 824-853 行 `test_plan_context_defaults` 一致）：

```rust
    #[test]
    fn test_style_blend_text_for_writer() {
        let base = PlanContext {
            current_story_id: None,
            has_story: false,
            has_chapters: false,
            chapter_count: 0,
            current_content_preview: None,
            user_input: "test".to_string(),
            scene_count: 0,
            scenes_summary: vec![],
            current_scene_id: None,
            current_scene_stage: None,
            total_word_count: 0,
            latest_chapter_word_count: 0,
            story_progress: "just_started".to_string(),
            selected_text: None,
            world_building_summary: None,
            character_list: vec![],
            foreshadowing_status: vec![],
            style_dna_info: None,
            mcp_tools_available: vec![],
            deep_insight_summary: None,
            style_weight: 50,
            chapter_number: 1,
            selected_strategy: None,
            intent_classification: None,
        };
        // blend 文本透传
        let mut ctx = base.clone();
        ctx.style_dna_info = Some("风格混合 [燃爽融合]: 热血:70%, 冷峻:30%".to_string());
        assert_eq!(
            style_blend_text_for_writer(&ctx),
            Some("风格混合 [燃爽融合]: 热血:70%, 冷峻:30%".to_string())
        );
        // 单 DNA 回退文本不透传（bundle 已从 story.style_dna_id 自载）
        let mut ctx2 = base.clone();
        ctx2.style_dna_info = Some("风格DNA ID: abc123".to_string());
        assert!(style_blend_text_for_writer(&ctx2).is_none());
        // 无风格信息
        assert!(style_blend_text_for_writer(&base).is_none());
    }
```

（`PlanContext` 派生了 `Debug, Clone`——planner/mod.rs:67，clone 可用。）

- [ ] **Step 2: 跑测试确认失败**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib
```

预期输出（编译失败）：

```text
error[E0425]: cannot find function `render_chase_debt_and_reading_goal` in module `super`
error[E0425]: cannot find function `style_blend_text_for_writer` in this scope
```

（`format_writing_strategy_constraints_semantic` 此时尚能编译但断言会失败——旧实现输出 `冲突强度：85` 裸数字；三个红灯齐备。）

- [ ] **Step 3: 实现 render_chase_debt_and_reading_goal（writer_assets.rs）**

```rust
/// 渲染追读力债务 + 本章追读力目标段落，供 TimeSliced 路径消费 executor
/// 注入的追读力参数（原仅 Full 路径消费，service.rs:2041-2111）。
///
/// 有资产才渲染：`chase_debt_count` 解析为 0 或缺失时跳过债务段；
/// `reading_power_hook_type` 缺失时跳过目标段；两段皆跳过返回 None。
pub(crate) fn render_chase_debt_and_reading_goal(
    pool: &crate::db::DbPool,
    params: &std::collections::HashMap<String, serde_json::Value>,
) -> Option<String> {
    let mut sections = Vec::new();

    // 追读力债务（模板 writer_chase_debt，对齐 Full 路径 service.rs:2041-2067 的变量）
    let debt_count = params
        .get("chase_debt_count")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    if debt_count > 0 {
        let mut debt_vars = std::collections::HashMap::new();
        debt_vars.insert("debt_count".to_string(), debt_count.to_string());
        debt_vars.insert(
            "debts".to_string(),
            params
                .get("chase_debts")
                .and_then(|v| v.as_str())
                .unwrap_or("无")
                .to_string(),
        );
        let tpl = crate::prompts::registry::resolve_prompt(pool, "writer_chase_debt")
            .ok()
            .or_else(|| crate::prompts::registry::resolve_prompt_default("writer_chase_debt"));
        if let Some(tpl) = tpl {
            let rendered =
                crate::prompts::engine::TemplateEngine::render_with_conditions(&tpl, &debt_vars);
            if !rendered.trim().is_empty() {
                sections.push(rendered);
            }
        }
    }

    // 本章追读力目标（模板 writer_reading_power_goal，对齐 service.rs:2069-2111 的变量）
    if let Some(hook_type) = params
        .get("reading_power_hook_type")
        .and_then(|v| v.as_str())
    {
        let mut goal_vars = std::collections::HashMap::new();
        goal_vars.insert("hook_type".to_string(), hook_type.to_string());
        goal_vars.insert(
            "hook_strength".to_string(),
            params
                .get("reading_power_hook_strength")
                .and_then(|v| v.as_str())
                .unwrap_or("medium")
                .to_string(),
        );
        goal_vars.insert(
            "foreshadowing_list".to_string(),
            params
                .get("reading_power_foreshadowing_list")
                .and_then(|v| v.as_str())
                .unwrap_or("无")
                .to_string(),
        );
        goal_vars.insert(
            "micropayoff_count".to_string(),
            params
                .get("reading_power_micropayoff_count")
                .and_then(|v| v.as_str())
                .unwrap_or("1-2")
                .to_string(),
        );
        let tpl = crate::prompts::registry::resolve_prompt(pool, "writer_reading_power_goal")
            .ok()
            .or_else(|| {
                crate::prompts::registry::resolve_prompt_default("writer_reading_power_goal")
            });
        if let Some(tpl) = tpl {
            let rendered =
                crate::prompts::engine::TemplateEngine::render_with_conditions(&tpl, &goal_vars);
            if !rendered.trim().is_empty() {
                sections.push(rendered);
            }
        }
    }

    if sections.is_empty() {
        None
    } else {
        Some(sections.join("\n\n"))
    }
}
```

- [ ] **Step 4: execute_time_sliced 读取参数写入 bundle（orchestrator.rs）**

在 `agents/orchestrator.rs` 四元组注入块（952-962，`bundle.narrative_quartet = Some(rendered);` 所在 if 块结束 `}` 962 行）之后插入：

```rust
        // 设计第一节：打通 executor 死注入——追读力债务/钩子类型/微兑现经共享渲染
        // 函数进 bundle.chase_debt_text（复用 writer_chase_debt /
        // writer_reading_power_goal 模板）；风格混合 blend 文本透传到
        // bundle.style_blend_text（to_prompt 渲染时优先 blend、回退单 DNA）。
        if let Some(text) = crate::agents::writer_assets::render_chase_debt_and_reading_goal(
            pool.inner(),
            &task.parameters,
        ) {
            bundle.chase_debt_text = Some(text);
        }
        if let Some(blend) = task
            .parameters
            .get("style_blend_text")
            .and_then(|v| v.as_str())
        {
            bundle.style_blend_text = Some(blend.to_string());
        }
```

- [ ] **Step 5: format_writing_strategy_constraints 语义化（write_time_bundle.rs）**

**5a.** 929-940 整个函数替换为：

```rust
/// v0.23.59: 将 `WritingStrategy` 格式化为写作策略约束提示文本。
///
/// 冲突强度与叙事节奏复用 Full 路径的分档语义文案（writer_assets 共享函数，
/// 原 service.rs:1888-1908），替代此前的裸数字（"冲突强度：0.5"）。
pub fn format_writing_strategy_constraints(
    strategy: &crate::config::settings::WritingStrategy,
) -> String {
    format!(
        "【写作策略约束】\n运行模式：{}\n{}\nAI 自由度：{}",
        strategy.run_mode,
        crate::agents::writer_assets::writing_constraints_semantic_text(
            strategy.conflict_level as f64,
            crate::agents::writer_assets::pace_to_factor(&strategy.pace),
        ),
        strategy.ai_freedom,
    )
}
```

**5b.** load_sync 中的硬编码默认值（322-328）同步改为同一函数（消除双份格式）：

```rust
        // v0.22.0: 加载写作策略约束（默认值；execute_time_sliced 会用 AppConfig 覆盖）
        let writing_strategy_constraints = Some(format_writing_strategy_constraints(
            &crate::config::settings::WritingStrategy::default(),
        ));
```

- [ ] **Step 6: style_blend_text 透传链（planner/mod.rs + planner/executor.rs）**

**6a.** `planner/mod.rs` 在 `PlanContext` 结构体定义（68-104）之后追加自由函数：

```rust
/// 从 PlanContext 提取风格混合 blend 文本，供 executor 透传给 writer 参数。
/// 仅当 style_dna_info 为 blend 形式（"风格混合" 前缀，commands/orchestrator.rs:631
/// 拼装）时返回；单 DNA 回退文本（"风格DNA ID: ..."）返回 None——bundle 已从
/// story.style_dna_id 自行加载单 DNA 六维指标。
pub(crate) fn style_blend_text_for_writer(ctx: &PlanContext) -> Option<String> {
    ctx.style_dna_info
        .as_ref()
        .filter(|info| info.starts_with("风格混合"))
        .cloned()
}
```

**6b.** `planner/executor.rs` 在 `enriched_params.insert("story_progress", ...)`（1196-1199）之后插入：

```rust
        // 设计第一节：风格混合 blend 文本经 writer 参数透传到 TimeSliced bundle
        // （commands/orchestrator.rs:612-651 已拼好 blend 文本进 PlanContext）。
        if let Some(blend) = crate::planner::style_blend_text_for_writer(plan_context) {
            enriched_params.insert(
                "style_blend_text".to_string(),
                serde_json::Value::String(blend),
            );
        }
```

- [ ] **Step 7: 跑测试确认通过 + 回归**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib writer_assets && cargo test --lib write_time_bundle && cargo test --lib planner
```

预期输出：三组全绿，含：

```text
test agents::writer_assets::tests::render_chase_debt_and_goal_with_assets ... ok
test agents::writer_assets::tests::render_chase_debt_and_goal_empty_params_returns_none ... ok
test agents::writer_assets::tests::render_chase_debt_and_goal_only_hook_type ... ok
test creative_engine::write_time_bundle::tests::format_writing_strategy_constraints_semantic ... ok
test planner::tests::test_style_blend_text_for_writer ... ok
```

- [ ] **Step 8: 提交**

```bash
cd /Users/yuzaimu/projects/StoryForge && git add src-tauri/src/agents/writer_assets.rs src-tauri/src/agents/orchestrator.rs src-tauri/src/creative_engine/write_time_bundle.rs src-tauri/src/planner/executor.rs src-tauri/src/planner/mod.rs && git commit -m "feat: TimeSliced 打通追读力死注入、冲突约束语义化并透传风格混合 blend"
```

---

## Task 4：推进锚点去重（build_progression_anchor）

依赖：Task 2 已给 `progression_bundle` 测试 helper 补完新字段。

**Files:**
- Modify: `src-tauri/src/agents/orchestrator.rs:3751-3852`（`build_progression_anchor` 签名 + 去重分支）、`1046-1052`（TimeSliced 调用方）、`1662-1668`（TriShot 调用方）、`5043/5071/5082`（既有测试调用补参）
- Test: `src-tauri/src/agents/orchestrator.rs` 内 tests（新增去重断言测试）

**Interfaces:**
- Consumes：`WriteTimeBundle`（to_prompt 已渲染的 story_outline / scene_outline.outline_content / world_setting 段落）、`build_recent_outline_progress`（3857 起，不变）。
- Produces：`fn build_progression_anchor(bundle, pool, story_id, chapter_number, user_instruction, bundle_already_rendered: bool) -> String`（私有函数，签名追加第 6 参）。
  - `bundle_already_rendered = true`（TimeSliced，1046 调用方）：跳过「故事大纲（硬约束）」「本章场景大纲（硬约束）」「世界观核心规则（硬约束）」三段——bundle.to_prompt() 已含 ①b/①c/③ 同款内容；只保留「本次创作指令」+「已推进进度」+ 调和指令。
  - `bundle_already_rendered = false`（TriShot，1662 调用方）：行为与现状完全一致（其 synthesized_prompt 不含 bundle 段落，去重会导致大纲/世界观彻底丢失）。

- [ ] **Step 1: 写失败测试（orchestrator.rs tests 模块追加）**

```rust
    #[test]
    fn test_build_progression_anchor_dedup_when_bundle_rendered() {
        // 设计第一节：TimeSliced 路径 bundle.to_prompt() 已含故事大纲/场景大纲/
        // 世界观段落，锚点不再重复注入，只保留指令 + 已推进进度 + 调和指令。
        let pool = crate::db::create_test_pool().unwrap();
        use crate::db::{
            dto::CreateStoryRequest,
            repositories::{SceneRepository, SceneUpdate, StoryRepository},
        };
        let story = StoryRepository::new(pool.clone())
            .create(CreateStoryRequest {
                title: "测试书".into(),
                description: None,
                genre: None,
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: None,
                reference_book_id: None,
            })
            .unwrap();
        let scene_repo = SceneRepository::new(pool.clone());
        let s1 = scene_repo.create(&story.id, 1, Some("第一章")).unwrap();
        scene_repo
            .update(
                &s1.id,
                &SceneUpdate {
                    outline_content: Some("主角抵达首尔，遭遇伏击".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        let bundle = progression_bundle(
            Some("1. 首尔暗战\n2. 核电站阴谋\n3. 真相对决".to_string()),
            Some("世界规则：核能受严格管控".to_string()),
            Some("本章：主角对峙反派".to_string()),
        );
        let anchor = build_progression_anchor(
            &bundle,
            &pool,
            &story.id,
            2,
            "续写主角揭穿核电站阴谋，但反派设下陷阱",
            true,
        );
        assert!(anchor.contains("剧情推进方向"), "应含推进方向总段");
        assert!(anchor.contains("本次创作指令"), "应含用户指令段");
        assert!(anchor.contains("已推进进度"), "应保留已推进进度段");
        assert!(anchor.contains("遭遇伏击"), "进度应含第一章 outline_content");
        assert!(
            !anchor.contains("故事大纲（硬约束"),
            "bundle 已渲染时不再重复注入故事大纲"
        );
        assert!(
            !anchor.contains("本章场景大纲（硬约束"),
            "bundle 已渲染时不再重复注入场景大纲"
        );
        assert!(
            !anchor.contains("世界观核心规则（硬约束"),
            "bundle 已渲染时不再重复注入世界观"
        );
    }
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib agents::orchestrator
```

预期输出（编译失败——函数当前只有 5 个参数）：

```text
error[E0061]: this function takes 5 arguments but 6 arguments were supplied
   --> src/agents/orchestrator.rs:...:...
```

- [ ] **Step 3: 实现去重（签名加参 + 三段包裹 + 调和文案分支出）**

**3a.** `build_progression_anchor`（3765-3852）签名追加参数，函数文档注释同步更新：

```rust
/// v0.30.31: 剧情推进方向锚点--确定性注入故事大纲/场景大纲/世界观/已推进进度
/// 到 writer prompt。v0.30.32: 纳入用户本次创作指令并显式调和。
/// 设计第一节（去重）：`bundle_already_rendered = true` 时（TimeSliced 路径，
/// bundle.to_prompt() 已含 ①b 故事大纲 / ①c 世界观 / ③ 场景大纲同款段落）
/// 跳过三段硬约束注入，只保留「本次创作指令 + 已推进进度 + 调和指令」；
/// TriShot 正常路径（synthesized_prompt 不含 bundle 段落）必须传 false。
fn build_progression_anchor(
    bundle: &crate::domain::write_time_bundle::WriteTimeBundle,
    pool: &crate::db::DbPool,
    story_id: &str,
    chapter_number: i32,
    user_instruction: &str,
    bundle_already_rendered: bool,
) -> String {
```

**3b.** 函数体内三段硬约束分别用 `if !bundle_already_rendered { ... }` 包裹（3784-3794 故事大纲段、3796-3808 场景大纲段、3821-3831 世界观段），「已推进进度」段（3810-3819）保持不变：

```rust
    // 故事大纲（前 1200 字）--硬约束；bundle 已渲染同款段落时跳过（去重）
    if !bundle_already_rendered {
        if let Some(ref outline) = bundle.story_outline {
            if !outline.trim().is_empty() {
                let truncated: String = outline.chars().take(1200).collect();
                sections.push(format!(
                    "【故事大纲（硬约束：必须围绕展开，禁止偏离）】\n{}",
                    truncated
                ));
                has_assets = true;
            }
        }
    }

    // 本章场景大纲 outline_content（前 800 字）--硬约束；同上跳过
    if !bundle_already_rendered {
        if let Some(ref scene) = bundle.scene_outline {
            if let Some(ref oc) = scene.outline_content {
                if !oc.trim().is_empty() {
                    let truncated: String = oc.chars().take(800).collect();
                    sections.push(format!(
                        "【本章场景大纲（硬约束：必须遵循的章节方向）】\n{}",
                        truncated
                    ));
                    has_assets = true;
                }
            }
        }
    }
```

（「已推进进度」段原样保留；世界观段同样包裹：）

```rust
    // 世界观核心规则（前 600 字）--硬约束；同上跳过
    if !bundle_already_rendered {
        if let Some(ref world) = bundle.world_setting {
            if !world.trim().is_empty() {
                let truncated: String = world.chars().take(600).collect();
                sections.push(format!(
                    "【世界观核心规则（硬约束：须遵循，违反即严重错误）】\n{}",
                    truncated
                ));
                has_assets = true;
            }
        }
    }
```

**3c.** 调和指令（3839-3845）在 `!directive.is_empty() && has_assets` 分支按去重状态分文案，其余两分支不变：

```rust
    let closing = if !directive.is_empty() && has_assets {
        if bundle_already_rendered {
            "\n- 本次创作指令是你的创作方向；上方已注入的故事大纲/场景大纲/世界观是硬约束，已推进进度是承接指针。须在硬约束内落实指令核心意图--推进到故事大纲下一节点、承接已推进进度。若指令与某硬约束冲突，调整指令的具体表现以符合约束，但保留指令核心意图；不得因约束丢弃指令，也不得因指令违反约束。"
        } else {
            "\n- 本次创作指令是你的创作方向；故事大纲/场景大纲/世界观/已推进进度是硬约束。须在硬约束内落实指令核心意图--推进到故事大纲下一节点、遵循世界观规则、承接已推进进度。若指令与某硬约束冲突，调整指令的具体表现以符合约束，但保留指令核心意图；不得因约束丢弃指令，也不得因指令违反约束。"
        }
    } else if !directive.is_empty() {
        "\n- 推进剧情向前发展，不得原地踏步、不得仅复述设定或复述前文。"
    } else {
        "\n- 必须推进到故事大纲的下一节点，不得原地踏步、不得仅复述设定或复述前文。角色行为须在世界观规则约束内，与已推进进度承接连贯。"
    };
```

**3d.** 两个调用方补参：

- TimeSliced（1046-1052）：`build_progression_anchor(&bundle, pool.inner(), &task.context.story.story_id, chapter_number, &user_instruction, true)`（其上方 1040-1045 的 v0.30.39 注释同步改写为：「设计第一节：bundle_already_rendered=true——bundle.to_prompt() 已含大纲/世界观，锚点只补进度指针与调和指令」）。
- TriShot（1662-1668）：同一调用末尾补 `false`（注释不变，仍成立）。

**3e.** 既有 3 个测试调用补 `false`（行为不变）：5043（`test_build_progression_anchor_injects_all_sections`）、5071（`test_build_progression_anchor_empty_returns_empty`）、5082（`test_build_progression_anchor_directive_only_no_assets`）。

- [ ] **Step 4: 跑测试确认通过 + 回归**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib agents::orchestrator
```

预期输出：全绿，含：

```text
test agents::orchestrator::tests::test_build_progression_anchor_dedup_when_bundle_rendered ... ok
test agents::orchestrator::tests::test_build_progression_anchor_directive_only_no_assets ... ok
test agents::orchestrator::tests::test_build_progression_anchor_empty_returns_empty ... ok
test agents::orchestrator::tests::test_build_progression_anchor_injects_all_sections ... ok
```

- [ ] **Step 5: 提交**

```bash
cd /Users/yuzaimu/projects/StoryForge && git add src-tauri/src/agents/orchestrator.rs && git commit -m "refactor: 推进锚点在 bundle 已渲染段落时去除大纲/世界观重复注入"
```

---

## 全部分完成后的回归门槛

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib
```

应全绿（设计第七节回归门槛中属于本部分的部分；`npx tsc --noEmit` 无前端改动不受影响）。

## 风险与备注

- **快照重复聚合开销**：`format_active_conflicts` / `format_character_goals` 各自调用一次 `get_snapshot_sync`。service.rs 原路径因此多两次本地 SQLite 聚合（原有的一次仍保留给叙事阶段/伏笔段），bundle load_sync 多两次。均为内存级本地查询，可接受；如需优化可后续加 snapshot 参数重载，本部分不做。
- **TriShot 行为零变化**：Task 4 的去重只对 TimeSliced 生效；TriShot 传 `false`，输出与现状逐字一致，防止大纲/世界观从 TriShot writer 丢失。
- **`has_assets` 语义变化**：去重模式下只有「已推进进度」计入 `has_assets`，调和文案相应区分（见 Task 4 Step 3c）；无进度无前序章节时回落到既有「推进剧情向前发展」分支，行为与现有一致。
- **executor 参数缺省值**：`chase_debt_count` 缺省注入 `"0"`、`hook_type` 缺省 `"（未指定）"`；`render_chase_debt_and_reading_goal` 对 `"0"` 跳过债务段，但 hook_type 恒存在时目标段总会渲染——与 Full 路径（无条件渲染目标段）行为对齐，属预期。

---

# 第二部分：推荐资产贯通 + 方法论动态化 + 扩张性写作合约（Task 5-8）

# StoryMoss 资产融合深度重构 — 实施计划 Part 2（Task 5–8）

> 设计依据：`docs/plans/2026-08-04-asset-fusion-deep-restructure-design.md` 第二节「推荐资产贯通 + 方法论动态化」与第三节「扩张性写作合约」。
>
> 通用约定：
> - 工作目录 `/Users/yuzaimu/projects/StoryForge`，Rust  crate 在 `src-tauri/`。
> - 测试全部写在各 lib 源文件内 `#[cfg(test)] mod tests`，运行命令统一为
>   `cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib <filter>`。
> - 提交信息使用中文 conventional commit。
> - PromptRegistry 内置提示词在测试/开发环境从 `CARGO_MANIFEST_DIR/../resources/prompts` 目录加载
>   （`prompts/registry.rs:282-296 prompts_resource_dir()`），因此 `resolve_prompt_default`
>   在 `cargo test` 中可直接命中 `resources/prompts/**/*.md`。
> - 关键既有事实（已读源码确认）：
>   - `SelectedStrategy`（`domain/strategy.rs:12-45`）含 `methodology_id / style_dna_ids / skill_ids`。
>   - `build_selected_strategy`（`commands/orchestrator.rs:1284-1289`）仅当 `story.methodology_id.is_none()`
>     时才把体裁画像推荐方法论写入 strategy —— 因此「推荐值优先」不会覆盖用户显式选择。
>   - `StoryRepository::update`（`db/repositories/story_repository.rs:208-240`）用 COALESCE 语义，
>     `UpdateStoryRequest`（`db/dto.rs:141-152`）已含 `methodology_id / methodology_step`。
>   - `StoryContextBuilder` 已把 `stories.methodology_step` 写入 `context.world.methodology_step`
>     （`creative_engine/context_builder.rs:598`），TimeSliced 路径可读。
>   - `methodology/` 目录现有 17 个 md：snowflake 10 个 step 文件、hdwb 4 个阶段文件
>     （seed/expansion/convergence/iteration，未按 step 规范命名）、hero_journey /
>     scene_structure / character_depth 各 1 个单文件。

---

## Task 5: 推荐资产透传与写回

体裁画像推荐（或用户锁定）的 methodology_id / style_dna_ids / skill_ids 从 `SelectedStrategy`
注入 writer 步骤参数；TimeSliced 路径优先消费推荐值、story 字段回退；story 无显式方法论时把推荐值写回 stories 表；skill_ids 对应技能提示词摘要以文本形式注入 writer system prompt。

**Files:**
- Modify: `src-tauri/src/planner/executor.rs`（`execute_writer` 的 `selected_strategy` 块 1221-1250 内接线；新增纯函数于 `inject_inspector_draft_fallback`（904）附近；测试在 2173 的 `mod tests`）
- Modify: `src-tauri/src/creative_engine/write_time_bundle.rs`（工具函数区 927-940 附近新增 `resolve_methodology_extension`；测试在 944 的 `mod tests`）
- Modify: `src-tauri/src/agents/orchestrator.rs`（`execute_time_sliced`：bundle 后处理 955-962 之后、writer system prompt 1079-1083 之后；测试在 4419 的 `mod tests`）
- Modify: `src-tauri/src/agents/service.rs`（`render_writer_system_from_bundle`（3301-3379）之后新增 `render_skill_guidance`；测试在 3688 的 `#[cfg(test)]` 区）
- Modify: `src-tauri/src/prompts/registry.rs`（`prompt_display_name`（194-199）之后新增 `prompt_description`）
- Modify: `src-tauri/src/commands/orchestrator.rs`（`smart_execute_inner` 740-746 之间接线写回；`build_selected_strategy`（1188-1329）之后新增 `persist_recommended_methodology`；测试在 1361 的 `mod tests`）

**Interfaces:**
- Consumes:
  - `crate::domain::strategy::SelectedStrategy { methodology_id: Option<String>, style_dna_ids: Vec<String>, skill_ids: Vec<String>, .. }`（`domain/strategy.rs:12-45`）
  - `crate::planner::PlanContext.selected_strategy: Option<SelectedStrategy>`
  - `crate::prompts::registry::resolve_prompt_default(prompt_id: &str) -> Option<String>`（`prompts/registry.rs:447`）
  - `crate::prompts::registry::prompt_display_name(prompt_id: &str) -> String`（`prompts/registry.rs:194`）
  - `crate::db::StoryRepository::{create, get_by_id, update}` + `crate::db::{CreateStoryRequest, UpdateStoryRequest}`
  - `task.context.world.methodology_step: Option<String>`（由 `context_builder.rs:598` 填充）
- Produces:
  - `pub(crate) fn inject_recommended_strategy_params(enriched_params: &mut HashMap<String, serde_json::Value>, selected: &SelectedStrategy)`（planner/executor.rs）→ task.parameters 新键 `recommended_methodology_id: String` / `recommended_style_dna_ids: Vec<String>` / `recommended_skill_ids: Vec<String>`
  - `pub fn resolve_methodology_extension(methodology_id: &str, step: i32) -> Option<String>`（write_time_bundle.rs）
  - `pub fn prompt_description(prompt_id: &str) -> Option<String>`（prompts/registry.rs）
  - `pub fn render_skill_guidance(skill_ids: &[String]) -> Option<String>`（agents/service.rs）
  - `fn persist_recommended_methodology(pool: &DbPool, current_story: &Option<Story>, selected_strategy: &Option<SelectedStrategy>) -> bool`（commands/orchestrator.rs）

### Steps

- [ ] **Step 1: 写 executor 透传纯函数的失败测试**

在 `src-tauri/src/planner/executor.rs` 的 `mod tests`（2173 行起）末尾追加：

```rust
    #[test]
    fn test_inject_recommended_strategy_params() {
        let mut params = HashMap::new();
        let mut selected = crate::domain::strategy::SelectedStrategy::default();
        selected.methodology_id = Some("snowflake".to_string());
        selected.style_dna_ids = vec!["dna_a".to_string(), "dna_b".to_string()];
        selected.skill_ids = vec!["emotion_pacing".to_string()];

        inject_recommended_strategy_params(&mut params, &selected);

        assert_eq!(
            params
                .get("recommended_methodology_id")
                .unwrap()
                .as_str()
                .unwrap(),
            "snowflake"
        );
        let dna = params
            .get("recommended_style_dna_ids")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(dna.len(), 2);
        assert_eq!(dna[0].as_str().unwrap(), "dna_a");
        let skills = params
            .get("recommended_skill_ids")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(skills[0].as_str().unwrap(), "emotion_pacing");
    }

    #[test]
    fn test_inject_recommended_strategy_params_empty_strategy_noop() {
        let mut params = HashMap::new();
        let selected = crate::domain::strategy::SelectedStrategy::default();
        inject_recommended_strategy_params(&mut params, &selected);
        assert!(params.get("recommended_methodology_id").is_none());
        assert!(params.get("recommended_style_dna_ids").is_none());
        assert!(params.get("recommended_skill_ids").is_none());
    }
```

- [ ] **Step 2: 跑测试确认失败**

```
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib inject_recommended_strategy_params
```

预期：编译失败，`error[E0425]: cannot find function inject_recommended_strategy_params in this scope`。

- [ ] **Step 3: 实现透传函数并接线到 execute_writer**

在 `planner/executor.rs` 的 `inject_inspector_draft_fallback`（904-948）之后（同 `impl PlanExecutor` 块外、文件级自由函数区，与 `resolve_parameters` 同文件作用域）新增：

```rust
/// v0.31.0: 把 SelectedStrategy 推荐的方法论/风格 DNA/技能 ID 注入 writer
/// 步骤参数，供 TimeSliced 路径优先于 story 字段消费（推荐资产贯通）。
/// 注意：build_selected_strategy 仅当 story 无显式 methodology_id 时才填推荐值，
/// 因此「推荐优先」不会覆盖用户显式选择。
pub(crate) fn inject_recommended_strategy_params(
    enriched_params: &mut HashMap<String, serde_json::Value>,
    selected: &crate::domain::strategy::SelectedStrategy,
) {
    if let Some(ref mid) = selected.methodology_id {
        if !mid.trim().is_empty() {
            enriched_params.insert(
                "recommended_methodology_id".to_string(),
                serde_json::Value::String(mid.clone()),
            );
        }
    }
    if !selected.style_dna_ids.is_empty() {
        enriched_params.insert(
            "recommended_style_dna_ids".to_string(),
            serde_json::to_value(&selected.style_dna_ids).unwrap_or_default(),
        );
    }
    if !selected.skill_ids.is_empty() {
        enriched_params.insert(
            "recommended_skill_ids".to_string(),
            serde_json::to_value(&selected.skill_ids).unwrap_or_default(),
        );
    }
}
```

在 `execute_writer` 的 `if let Some(ref selected) = plan_context.selected_strategy {`（1221 行）块内、四元组注入（1222 行）之前插入：

```rust
            // v0.31.0: 推荐方法论/风格 DNA/技能 ID 透传 writer 参数
            inject_recommended_strategy_params(&mut enriched_params, selected);
```

- [ ] **Step 4: 写方法论动态解析函数与技能摘要的失败测试**

在 `src-tauri/src/creative_engine/write_time_bundle.rs` 的 `mod tests`（944 行起）追加：

```rust
    #[test]
    fn test_resolve_methodology_extension_step_variant_hit() {
        // snowflake step3 有独立 md 文件，应命中 step 变体
        let ext = resolve_methodology_extension("snowflake", 3).expect("snowflake step3 应命中");
        assert!(
            ext.contains("雪花法第3步：角色概要"),
            "应使用 md frontmatter 的 name 作为标签: {}",
            ext
        );
        assert!(ext.contains("为每个主要角色写一页概要"));
    }

    #[test]
    fn test_resolve_methodology_extension_hdwb_legacy_alias() {
        // hdwb 文件未按 step 规范命名，走兼容映射；id 别名 hdwb 同样归一化
        let ext = resolve_methodology_extension("high_density_world_building", 2)
            .expect("hdwb step2 应命中旧命名 alias");
        assert!(ext.contains("状态网扩张"));
        let ext_alias = resolve_methodology_extension("hdwb", 1).expect("hdwb 别名应命中");
        assert!(ext_alias.contains("最小世界种子"));
    }

    #[test]
    fn test_resolve_methodology_extension_unknown_returns_none() {
        assert!(resolve_methodology_extension("nonexistent_methodology_xyz", 1).is_none());
    }
```

在 `src-tauri/src/agents/service.rs` 文件末尾的 `#[cfg(test)]` 测试区（3688 行起任一模测试块内）追加：

```rust
    #[test]
    fn test_render_skill_guidance_known_and_unknown() {
        let g = render_skill_guidance(&[
            "emotion_pacing".to_string(),
            "builtin.style_enhancer".to_string(),
        ])
        .expect("已知技能应生成指导段落");
        assert!(g.contains("激活写作技能"));
        assert!(g.contains("情感节奏优化提示词"));
        assert!(g.contains("分析并优化文本的情感曲线和叙事节奏"));

        // 全部未知 → None
        assert!(render_skill_guidance(&["no_such_skill_xyz".to_string()]).is_none());
    }
```

- [ ] **Step 5: 跑测试确认失败**

```
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib resolve_methodology_extension
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib render_skill_guidance
```

预期：编译失败，`cannot find function resolve_methodology_extension` / `cannot find function render_skill_guidance`。

- [ ] **Step 6: 实现 resolve_methodology_extension / prompt_description / render_skill_guidance**

在 `src-tauri/src/prompts/registry.rs` 的 `prompt_display_name`（194-199）之后新增：

```rust
/// 按 id 取内置提示词描述（未知 id 返回 None）。
pub fn prompt_description(prompt_id: &str) -> Option<String> {
    get_builtin_prompts()
        .get(prompt_id)
        .map(|e| e.description.clone())
}
```

在 `src-tauri/src/creative_engine/write_time_bundle.rs` 的工具函数区（`format_writing_strategy_constraints` 之前，927 行「工具函数」注释之后）新增：

```rust
/// v0.31.0: 方法论扩展动态解析（推荐资产贯通 + 方法论动态化）。
///
/// 解析顺序：
/// 1. `methodology_{id}_step{N}`（step 变体，如雪花法 10 步）
/// 2. `methodology_{id}`（无 step 后缀的单文件，如英雄之旅）
/// 3. 兼容旧命名：hdwb 系列 4 个阶段文件（seed/expansion/convergence/iteration）
///    未按 step 规范命名，按步数映射到既有文件
///
/// 均未命中：记 `log::warn!`（不再静默丢弃）并返回 None。
/// 新增方法论 = 向 `resources/prompts/methodology/` 丢一个
/// `methodology_{id}.md`（可选 `methodology_{id}_step{N}.md`），无需改代码。
pub fn resolve_methodology_extension(methodology_id: &str, step: i32) -> Option<String> {
    let mid = crate::domain::methodology::normalize_methodology_id(methodology_id);
    let step = step.max(1);

    let step_id = format!("methodology_{}_step{}", mid, step);
    if let Some(content) = crate::prompts::registry::resolve_prompt_default(&step_id) {
        let label = crate::prompts::registry::prompt_display_name(&step_id);
        return Some(format!("【创作方法论（{}）】\n{}", label, content));
    }

    let base_id = format!("methodology_{}", mid);
    if let Some(content) = crate::prompts::registry::resolve_prompt_default(&base_id) {
        let label = crate::prompts::registry::prompt_display_name(&base_id);
        return Some(format!("【创作方法论（{}）】\n{}", label, content));
    }

    // 兼容旧命名：hdwb 的 4 个阶段文件（step 1=seed / 2=expansion / 3=convergence / 4=iteration）
    if mid == "high_density_world_building" {
        let legacy_id = match step {
            2 => "methodology_hdwb_expansion",
            3 => "methodology_hdwb_convergence",
            4 => "methodology_hdwb_iteration",
            _ => "methodology_hdwb_seed",
        };
        if let Some(content) = crate::prompts::registry::resolve_prompt_default(legacy_id) {
            let label = crate::prompts::registry::prompt_display_name(legacy_id);
            return Some(format!("【创作方法论（{}）】\n{}", label, content));
        }
    }

    log::warn!(
        "[WriteTimeBundle] 未知方法论 ID '{}'（step {}），跳过方法论注入",
        mid,
        step
    );
    None
}
```

在 `src-tauri/src/agents/service.rs` 的 `render_writer_system_from_bundle`（3301-3379）之后新增：

```rust
/// v0.31.0: 推荐技能提示词摘要——注入 writer system_prompt。
///
/// 第一阶段为纯文本注入：按 skill id（兼容 `builtin.` 前缀）解析
/// `skill_{id}` 提示词的名称与描述，生成「激活写作技能」段落。
/// 未知 id 跳过（记 debug）；全部未知返回 None。
pub fn render_skill_guidance(skill_ids: &[String]) -> Option<String> {
    let mut lines: Vec<String> = Vec::new();
    for raw in skill_ids.iter().take(3) {
        let id = raw.strip_prefix("builtin.").unwrap_or(raw.as_str());
        let prompt_id = format!("skill_{}", id);
        if let Some(description) = crate::prompts::registry::prompt_description(&prompt_id) {
            let name = crate::prompts::registry::prompt_display_name(&prompt_id);
            lines.push(format!("- {}：{}", name, description));
        } else {
            log::debug!("[render_skill_guidance] 未找到技能提示词 {}", prompt_id);
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(format!(
            "【激活写作技能（本次写作请主动运用以下技能手法）】\n{}",
            lines.join("\n")
        ))
    }
}
```

- [ ] **Step 7: TimeSliced 路径消费推荐参数**

在 `src-tauri/src/agents/orchestrator.rs` 的 `execute_time_sliced` 中，叙事四元组注入块（955-962，`bundle.narrative_quartet` 赋值）之后、`writing_strategy_constraints` 覆盖块（964 行注释）之前插入：

```rust
        // v0.31.0: 推荐资产贯通——任务参数中的推荐方法论/风格 DNA 优先于
        // story 字段的加载结果（build_selected_strategy 保证 story 有显式值时
        // 推荐值即显式值，不覆盖用户选择）。
        if let Some(mid) = task
            .parameters
            .get("recommended_methodology_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
        {
            let step = task
                .context
                .world
                .methodology_step
                .as_deref()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(1);
            if let Some(ext) =
                crate::creative_engine::write_time_bundle::resolve_methodology_extension(&mid, step)
            {
                bundle.methodology_extension = Some(ext);
            }
        }
        // 推荐风格 DNA 兜底：story 未配置 style_dna_id（bundle 未加载出六维扩展）时，
        // 用推荐的第一个 DNA 加载扩展。
        if bundle.style_dna_extension.is_none() {
            let recommended_dna: Option<String> = task
                .parameters
                .get("recommended_style_dna_ids")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if let Some(dna_id) = recommended_dna {
                let dna_repo = crate::db::StyleDnaRepository::new(pool.inner().clone());
                match dna_repo.get_by_id(&dna_id) {
                    Ok(Some(dna)) => {
                        match serde_json::from_str::<crate::domain::style::StyleDNA>(&dna.dna_json)
                        {
                            Ok(dna_obj) => {
                                bundle.style_dna_extension = Some(dna_obj.to_prompt_extension());
                            }
                            Err(e) => {
                                log::warn!("[TimeSliced] 推荐 StyleDNA 解析失败: {}", e);
                            }
                        }
                    }
                    _ => {
                        log::debug!("[TimeSliced] 推荐 StyleDNA {} 不存在，跳过", dna_id);
                    }
                }
            }
        }
```

在 writer system prompt 渲染（1079-1083，`render_writer_system_from_bundle` 调用）之后插入技能摘要注入：

```rust
        // v0.31.0: 推荐技能摘要注入 writer system prompt（文本注入第一阶段）
        let recommended_skills: Vec<String> = task
            .parameters
            .get("recommended_skill_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        if let Some(guidance) = crate::agents::service::render_skill_guidance(&recommended_skills) {
            writer_system_prompt = Some(match writer_system_prompt {
                Some(p) => format!("{}\n\n{}", p, guidance),
                None => guidance,
            });
        }
```

（1079 行的 `let writer_system_prompt =` 需改为 `let mut writer_system_prompt =`。）

- [ ] **Step 8: 写推荐方法论写回/不覆盖的失败测试**

在 `src-tauri/src/commands/orchestrator.rs` 的 `mod tests`（1361 行起）追加：

```rust
    fn create_story_with_methodology(pool: &crate::db::DbPool, mid: Option<&str>) -> Story {
        StoryRepository::new(pool.clone())
            .create(crate::db::CreateStoryRequest {
                title: "写回测试故事".to_string(),
                description: None,
                genre: Some("玄幻".to_string()),
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: mid.map(|s| s.to_string()),
                reference_book_id: None,
            })
            .expect("create story")
    }

    #[test]
    fn test_persist_recommended_methodology_writes_back_when_story_has_none() {
        let pool = create_test_pool().expect("test pool");
        let story = create_story_with_methodology(&pool, None);
        let mut strategy = crate::domain::strategy::SelectedStrategy::default();
        strategy.methodology_id = Some("snowflake".to_string());

        let written = persist_recommended_methodology(&pool, &Some(story.clone()), &Some(strategy));
        assert!(written, "story 无显式方法论且推荐存在时应写回");

        let reloaded = StoryRepository::new(pool.clone())
            .get_by_id(&story.id)
            .expect("get")
            .expect("story exists");
        assert_eq!(reloaded.methodology_id.as_deref(), Some("snowflake"));
    }

    #[test]
    fn test_persist_recommended_methodology_never_overrides_explicit() {
        let pool = create_test_pool().expect("test pool");
        let story = create_story_with_methodology(&pool, Some("hero_journey"));
        let mut strategy = crate::domain::strategy::SelectedStrategy::default();
        strategy.methodology_id = Some("snowflake".to_string());

        let written = persist_recommended_methodology(&pool, &Some(story.clone()), &Some(strategy));
        assert!(!written, "story 有显式方法论时不得覆盖");

        let reloaded = StoryRepository::new(pool.clone())
            .get_by_id(&story.id)
            .expect("get")
            .expect("story exists");
        assert_eq!(reloaded.methodology_id.as_deref(), Some("hero_journey"));
    }
```

- [ ] **Step 9: 跑测试确认失败**

```
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib persist_recommended_methodology
```

预期：编译失败，`cannot find function persist_recommended_methodology in this scope`。

- [ ] **Step 10: 实现写回函数并接线 smart_execute_inner**

在 `src-tauri/src/commands/orchestrator.rs` 的 `build_selected_strategy`（1188-1329）之后新增：

```rust
/// v0.31.0: 推荐方法论写回——story 无显式 methodology_id 且推荐存在时落库，
/// 后续续写与 WriteTimeBundle 加载可直接读取。有显式值（用户选择或此前写回）
/// 不覆盖。返回是否发生了写回。
fn persist_recommended_methodology(
    pool: &crate::db::DbPool,
    current_story: &Option<crate::db::Story>,
    selected_strategy: &Option<crate::domain::strategy::SelectedStrategy>,
) -> bool {
    let story = match current_story {
        Some(s) => s,
        None => return false,
    };
    if story
        .methodology_id
        .as_deref()
        .map(|m| !m.trim().is_empty())
        .unwrap_or(false)
    {
        return false; // 显式值不覆盖
    }
    let recommended = match selected_strategy
        .as_ref()
        .and_then(|s| s.methodology_id.as_deref())
    {
        Some(m) if !m.trim().is_empty() => m,
        _ => return false,
    };
    let req = crate::db::UpdateStoryRequest {
        title: None,
        description: None,
        genre: None,
        tone: None,
        pacing: None,
        style_dna_id: None,
        genre_profile_id: None,
        methodology_id: Some(recommended.to_string()),
        methodology_step: None,
        reference_book_id: None,
    };
    match crate::db::StoryRepository::new(pool.clone()).update(&story.id, &req) {
        Ok(_) => {
            log::info!(
                "[smart_execute] 推荐方法论写回: story={} methodology={}",
                story.id,
                recommended
            );
            true
        }
        Err(e) => {
            log::warn!("[smart_execute] 推荐方法论写回失败: {}", e);
            false
        }
    }
}
```

在 `smart_execute_inner` 的 `selected_strategy` 计算（740-744）之后、`plan_context` 构造（746）之前插入：

```rust
    // v0.31.0: 推荐方法论写回（story 无显式值才落库，显式值不覆盖）
    {
        let wb_pool = pool.clone();
        let wb_story = current_story.clone();
        let wb_strategy = selected_strategy.clone();
        let _ = tokio::task::spawn_blocking(move || {
            persist_recommended_methodology(&wb_pool, &wb_story, &wb_strategy);
        })
        .await;
    }
```

- [ ] **Step 11: 跑全部相关测试确认通过**

```
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib inject_recommended_strategy_params
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib resolve_methodology_extension
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib render_skill_guidance
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib persist_recommended_methodology
```

预期：4 组测试全部 `test result: ok`。

- [ ] **Step 12: 提交**

```
cd /Users/yuzaimu/projects/StoryForge && git add -A && git commit -m "feat(续写): 推荐方法论/风格DNA/技能透传writer参数并在无显式值时写回stories表"
```

---

## Task 6: 方法论动态解析 + 未知 ID warn

`write_time_bundle.rs:206-250` 的硬编码 5-ID match 改为走 Task 5 已落地的
`resolve_methodology_extension` 动态解析（先试 `methodology_{id}_step{N}` 再试 `methodology_{id}`，
hdwb 旧命名兼容映射，未知 ID `log::warn!` 并跳过），保留对现有 5 个方法论的兼容。

**Files:**
- Modify: `src-tauri/src/creative_engine/write_time_bundle.rs`（`load_sync` 内方法论加载块 205-250；测试在 944 的 `mod tests`）
- 依赖（Task 5 已建，本任务不改动）：`resolve_methodology_extension`（同文件工具函数区）、`resources/prompts/methodology/` 现有 17 个 md（命名已核对：snowflake step 变体 10 个、hdwb 4 个旧命名走兼容映射、3 个单文件走 base 回退，无需重命名/新增文件）

**Interfaces:**
- Consumes:
  - `pub fn resolve_methodology_extension(methodology_id: &str, step: i32) -> Option<String>`（Task 5 产出）
  - `crate::domain::methodology::normalize_methodology_id(id: &str) -> &str`（`domain/methodology.rs:66-71`）
  - `story.methodology_id: Option<String>` / `story.methodology_step: Option<i32>`（`db/models.rs:1103-1104`）
- Produces:
  - `load_sync` 内 `methodology_extension: Option<String>` 的构建方式变更（签名不变）；行为差异：标签从硬编码中文（如 `"雪花法 第{}步"`）改为 md frontmatter 的 `name`（如 `雪花法第3步：角色概要`），未知 ID 从静默 None 改为 `log::warn!` + None。

### Steps

- [ ] **Step 1: 写失败测试（动态解析命中 / step 变体 / 未知 ID）**

在 `src-tauri/src/creative_engine/write_time_bundle.rs` 的 `mod tests` 追加（Step 变体命中与 hdwb alias 已在 Task 5 覆盖，这里补 base 回退、未知 ID warn 路径与 load_sync 集成断言）：

```rust
    #[test]
    fn test_resolve_methodology_base_file_fallback() {
        // hero_journey 无 step 变体文件，应回退 methodology_hero_journey 单文件
        let ext = resolve_methodology_extension("hero_journey", 5)
            .expect("hero_journey 应回退到无 step 后缀的单文件");
        assert!(ext.contains("英雄之旅"));
    }

    #[test]
    fn test_load_sync_dynamic_methodology_resolution() {
        // 集成断言：load_sync 走动态解析后，snowflake step2 的扩展来自 md 文件
        let pool = crate::db::create_test_pool().expect("test pool");
        let story = crate::db::StoryRepository::new(pool.clone())
            .create(crate::db::CreateStoryRequest {
                title: "动态解析测试".to_string(),
                description: None,
                genre: Some("玄幻".to_string()),
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: Some("snowflake".to_string()),
                reference_book_id: None,
            })
            .expect("create story");
        crate::db::StoryRepository::new(pool.clone())
            .update(
                &story.id,
                &crate::db::UpdateStoryRequest {
                    title: None,
                    description: None,
                    genre: None,
                    tone: None,
                    pacing: None,
                    style_dna_id: None,
                    genre_profile_id: None,
                    methodology_id: None,
                    methodology_step: Some(2),
                    reference_book_id: None,
                },
            )
            .expect("set step 2");

        let bundle = WriteTimeBundle::load_sync(&pool, &story.id, 1, None, None, None)
            .expect("load_sync 应成功（各资产缺失均为软降级）");
        let ext = bundle
            .methodology_extension
            .expect("snowflake step2 应解析出方法论扩展");
        assert!(
            ext.contains("雪花法第2步"),
            "标签应来自 md frontmatter name: {}",
            ext
        );
    }

    #[test]
    fn test_load_sync_unknown_methodology_warns_and_skips() {
        // 未知 ID：log::warn! 记录（测试无法直接断言日志），行为断言为跳过注入返回 None
        let pool = crate::db::create_test_pool().expect("test pool");
        let story = crate::db::StoryRepository::new(pool.clone())
            .create(crate::db::CreateStoryRequest {
                title: "未知方法论测试".to_string(),
                description: None,
                genre: Some("玄幻".to_string()),
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: Some("totally_unknown_methodology".to_string()),
                reference_book_id: None,
            })
            .expect("create story");

        let bundle = WriteTimeBundle::load_sync(&pool, &story.id, 1, None, None, None)
            .expect("未知方法论不应导致加载失败");
        assert!(
            bundle.methodology_extension.is_none(),
            "未知方法论 ID 应跳过注入"
        );
    }
```

- [ ] **Step 2: 跑测试确认失败**

```
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib test_resolve_methodology_base_file_fallback
```

预期：base 回退测试在 Task 5 实现后**已通过**（动态解析本身已就绪）；两个 `load_sync` 集成测试在 rewiring 前也大概率通过——因为旧硬编码 match 对 snowflake step2 同样产出扩展。因此本任务「失败先行」的体现是标签断言：

```
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib test_load_sync_dynamic_methodology_resolution
```

预期：`assertion failed: ext.contains("雪花法第2步")` —— 旧代码标签是硬编码 `雪花法 第2步`（含空格、无副标题），新标签来自 md frontmatter `雪花法第2步：…`。此断言失败即证明 load_sync 仍在走旧硬编码路径。

- [ ] **Step 3: 实现——load_sync 硬编码 match 改为动态解析**

将 `write_time_bundle.rs` 205-250 行的整个 `// v0.22.0: 加载方法论扩展` 块替换为：

```rust
        // v0.31.0: 加载方法论扩展——动态解析（Task 6）。
        // 先试 methodology_{id}_step{N}，再试 methodology_{id}，hdwb 旧命名
        // 走兼容映射；未知 ID 在 resolve_methodology_extension 内 log::warn! 并返回 None。
        let methodology_extension = match story.methodology_id.as_deref() {
            Some(mid) if !mid.is_empty() => {
                let step = story.methodology_step.unwrap_or(1);
                resolve_methodology_extension(mid, step)
            }
            _ => None,
        };
```

注意：`resolve_methodology_extension` 内部已做 `normalize_methodology_id`，此处不再重复归一化。旧块中的 5-ID match、snowflake/hdwb 特判、空 prompt_id 守卫全部删除（由动态解析与 warn 覆盖）。

- [ ] **Step 4: 跑测试确认通过**

```
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib write_time_bundle
```

预期：`test result: ok`，含 Task 5/6 全部方法论解析测试与既有 `genre_category_*` 测试。

再跑全量回归（标签变化可能影响其他断言）：

```
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib
```

预期：全绿。已知不受影响的相关方：`agents/service.rs:2567-2580` 与 `3514-3523` 是 Full 路径/框架映射的独立硬编码（不在设计第二节范围，列为后续跟进项，见文末）；`prompts/registry.rs:655-656` 的注册测试只断言 md 已加载，不受本改动影响。

- [ ] **Step 5: 提交**

```
cd /Users/yuzaimu/projects/StoryForge && git add -A && git commit -m "refactor(方法论): load_sync硬编码5-ID匹配改为PromptRegistry动态解析,未知ID告警跳过"
```

---

## Task 7: methodology_step 自动推进

每完成一章 `methodology_step` +1，到该方法论最大步数停留。最大步数采用
`domain/methodology.rs` 显式映射（比「从 PromptRegistry 数 step 文件」更简单、且不依赖
hdwb 旧命名的文件计数）；`MethodologySettings` 手动步进路径不受影响。

**Files:**
- Modify: `src-tauri/src/domain/methodology.rs`（`normalize_methodology_id`（66-71）之后新增两个函数；测试在 73 的 `mod methodology_id_tests`）
- Modify: `src-tauri/src/scene_commands.rs`（`update_scene`（199-302）：闭包捕获更新前内容状态 + 完成后触发推进；新增 `advance_methodology_step` 辅助函数；文件末尾新增 `#[cfg(test)] mod tests`）

**Interfaces:**
- Consumes:
  - `crate::db::SceneRepository::get_by_id` / `Scene { content: Option<String>, draft_content: Option<String>, story_id, .. }`（`db/models.rs:23-53`）
  - `crate::db::StoryRepository::{get_by_id, update}` + `crate::db::UpdateStoryRequest`
  - `crate::domain::methodology::normalize_methodology_id`
  - 章节完成检测钩子：`update_scene`（`scene_commands.rs:199`）——前端 `appendAiContent` → `persistSceneContent`（`src-frontend/src/frontstage/FrontstageApp.tsx:1171`）序列化调用此命令写入正文；「正文（或草稿）从无到有」即本章完成事件。
- Produces:
  - `pub fn methodology_max_steps(id: &str) -> i32`（domain/methodology.rs）
  - `pub fn next_methodology_step(id: &str, current: i32) -> i32`（domain/methodology.rs）
  - `fn advance_methodology_step(pool: &DbPool, story_id: &str) -> bool`（scene_commands.rs）

### Steps

- [ ] **Step 1: 写失败测试（步进 + 到顶停留 + 无方法论 noop）**

在 `src-tauri/src/domain/methodology.rs` 的 `mod methodology_id_tests`（73 行起）追加：

```rust
    #[test]
    fn methodology_step_advances_and_caps() {
        assert_eq!(next_methodology_step("snowflake", 1), 2);
        assert_eq!(next_methodology_step("snowflake", 9), 10);
        // 到顶停留
        assert_eq!(next_methodology_step("snowflake", 10), 10);
        // hdwb 别名归一化后 cap=4
        assert_eq!(next_methodology_step("hdwb", 3), 4);
        assert_eq!(next_methodology_step("high_density_world_building", 4), 4);
        // 单文件方法论无步骤概念，永不推进
        assert_eq!(next_methodology_step("hero_journey", 1), 1);
        // NULL/非法当前值按 1 处理
        assert_eq!(next_methodology_step("snowflake", 0), 2);
    }
```

在 `src-tauri/src/scene_commands.rs` 文件末尾新增：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_story(pool: &crate::db::DbPool, methodology_id: Option<&str>) -> crate::db::Story {
        crate::db::StoryRepository::new(pool.clone())
            .create(crate::db::CreateStoryRequest {
                title: "步进测试故事".to_string(),
                description: None,
                genre: Some("玄幻".to_string()),
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: methodology_id.map(|s| s.to_string()),
                reference_book_id: None,
            })
            .expect("create story")
    }

    fn update_step_req(step: i32) -> crate::db::UpdateStoryRequest {
        crate::db::UpdateStoryRequest {
            title: None,
            description: None,
            genre: None,
            tone: None,
            pacing: None,
            style_dna_id: None,
            genre_profile_id: None,
            methodology_id: None,
            methodology_step: Some(step),
            reference_book_id: None,
        }
    }

    #[test]
    fn test_advance_methodology_step_on_chapter_complete() {
        let pool = crate::db::create_test_pool().expect("test pool");
        let story = create_story(&pool, Some("snowflake"));

        assert!(advance_methodology_step(&pool, &story.id));
        let reloaded = crate::db::StoryRepository::new(pool.clone())
            .get_by_id(&story.id)
            .expect("get")
            .expect("story exists");
        assert_eq!(reloaded.methodology_step, Some(2));
    }

    #[test]
    fn test_advance_methodology_step_stays_at_max() {
        let pool = crate::db::create_test_pool().expect("test pool");
        let story = create_story(&pool, Some("snowflake"));
        crate::db::StoryRepository::new(pool.clone())
            .update(&story.id, &update_step_req(10))
            .expect("set step 10");

        assert!(!advance_methodology_step(&pool, &story.id), "到顶应停留");
        let reloaded = crate::db::StoryRepository::new(pool.clone())
            .get_by_id(&story.id)
            .expect("get")
            .expect("story exists");
        assert_eq!(reloaded.methodology_step, Some(10));
    }

    #[test]
    fn test_advance_methodology_step_noop_without_methodology() {
        let pool = crate::db::create_test_pool().expect("test pool");
        let story = create_story(&pool, None);
        assert!(!advance_methodology_step(&pool, &story.id));
        let reloaded = crate::db::StoryRepository::new(pool.clone())
            .get_by_id(&story.id)
            .expect("get")
            .expect("story exists");
        assert_eq!(reloaded.methodology_step, None);
    }
}
```

- [ ] **Step 2: 跑测试确认失败**

```
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib methodology_step
```

预期：编译失败，`cannot find function next_methodology_step` / `cannot find function advance_methodology_step`。

- [ ] **Step 3: 实现 domain 映射函数**

在 `src-tauri/src/domain/methodology.rs` 的 `normalize_methodology_id`（66-71）之后新增：

```rust
/// 各方法论的最大步数（章节完成自动推进到顶后停留）。
///
/// 与 `resources/prompts/methodology/` 下已注册的阶段文件数保持一致：
/// - snowflake：methodology_snowflake_step1..step10 → 10
/// - high_density_world_building：seed/expansion/convergence/iteration → 4
/// - 其余单文件方法论无步骤概念 → 1（永不推进）
pub fn methodology_max_steps(id: &str) -> i32 {
    match normalize_methodology_id(id) {
        "snowflake" => 10,
        "high_density_world_building" => 4,
        _ => 1,
    }
}

/// 章节完成后的下一步号：+1，到 `methodology_max_steps` 停留。
pub fn next_methodology_step(id: &str, current: i32) -> i32 {
    let max = methodology_max_steps(id);
    (current.max(1) + 1).min(max)
}
```

- [ ] **Step 4: 实现 advance_methodology_step 并接线 update_scene**

在 `src-tauri/src/scene_commands.rs` 的 `update_scene`（199-302）之前新增辅助函数：

```rust
/// v0.31.0: 章节完成后推进 methodology_step（到该方法论最大步数停留）。
/// 返回是否发生了推进。story 无方法论或已到顶时为 noop。
fn advance_methodology_step(pool: &crate::db::DbPool, story_id: &str) -> bool {
    let story_repo = crate::db::StoryRepository::new(pool.clone());
    let story = match story_repo.get_by_id(story_id) {
        Ok(Some(s)) => s,
        _ => return false,
    };
    let mid = match story.methodology_id.as_deref() {
        Some(m) if !m.trim().is_empty() => m,
        _ => return false,
    };
    let current = story.methodology_step.unwrap_or(1);
    let next = crate::domain::methodology::next_methodology_step(mid, current);
    if next == current {
        return false;
    }
    let req = crate::db::UpdateStoryRequest {
        title: None,
        description: None,
        genre: None,
        tone: None,
        pacing: None,
        style_dna_id: None,
        genre_profile_id: None,
        methodology_id: None,
        methodology_step: Some(next),
        reference_book_id: None,
    };
    match story_repo.update(story_id, &req) {
        Ok(_) => {
            log::info!(
                "[update_scene] 章节完成，方法论 {} step {} -> {}",
                mid,
                current,
                next
            );
            true
        }
        Err(e) => {
            log::warn!("[update_scene] 推进 methodology_step 失败: {}", e);
            false
        }
    }
}
```

改造 `update_scene` 的 spawn_blocking 闭包（216-230），捕获更新前内容状态：

```rust
    let (result, story_id_opt, had_content_before) =
        tokio::task::spawn_blocking(move || -> Result<(usize, Option<String>, bool), AppError> {
            let repo = SceneRepository::new(pool_clone);
            // 获取 story_id 用于同步事件（P0-3 修复: 避免 unwrap_or_default 导致空字符串）
            let prior_scene = repo.get_by_id(&scene_id_clone).ok().flatten();
            let story_id_opt = prior_scene.as_ref().map(|s| s.story_id.clone());
            // v0.31.0: 章节完成检测——正文（或草稿）从无到有视为本章完成
            let had_content_before = prior_scene
                .as_ref()
                .map(|s| s.content.is_some() || s.draft_content.is_some())
                .unwrap_or(true);
            let result = repo.update(&scene_id_clone, &updates_clone).map_err(|e| {
                log::error!("[story_commands] {} failed: {}", "update_scene", e);
                AppError::from(e)
            })?;
            Ok((result, story_id_opt, had_content_before))
        })
        .await
        .map_err(|e| {
            AppError::from(format!("[update_scene] spawn_blocking join error: {}", e))
        })??;
```

在 `if let Some(ref story_id) = story_id_opt {` 块（249-300）结束之后、`Ok(result)`（301）之前插入：

```rust
    // v0.31.0: 章节完成（正文从无到有）→ methodology_step +1（到顶停留）
    if updates.content.is_some() && !had_content_before {
        if let Some(ref story_id) = story_id_opt {
            let pool_step = pool.inner().clone();
            let story_id_step = story_id.clone();
            let _ = tokio::task::spawn_blocking(move || {
                advance_methodology_step(&pool_step, &story_id_step);
            })
            .await;
        }
    }
```

（原闭包返回值元组从 2 元改为 3 元，闭包内 `Ok((result, story_id_opt))` 同步改为 `Ok((result, story_id_opt, had_content_before))`；`had_content_before` 默认 `true` 保证查不到旧场景时不误推进。）

- [ ] **Step 5: 跑测试确认通过**

```
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib methodology_step
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib scene_commands
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib methodology
```

预期：全部 `test result: ok`（含 domain 纯函数与 scene_commands 的 3 个 DB 测试）。

- [ ] **Step 6: 提交**

```
cd /Users/yuzaimu/projects/StoryForge && git add -A && git commit -m "feat(方法论): 章节完成自动推进methodology_step,到该方法论最大步数停留"
```

---

## Task 8: 扩张性写作合约 + 字数配置

writer prompt 体系从「收敛导向」改为阶段感知的扩张-收敛平衡；续写目标字数改为
AppConfig 配置项 `continuation_target_words`（默认 2000），模板按 0.7x–1.3x 动态渲染；
新增 `plan_mode`（默认 `"beat"`，供后续 Task 的计划结构重构使用）。

**Files:**
- Modify: `resources/prompts/writer/writer_system.md`（全文改造，见 Step 4 完整内容）
- Modify: `resources/prompts/writer/orchestrator_timesliced_writer.md`（全文改造，见 Step 4 完整内容）
- Modify: `src-tauri/src/creative_engine/write_time_bundle.rs`（`to_prompt` 故事大纲段 598-603；工具函数区新增 `new_character_policy_text`；测试在 944 的 `mod tests`）
- Modify: `src-tauri/src/config/settings.rs`（AppConfig 字段 337-338 之后、默认函数区 361 之后、Default impl 1076 之后、访问器 1446 附近；文件末尾新增 `#[cfg(test)] mod tests`）
- Modify: `src-tauri/src/agents/orchestrator.rs`（`execute_time_sliced` 的 AppConfig 加载块 968-990、模板 vars 1025-1028、硬编码 fallback 1031-1038；测试在 4419 的 `mod tests`）

**Interfaces:**
- Consumes:
  - `crate::prompts::engine::TemplateEngine::render_with_conditions(&template, &vars)`（`{{var}}` 双花括号语法）
  - `CreativeAssetSnapshot::narrative_phase_guidance()` → `Option<String>`，文案含「铺垫期/上升期/高潮期/收尾期/冲突激化期」（`creative_engine/asset_snapshot.rs:142-202`）
  - `AppConfig::load(&app_dir)`（`execute_time_sliced` 已在 968-990 加载，就地扩展）
- Produces:
  - `AppConfig.continuation_target_words: u32`（serde default 2000）
  - `AppConfig.plan_mode: String`（serde default `"beat"`）
  - `pub fn AppConfig::continuation_target_words_range(&self) -> (u32, u32)`（0.7x–1.3x）
  - `pub(crate) fn new_character_policy_text(narrative_phase_guidance: Option<&str>) -> &'static str`（write_time_bundle.rs）
  - 模板 `orchestrator_timesliced_writer` 新变量 `{{target_words_range}}`

### Steps

- [ ] **Step 1: 写配置缺省值与字数范围的失败测试**

在 `src-tauri/src/config/settings.rs` 文件末尾新增：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// v0.31.0: 旧配置（无新字段）反序列化时 serde default 兜底。
    /// 注意：AppConfig.llm 无 serde default，测试 JSON 必须带完整 llm 对象。
    #[test]
    fn test_continuation_target_words_and_plan_mode_defaults() {
        let json = r#"{
            "llm": {"provider": "openai", "api_key": "", "model": "gpt-4",
                    "api_base": null, "max_tokens": 2500, "temperature": 0.8}
        }"#;
        let cfg: AppConfig = serde_json::from_str(json).expect("旧配置应可反序列化");
        assert_eq!(cfg.continuation_target_words, 2000);
        assert_eq!(cfg.plan_mode, "beat");
    }

    #[test]
    fn test_continuation_target_words_range_render() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.continuation_target_words_range(), (1400, 2600));

        let json = r#"{
            "llm": {"provider": "openai", "api_key": "", "model": "gpt-4",
                    "api_base": null, "max_tokens": 2500, "temperature": 0.8},
            "continuation_target_words": 1000,
            "plan_mode": "single_writer"
        }"#;
        let cfg: AppConfig = serde_json::from_str(json).expect("新字段应可配置");
        assert_eq!(cfg.continuation_target_words_range(), (700, 1300));
        assert_eq!(cfg.plan_mode, "single_writer");
    }
}
```

- [ ] **Step 2: 写阶段感知新角色策略的失败测试**

在 `src-tauri/src/creative_engine/write_time_bundle.rs` 的 `mod tests` 追加：

```rust
    #[test]
    fn test_new_character_policy_phase_aware() {
        // 发展期：允许有叙事功能的新角色
        let dev = new_character_policy_text(Some("当前叙事阶段：上升期。请逐步升级冲突……"));
        assert!(dev.contains("允许引入具有明确叙事功能的新角色"));
        // 高潮期：收敛
        let climax = new_character_policy_text(Some("当前叙事阶段：高潮期。请保持紧张节奏……"));
        assert!(climax.contains("不引入新的重要角色"));
        // 冲突激化期同样收敛
        assert!(
            new_character_policy_text(Some("当前叙事阶段：冲突激化期。")).contains("不引入新的重要角色")
        );
        // 无阶段信息：默认发展期（扩张取向，替代旧「禁止自创新角色」一刀切）
        assert!(new_character_policy_text(None).contains("允许引入"));
    }
```

在 `src-tauri/src/agents/orchestrator.rs` 的 `mod tests`（4419 行起）追加：

```rust
    #[test]
    fn test_timesliced_writer_template_renders_target_words_range() {
        let tpl =
            crate::prompts::registry::resolve_prompt_default("orchestrator_timesliced_writer")
                .expect("timesliced writer template");
        let mut vars = std::collections::HashMap::new();
        vars.insert("context".to_string(), "CTX".to_string());
        vars.insert("instruction".to_string(), "INS".to_string());
        vars.insert("continuation".to_string(), "CONT".to_string());
        vars.insert("target_words_range".to_string(), "1400-2600".to_string());
        let rendered =
            crate::prompts::engine::TemplateEngine::render_with_conditions(&tpl, &vars);
        assert!(
            rendered.contains("1400-2600字"),
            "字数范围应渲染进模板: {}",
            rendered
        );
        assert!(!rendered.contains("800-1500"), "旧硬编码字数应已移除");
    }
```

- [ ] **Step 3: 跑测试确认失败**

```
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib continuation_target_words
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib new_character_policy
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib test_timesliced_writer_template_renders_target_words_range
```

预期：前两条编译失败（字段/函数不存在）；模板测试编译通过但断言失败——`rendered.contains("1400-2600字")` 不成立（旧模板是硬编码「800-1500字」且无 `{{target_words_range}}` 变量）。

- [ ] **Step 4: 改造两个 writer prompt md（完整内容）**

`resources/prompts/writer/writer_system.md` 完整替换为：

```markdown
---
id: writer_system
name: "Writer 系统提示词"
description: "AI 写作助手的基础角色设定与行为准则"
category: writer
version: 0.31.0
variables:
  - story_title
  - genre
  - tone
  - pacing
  - characters
  - previous_chapters
  - narrative_structure
  - current_content
  - instruction
  - world_rules
  - scene_structure
  - outline_context
  - story_description
---

你是一位专业的小说创作助手，擅长中文写作。

你的任务是根据提供的故事上下文和指令，续写或改写小说内容。

核心要求：
1. 使用中文（简体中文）写作
2. 保持角色声音一致性——每个角色的用词习惯、语气、句式结构符合其性格
3. 展示而非讲述——用动作、对话、细节描写传达情感，避免直接陈述
4. 对话必须推动情节或揭示性格，禁止无意义闲聊
5. 每个场景结尾留下钩子（悬念、新问题、新威胁）
6. 遵循提供的世界观规则和设定约束
7. 保持与已有情节的连贯性，不引入与设定矛盾的新元素
8. 剧情必须向前推进--每个场景都要推进到故事大纲的下一节点，不得原地踏步、不得仅复述设定或重复前文已有情节

扩张-收敛平衡准则（依据上下文给出的当前叙事阶段执行）：
- 开篇/发展期（铺垫期、上升期）：鼓励引入具有明确叙事功能的新角色——新角色必须推动冲突、揭示世界观或制造转折，不得是路人式点缀；允许在符合大纲的前提下切换场景、开辟新的冲突线，推动冲突升级
- 高潮期（含冲突激化期）：聚焦既有角色与既有冲突的集中爆发，不再引入新的重要角色，把已铺设的张力推向顶点
- 收尾期：全面收敛——回收伏笔、收束人物弧线、解决剩余悬念，不开启新的情节线
- 底线（任何阶段）：新角色必须有明确的叙事功能并能在后续情节中持续发挥作用；不得违反世界观红线（MASTER_SETTING）与故事大纲；保持与已有设定的一致性

写作风格：
- 根据指定的题材和基调调整语言风格
- 环境描写服务于氛围营造，不过度铺陈
- 内心独白适度，主要用于揭示角色动机和冲突
- 节奏控制：紧张场景用短句、快节奏；抒情场景允许长句和细腻描写

输出要求：
- 只输出小说正文，不要添加解释、总结或元评论
- 不要输出你的思考过程、分析、规划或元评论--直接从小说正文第一句开始
- 禁止以"这是一个..."、"让我..."、"我需要..."、"根据要求"等分析性语句开头
- 不要输出"以下是续写内容"等过渡语
- 保持与已有文本的自然衔接
- 禁止重复输出：同一段落、同一句子不得在文中出现两次
```

`resources/prompts/writer/orchestrator_timesliced_writer.md` 完整替换为：

```markdown
---
id: orchestrator_timesliced_writer
name: "TimeSliced Writer 正文生成"
description: "AgentOrchestrator：时分模式下单次 Writer 正文生成（目标字数由配置渲染）"
category: writer
version: 0.31.0
variables:
  - context
  - instruction
  - continuation
  - target_words_range
---

你是一名专业的小说作者。请根据以下设定写一段正文（{{target_words_range}}字）。

故事上下文：
{{context}}

{{continuation}}

写作指令：
{{instruction}}

要求：
1. 只输出小说正文
2. 保持与已有内容的自然衔接
3. 符合角色性格和世界观设定
4. 剧情必须向前推进到故事大纲的下一节点，不得原地踏步、不得仅复述设定或重复前文
5. 写作指令须与故事上下文中的世界观、故事大纲、场景大纲协调一致；若指令与上下文冲突，在遵循上下文硬约束的前提下落实指令核心意图
6. 直接输出正文，不要输出思考过程、分析或规划--禁止以"这是一个..."、"让我..."、"我需要..."等分析性语句开头
7. 扩张-收敛平衡：依据上下文给出的当前叙事阶段执行——开篇/发展期鼓励引入有叙事功能的新角色、允许场景切换、推动冲突升级；高潮期聚焦冲突爆发；收尾期收束伏笔与弧线。任何阶段新角色都必须有明确叙事功能，且不得违反世界观红线（MASTER_SETTING）
```

- [ ] **Step 5: 实现 AppConfig 新字段与字数范围**

`src-tauri/src/config/settings.rs`：

(a) AppConfig 字段区，在 `writer_max_tokens`（337-338）之后插入：

```rust
    /// v0.31.0: 续写单次目标字数（中文「字」，默认 2000）。
    /// `orchestrator_timesliced_writer` 模板按 0.7x-1.3x 渲染目标字数范围。
    #[serde(default = "default_continuation_target_words")]
    pub continuation_target_words: u32,
    /// v0.31.0: 续写计划模式 — `beat`（默认，beat 驱动多步计划）或
    /// `single_writer`（旧单 writer 步回退开关）。后续计划结构重构任务消费。
    #[serde(default = "default_plan_mode")]
    pub plan_mode: String,
```

(b) 默认函数区（`default_generation_mode`（361-363）之后）插入：

```rust
fn default_continuation_target_words() -> u32 {
    2000
}

fn default_plan_mode() -> String {
    "beat".to_string()
}
```

(c) Default impl（`writer_max_tokens: default_writer_max_tokens(),`（1076）之后）插入：

```rust
            continuation_target_words: default_continuation_target_words(),
            plan_mode: default_plan_mode(),
```

(d) 访问器区（`continuation_temperature()`（1446-1448）之后）插入：

```rust
    /// v0.31.0: 续写目标字数范围（0.7x-1.3x），供 writer 模板渲染。
    pub fn continuation_target_words_range(&self) -> (u32, u32) {
        let t = self.continuation_target_words.max(100);
        (
            (t as f32 * 0.7).round() as u32,
            (t as f32 * 1.3).round() as u32,
        )
    }
```

- [ ] **Step 6: 实现阶段感知新角色策略并接入 to_prompt**

在 `src-tauri/src/creative_engine/write_time_bundle.rs` 工具函数区（`format_writing_strategy_constraints` 之前）新增：

```rust
/// v0.31.0: 阶段感知的新角色策略（替代旧「禁止自创新角色」一刀切文案）。
///
/// 依据【叙事阶段】指导文本判定扩张/收敛取向；底线恒定：新角色必须有
/// 明确叙事功能、不得违反世界观红线（MASTER_SETTING）。
pub(crate) fn new_character_policy_text(narrative_phase_guidance: Option<&str>) -> &'static str {
    const EXPANSION: &str = "若当前处于开篇/发展期，允许引入具有明确叙事功能的新角色（推动冲突、揭示世界观或制造转折），允许合理切换场景、推动冲突升级；新角色必须服务于本场景戏剧目标，不得违反上述世界观红线与故事大纲。";
    const CONVERGENCE: &str = "当前处于高潮/收尾期：聚焦既有角色与既有冲突的爆发与收束，不引入新的重要角色，优先回收伏笔、推进故事大纲的下一节点。";
    match narrative_phase_guidance {
        Some(g)
            if g.contains("高潮期") || g.contains("收尾期") || g.contains("冲突激化期") =>
        {
            CONVERGENCE
        }
        _ => EXPANSION,
    }
}
```

将 `to_prompt()` 的故事大纲段（598-603）替换为：

```rust
        if let Some(ref outline) = self.story_outline {
            sections.push(format!(
                "【故事大纲（本场景必须围绕此大纲展开，禁止偏离）】\n{}\n（若下方「本场景任务」与此大纲冲突，以本故事大纲为准。{}）",
                outline,
                new_character_policy_text(self.narrative_phase_guidance.as_deref())
            ));
        }
```

- [ ] **Step 7: execute_time_sliced 渲染 target_words_range**

在 `src-tauri/src/agents/orchestrator.rs` 的 `execute_time_sliced` 中：

(a) AppConfig 加载块（968-990）之前声明默认值，并在 `Ok(cfg)` 分支内追加计算：

```rust
        // v0.31.0: 续写目标字数范围（默认 2000 → 1400-2600），供模板渲染
        let mut target_words_range = "1400-2600".to_string();
        match self.app_handle.path().app_data_dir() {
            Ok(app_dir) => match crate::config::AppConfig::load(&app_dir) {
                Ok(cfg) => {
                    bundle.writing_strategy_constraints = Some(
                            crate::creative_engine::write_time_bundle::format_writing_strategy_constraints(
                                &cfg.writing_strategy,
                            ),
                        );
                    let (lo, hi) = cfg.continuation_target_words_range();
                    target_words_range = format!("{}-{}", lo, hi);
                }
                // …… 两个 Err 分支保持原样 ……
```

（即仅在原 `Ok(cfg)` 分支末尾追加两行 `let (lo, hi) ...; target_words_range = ...;`，其余不变。）

(b) 模板 vars（1025-1028）追加：

```rust
            vars.insert("target_words_range".to_string(), target_words_range.clone());
```

(c) 硬编码 fallback（1031-1038）改为使用变量：

```rust
        } else {
            format!(
                "你是一名专业的小说作者。请根据以下设定写一段正文（{target_words_range}字）。\n\n\
                 {bundle_prompt}\n\n\
                 {continuation_ctx}\n\n\
                 【创作指令】\n{user_instruction}\n\n\
                 写作指令须与上述设定（世界观/故事大纲/场景大纲）协调一致；若冲突，在遵循设定硬约束的前提下落实指令核心意图。\n\n\
                 请直接输出正文，不要写说明、标题或分章标记。"
            )
        };
```

- [ ] **Step 8: 跑测试确认通过**

```
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib continuation_target_words
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib new_character_policy
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib test_timesliced_writer_template_renders_target_words_range
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib write_time_bundle
```

预期：全部 `test result: ok`。注意 `prompts/registry.rs` 的既有测试（如 `test_resolve_prompt_default` 断言 `writer_system` 含「小说创作助手」）不受影响——改造后的 md 保留了该句。

- [ ] **Step 9: 全量回归 + 提交**

```
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib
cd /Users/yuzaimu/projects/StoryForge && git add -A && git commit -m "feat(写作合约): writer提示词改阶段感知扩张-收敛准则,续写字数配置化并新增plan_mode"
```

预期：`cargo test --lib` 全绿（设计第七节回归门槛之一）。

---

## 跨任务说明与后续跟进项

- **执行顺序依赖**：Task 6 依赖 Task 5 产出的 `resolve_methodology_extension`；Task 7/8 相互独立，可按 5 → 6 → 7 → 8 顺序执行。全部完成后 `cargo test --lib` 全绿 + `npx tsc --noEmit` 通过（本 Part 不动前端，tsc 应为零变化）。
- **后续跟进项（不在本 Part）**：`agents/service.rs:2567-2580` 与 `3514-3523` 仍有 Full 路径/框架映射的独立硬编码方法论 ID 表，设计第二节未要求覆盖（默认路径是 TimeSliced）；如需统一，可在后续任务中复用 `resolve_methodology_extension`。
- **TriShot 路径**：Task 5 的推荐参数消费只接入了 TimeSliced（默认续写路径）；TriShot（`agents/orchestrator.rs:1772` 附近）同样加载 bundle，如需对齐可复用同一后处理代码，属增量改进。

---

# 第三部分：计划结构重构 + 创世链路修复（Task 9-12）

# StoryMoss 资产融合深度重构 — 实施计划（第 3 部分：Task 9–12）

> 设计文档：`docs/plans/2026-08-04-asset-fusion-deep-restructure-design.md` 第四节（计划结构重构）+ 第五节（创世链路修复 + 数据迁移）。
> 本部分所有代码均基于 2026-08-05 读到的真实当前源码改编。仓库约定：Rust 测试在 lib 内 `#[cfg(test)] mod tests`；提交信息用中文 conventional commit。
>
> 关键事实核查记录（实施时不必重查）：
> - SQL 迁移最新为 `V114__stories_logline.sql`，Rust 迁移最新为 `V118__unify_foreshadowing_thread`（`db/migrations/mod.rs:903`），故 V119 是合法下一编号。SQL 文件迁移由 `MigrationRunner::load_migrations`（`db/migrations/mod.rs:128`）按文件名自动扫描执行，**无需注册**；打包经 `tauri.conf.json:63` 的 `"src/db/migrations/": "db/migrations/"` 携带。不存在 `src-tauri/migrations/` 目录。
> - PromptRegistry 在运行时递归扫描 `resources/prompts/**/*.md`（`prompts/registry.rs:298 load_prompts_from_dir`），新增/删除 md 文件即注册/注销，无硬编码清单；唯二引用点是 `narrative/prompts.rs` 与 `prompts/registry.rs` 的测试。
> - 18 个待删提示词的磁盘位置已全部核实（Task 12 Step 1）。
> - `first_chapter_prompt`（`narrative/prompts.rs:624`）全仓库无调用方；`build_prompt_framework_catalog`（同文件:670）被 `creative_engine/prompt_synthesis/synthesizer.rs:124` 使用，**保留**。
> - `PromptMode::Generate` 在 `narrative/analysis.rs` 中零使用（6 处调用全部 `PromptMode::Extract`，且 strategy/quartet 参数全部传 `None`）。
> - `execute_plan` 的依赖检查（`planner/executor.rs:437-473`）：依赖步骤失败 → 输出缺失 → 后续依赖步骤被跳过。因此 beat_planner 降级必须在 `execute_beat_planner` 内部把失败转为 `Ok(degraded)`，不能让它返回 `Err`。

---

## Task 9：beat_planner capability + writer_beat_plan 提示词

**目标**：新增 `beat_planner` capability——单次 LLM（max_tokens 600、60s 超时）产出 ≤300 字节拍规划 JSON，供 Task 10 的 beat 链注入 writer。失败/超时/解析失败一律降级为 `Ok(degraded)` 输出（见头部事实核查：返回 Err 会连累 writer 步骤被依赖检查跳过）。

**Files:**
- Create: `resources/prompts/writer/writer_beat_plan.md`
- Modify: `src-tauri/src/planner/executor.rs`
  - `capability_display_name`（现 :775-803）注册 `beat_planner`
  - `execute_step` dispatch（现 :811-835）新增 `"beat_planner"` 臂
  - `execute_writer`（现 :1138-1441）之后新增 `execute_beat_planner` + `BeatPlan` + 解析/降级辅助函数
  - 文件末尾（现 2418 行结束）新增 `#[cfg(test)] mod tests`
- Test: 同上（`src-tauri/src/planner/executor.rs` 内 `mod tests`）

**Interfaces:**
- Consumes:
  - `crate::prompts::registry::resolve_prompt(pool: &DbPool, prompt_id: &str) -> Result<String, AppError>`（registry.rs:433）
  - `crate::prompts::registry::resolve_prompt_default(prompt_id: &str) -> Option<String>`（registry.rs:447）
  - `crate::prompts::engine::TemplateEngine::render_with_conditions(template: &str, variables: &HashMap<String, String>) -> String`（engine.rs:30）
  - `crate::llm::LlmService::new(app_handle: AppHandle)` + `generate_for_task(task: TaskType, prompt: String, max_tokens: Option<i32>, temperature: Option<f32>, context_label: Option<&str>) -> Result<GenerateResponse, AppError>`（llm/service.rs:631；`GenerateResponse.content: String`）
  - `crate::narrative::extract_and_sanitize_json(content: &str) -> Result<String, String>`（narrative/mod.rs:143，剥离 markdown 围栏）
  - `crate::db::StoryRepository::new(pool).get_by_id(&str) -> Result<Option<Story>, rusqlite::Error>`（story_repository.rs:157）
  - `crate::strategy::quartet_inference::serialize_quartet_for_prompt(&SelectedStrategy) -> Result<serde_json::Value, _>`（executor.rs:1223 既有用法）
- Produces:
  - `pub struct BeatPlan { goal: String, conflict_escalation: String, new_elements: String, foreshadowing_ops: String, target_words: u32 }`（serde，全字段带默认值）
  - `impl PlanExecutor { async fn execute_beat_planner(&self, params: &HashMap<String, serde_json::Value>, plan_context: &PlanContext) -> Result<serde_json::Value, AppError> }`
  - `impl PlanExecutor { fn parse_beat_plan_output(content: &str) -> Result<BeatPlan, String>; fn degraded_beat_output(reason: &str) -> serde_json::Value }`
  - 输出 JSON 形状：`{"content": "<节拍规划文本>", "beat_plan": {...}, "degraded": false}`；降级时 `{"content": "", "beat_plan": null, "degraded": true, "reason": "..."}`。`content` 键供 `resolve_parameters`（executor.rs:874）的 `{{beat_planner}}` 占位符替换使用。

- [ ] **Step 1: 写失败测试（JSON 解析 + markdown 围栏容错 + 降级输出 + 中文名）**

在 `src-tauri/src/planner/executor.rs` 文件末尾（现 2418 行后）新增：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const VALID_BEAT_JSON: &str = r#"{"goal":"主角潜入档案室窃取名单","conflict_escalation":"守卫临时换岗，时间窗缩短","new_elements":"引入线人角色「灰雀」","foreshadowing_ops":"兑现第3章埋下的钥匙伏笔","target_words":1500}"#;

    #[test]
    fn test_parse_beat_plan_clean_json() {
        let plan = PlanExecutor::parse_beat_plan_output(VALID_BEAT_JSON).unwrap();
        assert_eq!(plan.goal, "主角潜入档案室窃取名单");
        assert_eq!(plan.conflict_escalation, "守卫临时换岗，时间窗缩短");
        assert_eq!(plan.new_elements, "引入线人角色「灰雀」");
        assert_eq!(plan.foreshadowing_ops, "兑现第3章埋下的钥匙伏笔");
        assert_eq!(plan.target_words, 1500);
    }

    #[test]
    fn test_parse_beat_plan_markdown_fenced() {
        // 模型常把 JSON 包在 ```json ... ``` 围栏中，必须先剥离再解析
        let raw = format!("好的，以下是节拍规划：\n```json\n{}\n```\n希望对你有帮助。", VALID_BEAT_JSON);
        let plan = PlanExecutor::parse_beat_plan_output(&raw).unwrap();
        assert_eq!(plan.goal, "主角潜入档案室窃取名单");
        assert_eq!(plan.target_words, 1500);
    }

    #[test]
    fn test_parse_beat_plan_missing_fields_use_defaults() {
        // 模型漏字段不应失败：字符串默认空、target_words 默认 1200
        let plan = PlanExecutor::parse_beat_plan_output(r#"{"goal":"推进主线"}"#).unwrap();
        assert_eq!(plan.goal, "推进主线");
        assert_eq!(plan.target_words, 1200);
        assert!(plan.new_elements.is_empty());
    }

    #[test]
    fn test_parse_beat_plan_invalid_json_errs() {
        assert!(PlanExecutor::parse_beat_plan_output("这不是JSON输出").is_err());
    }

    #[test]
    fn test_degraded_beat_output_shape() {
        // 降级输出必须含 content 键（空串），让 writer 依赖检查通过、
        // {{beat_planner}} 占位符替换为空（TimeSliced 跳过空 beat_plan）
        let out = PlanExecutor::degraded_beat_output("测试降级");
        assert_eq!(out.get("content").and_then(|v| v.as_str()), Some(""));
        assert_eq!(out.get("degraded").and_then(|v| v.as_bool()), Some(true));
        assert!(out.get("beat_plan").map(|v| v.is_null()).unwrap_or(false));
    }

    #[test]
    fn test_beat_planner_display_name() {
        assert_eq!(
            PlanExecutor::capability_display_name("beat_planner"),
            "节拍规划师"
        );
    }
}
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib planner::executor 2>&1 | tail -20
```

预期输出：编译失败，含 `error[E0599]: no function or associated item named 'parse_beat_plan_output' found for struct 'PlanExecutor'`（以及 `degraded_beat_output` 同款错误）。

- [ ] **Step 3: 实现 BeatPlan + 解析/降级函数 + execute_beat_planner + dispatch + 中文名**

3a. `capability_display_name`（executor.rs:776 match 内、`"writer"` 行之后）加一行：

```rust
            "beat_planner" => "节拍规划师".to_string(),
```

3b. `execute_step`（executor.rs:811 match 内、`"writer"` 行之后）加一臂：

```rust
            "beat_planner" => self.execute_beat_planner(params, plan_context).await,
```

3c. 在 `execute_writer` 结束（现 :1441 `}` 之后、`execute_inspector` 之前）插入。结构参照 `execute_writer`（:1138-1158 的参数提取、:1163-1165 的 AppConfig 加载）与 `generate_plan_inner`（planner/mod.rs:695-704 的 `generate_for_task(TaskType::Analysis, ..., Some(1024), Some(0.3), ...)`）：

```rust
    /// v0.31 资产融合重构（Task 9）：beat_planner 节拍规划师。
    ///
    /// 单次 LLM（max_tokens 600、60s 超时）产出 ≤300 字节拍 JSON
    /// （戏剧目标/冲突升级点/引入新元素/伏笔操作/目标字数），注入后续
    /// writer 步骤参数。失败/超时/解析失败**不返回 Err**——execute_plan
    /// 的依赖检查（:437-473）会因依赖步骤无输出而跳过 writer，故降级
    /// 为 `degraded` 输出，writer 拿到空 beat_plan 走单 writer 路径。
    async fn execute_beat_planner(
        &self,
        params: &HashMap<String, serde_json::Value>,
        plan_context: &PlanContext,
    ) -> Result<serde_json::Value, AppError> {
        let story_id = params
            .get("story_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let instruction = params
            .get("instruction")
            .and_then(|v| v.as_str())
            .unwrap_or("续写")
            .to_string();

        // 1) 组装上下文变量：故事上下文摘要 + 当前方法论 step + 策略节拍卡
        let methodology_text = {
            let repo = crate::db::StoryRepository::new(self.pool.clone());
            match repo.get_by_id(&story_id) {
                Ok(Some(story)) => match story.methodology_id {
                    Some(id) => format!(
                        "{}（当前第 {} 步）",
                        id,
                        story.methodology_step.unwrap_or(1)
                    ),
                    None => "未指定".to_string(),
                },
                _ => "未指定".to_string(),
            }
        };
        let mut ctx_lines = vec![format!("故事进度：{}", plan_context.story_progress)];
        if let Some(ref wb) = plan_context.world_building_summary {
            let truncated: String = wb.chars().take(200).collect();
            ctx_lines.push(format!("世界观：{}", truncated));
        }
        if !plan_context.character_list.is_empty() {
            ctx_lines.push(format!(
                "角色：{}",
                plan_context.character_list.iter().take(5).cloned().collect::<Vec<_>>().join("、")
            ));
        }
        if !plan_context.foreshadowing_status.is_empty() {
            ctx_lines.push(format!(
                "活跃伏笔：{}",
                plan_context.foreshadowing_status.iter().take(5).cloned().collect::<Vec<_>>().join("；")
            ));
        }
        if let Some(ref preview) = plan_context.current_content_preview {
            let tail: String = preview.chars().rev().take(300).collect::<String>().chars().rev().collect();
            ctx_lines.push(format!("当前正文末尾：{}", tail));
        }
        let story_context = ctx_lines.join("\n");
        let quartet_text = plan_context
            .selected_strategy
            .as_ref()
            .and_then(|s| crate::strategy::quartet_inference::serialize_quartet_for_prompt(s).ok())
            .filter(|v| !v.is_null())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "无".to_string());

        // 2) 渲染 prompt（PromptRegistry 覆盖优先，回退内置 md，再回退内联模板）
        let template = crate::prompts::registry::resolve_prompt(&self.pool, "writer_beat_plan")
            .ok()
            .or_else(|| crate::prompts::registry::resolve_prompt_default("writer_beat_plan"))
            .unwrap_or_else(|| DEFAULT_WRITER_BEAT_PLAN_TEMPLATE.to_string());
        let mut vars = HashMap::new();
        vars.insert("story_context".to_string(), story_context);
        vars.insert("methodology_step".to_string(), methodology_text);
        vars.insert("strategy_quartet".to_string(), quartet_text);
        vars.insert("instruction".to_string(), instruction);
        let prompt = crate::prompts::engine::TemplateEngine::render_with_conditions(&template, &vars);

        // 3) 单次 LLM，60s 超时（execute_step 外层 90s 步超时兜底）
        let llm = crate::llm::LlmService::new(self.app_handle.clone());
        let call = llm.generate_for_task(
            TaskType::Analysis,
            prompt,
            Some(600),
            Some(0.3),
            Some("beat_plan"),
        );
        let response = match tokio::time::timeout(std::time::Duration::from_secs(60), call).await {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => {
                return Ok(Self::degraded_beat_output(&format!("LLM 调用失败: {}", e)))
            }
            Err(_) => return Ok(Self::degraded_beat_output("beat_planner 超时（60s）")),
        };

        // 4) 解析输出；失败同样降级
        match Self::parse_beat_plan_output(&response.content) {
            Ok(plan) => {
                let text = plan.to_prompt_text();
                Ok(serde_json::json!({
                    "content": text,
                    "beat_plan": plan,
                    "degraded": false,
                }))
            }
            Err(e) => Ok(Self::degraded_beat_output(&format!("输出解析失败: {}", e))),
        }
    }

    /// 解析 beat_planner 的 LLM 输出为 BeatPlan。先经
    /// `extract_and_sanitize_json` 剥离 markdown 围栏并修复未转义换行
    /// （与 novel_creation.rs:138 同款容错），再反序列化。
    fn parse_beat_plan_output(content: &str) -> Result<BeatPlan, String> {
        let sanitized = crate::narrative::extract_and_sanitize_json(content)
            .unwrap_or_else(|_| content.to_string());
        serde_json::from_str(&sanitized).map_err(|e| format!("beat_plan JSON 解析失败: {}", e))
    }

    /// beat_planner 降级输出：content 为空串，writer 依赖检查通过、
    /// `{{beat_planner}}` 占位符替换为空，TimeSliced 跳过空 beat_plan。
    fn degraded_beat_output(reason: &str) -> serde_json::Value {
        log::warn!(
            "[PlanExecutor::execute_beat_planner] 降级为单 writer 路径: {}",
            reason
        );
        serde_json::json!({
            "content": "",
            "beat_plan": null,
            "degraded": true,
            "reason": reason,
        })
    }
```

3d. 同文件（`BeatPlan` 放在 `execute_beat_planner` 之前、`use` 声明之后的模块级位置，紧跟 `PlanExecutor` struct 定义即 :40 `}` 之后）：

```rust
/// beat_planner 产出的单节拍规划（≤300 字 JSON）。
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct BeatPlan {
    /// 戏剧目标
    #[serde(default)]
    pub goal: String,
    /// 冲突升级点
    #[serde(default)]
    pub conflict_escalation: String,
    /// 引入新元素（新角色/新场景/新道具）
    #[serde(default)]
    pub new_elements: String,
    /// 伏笔操作（埋设/推进/兑现）
    #[serde(default)]
    pub foreshadowing_ops: String,
    /// 本节拍目标字数
    #[serde(default = "default_beat_target_words")]
    pub target_words: u32,
}

fn default_beat_target_words() -> u32 {
    1200
}

impl BeatPlan {
    /// 渲染为注入 writer prompt 的文本段。
    pub fn to_prompt_text(&self) -> String {
        format!(
            "【本节拍规划】\n戏剧目标：{}\n冲突升级：{}\n新元素：{}\n伏笔操作：{}\n目标字数：{}",
            self.goal, self.conflict_escalation, self.new_elements, self.foreshadowing_ops,
            self.target_words
        )
    }
}

/// writer_beat_plan.md 缺失时的内联兜底模板（与 md 文件内容保持一致）。
const DEFAULT_WRITER_BEAT_PLAN_TEMPLATE: &str = r#"你是一位小说节拍规划师。基于故事上下文，为下一段续写规划一个节拍。

【故事上下文】
{{story_context}}

【当前方法论进度】
{{methodology_step}}

【创作策略四元组】
{{strategy_quartet}}

【创作指令】
{{instruction}}

请用 JSON 输出本节拍规划（总字数不超过300字）：
{
  "goal": "本节拍的戏剧目标（一句话）",
  "conflict_escalation": "冲突如何升级（一句话）",
  "new_elements": "引入的新元素：有叙事功能的新角色/新场景/新道具（一句话，可为无）",
  "foreshadowing_ops": "伏笔操作：埋设/推进/兑现哪条伏笔（一句话，可为无）",
  "target_words": 1500
}

要求：
1. 新元素必须有叙事功能，不与世界观冲突
2. 只输出 JSON，不要其他内容"#;
```

- [ ] **Step 4: 新建提示词 `resources/prompts/writer/writer_beat_plan.md`**

frontmatter 格式参照 `resources/prompts/writer/writer_chase_debt.md`（category 合法值见 registry.rs:366-391，`writer` 合法）：

```markdown
---
id: writer_beat_plan
name: "Writer 节拍规划"
description: "beat_planner 的单节拍规划提示词：输出戏剧目标/冲突升级/新元素/伏笔操作/目标字数 JSON"
category: writer
version: 0.31.0
variables:
  - story_context
  - methodology_step
  - strategy_quartet
  - instruction
---

你是一位小说节拍规划师。基于故事上下文，为下一段续写规划一个节拍。

【故事上下文】
{{story_context}}

【当前方法论进度】
{{methodology_step}}

【创作策略四元组】
{{strategy_quartet}}

【创作指令】
{{instruction}}

请用 JSON 输出本节拍规划（总字数不超过300字）：
{
  "goal": "本节拍的戏剧目标（一句话）",
  "conflict_escalation": "冲突如何升级（一句话）",
  "new_elements": "引入的新元素：有叙事功能的新角色/新场景/新道具（一句话，可为无）",
  "foreshadowing_ops": "伏笔操作：埋设/推进/兑现哪条伏笔（一句话，可为无）",
  "target_words": 1500
}

要求：
1. 新元素必须有叙事功能，不与世界观冲突
2. 只输出 JSON，不要其他内容
```

- [ ] **Step 5: 跑测试确认通过**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib planner::executor 2>&1 | tail -12
```

预期输出：`test result: ok. 6 passed; 0 failed`（含 `test_parse_beat_plan_clean_json`、`test_parse_beat_plan_markdown_fenced`、`test_parse_beat_plan_missing_fields_use_defaults`、`test_parse_beat_plan_invalid_json_errs`、`test_degraded_beat_output_shape`、`test_beat_planner_display_name`）。

- [ ] **Step 6: 提交**

```bash
cd /Users/yuzaimu/projects/StoryForge && git add src-tauri/src/planner/executor.rs resources/prompts/writer/writer_beat_plan.md && git commit -m "feat(planner): 新增 beat_planner 节拍规划师 capability 与 writer_beat_plan 提示词"
```

---

## Task 10：计划结构改造（beat 链 + 降级 + plan_mode 开关）

**目标**：续写默认计划从塌缩单 writer 步改为 `beat_planner → writer（depends_on beat_planner）` 两步链；`plan_mode == "single_writer"` 时保持旧塌缩行为；TimeSliced 组装注入 beat_plan 文本。beat_planner 失败降级已在 Task 9 的 `execute_beat_planner` 内部完成（返回 `Ok(degraded)`，writer 正常执行、beat_plan 为空即跳过注入），本任务不再改 execute_plan。

**Files:**
- Modify: `src-tauri/src/planner/mod.rs`
  - `sanitize_plan_for_prose_request`（现 :247-311）：新增 `plan_mode: &str` 参数；续写分支改为生成 beat 链
  - `make_sanitized_writer_step`（现 :327-353）之后新增 `make_beat_planner_step`
  - `mod tests`（现 :782 起）：既有 sanitize 测试调用点（:1103/:1122/:1146/:1169/:1188/:1203）补第 4 参；新增 3 个测试
- Modify: `src-tauri/src/planner/executor.rs`：sanitize 调用点（现 :311-315）加载并透传 `plan_mode`
- Modify: `src-tauri/src/agents/orchestrator.rs`：`execute_time_sliced`（现 :868 起）在 progression 锚点注入后（现 :1053-1058）注入 beat_plan
- Test: `src-tauri/src/planner/mod.rs` 内 `mod tests`

**Interfaces:**
- Consumes:
  - `AppConfig.plan_mode: String`（**由另一任务**添加到 `config/settings.rs`，形如 `#[serde(default = "default_plan_mode")] pub plan_mode: String`，默认 `"beat"`，默认值函数参照 :361 `default_generation_mode`；本任务直接以 `c.plan_mode` 引用，缺失时回退 `"beat"`）
  - `crate::config::AppConfig::load(app_dir: &Path) -> Result<AppConfig, _>`（executor.rs:602 既有用法）
  - `PlanContext` 字段（planner/mod.rs:68-104）
- Produces:
  - `PlanGenerator::sanitize_plan_for_prose_request(plan: &mut ExecutionPlan, classification: Option<&WritingIntentClassification>, context: &PlanContext, plan_mode: &str)`（新签名，第 4 参）
  - `PlanGenerator::make_beat_planner_step(context: &PlanContext) -> PlanStep`
  - beat 链形状：`[PlanStep{step_id:"beat_planner", capability_id:"beat_planner", depends_on:[], long_running:false}, PlanStep{step_id:"sanitized_writer", capability_id:"writer", depends_on:["beat_planner"], parameters 含 "beat_plan":"{{beat_planner}}" 与 "planner_understanding", long_running:true}]`

- [ ] **Step 1: 写失败测试（beat 链保留 / single_writer 回退 / 幂等）**

在 `src-tauri/src/planner/mod.rs` 的 `mod tests` 中（`make_sanitize_ctx`/`make_step`/`make_plan` 辅助函数 :1039-1085 之后）新增：

```rust
    // ---- v0.31 资产融合：beat 链计划结构 ----

    #[test]
    fn test_sanitize_continuation_rebuilds_beat_chain() {
        // 续写（is_continuation=true）且 plan_mode=beat：无论 LLM 产出几步，
        // 净化后必须是 [beat_planner, writer(depends_on beat_planner)] 两步链。
        let cls = WritingIntentClassification {
            is_continuation: true,
            is_prose_request: true,
            ..WritingIntentClassification::conservative_fallback()
        };
        let ctx = make_sanitize_ctx("继续写当前这部小说", cls);
        let mut plan = make_plan(vec![
            make_step("s1", "writer"),
            make_step("s2", "inspector"),
        ]);
        PlanGenerator::sanitize_plan_for_prose_request(
            &mut plan,
            ctx.intent_classification.as_ref(),
            &ctx,
            "beat",
        );
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].step_id, "beat_planner");
        assert_eq!(plan.steps[0].capability_id, "beat_planner");
        assert!(plan.steps[0].depends_on.is_empty());
        assert!(!plan.steps[0].long_running);
        assert_eq!(plan.steps[1].capability_id, "writer");
        assert_eq!(plan.steps[1].depends_on, vec!["beat_planner".to_string()]);
        assert!(plan.steps[1].long_running);
        // writer 参数携带 beat_planner 输出引用与 planner understanding
        assert_eq!(
            plan.steps[1]
                .parameters
                .get("beat_plan")
                .and_then(|v| v.as_str()),
            Some("{{beat_planner}}")
        );
        assert!(plan.steps[1]
            .parameters
            .get("planner_understanding")
            .and_then(|v| v.as_str())
            .map(|s| s.contains("test plan"))
            .unwrap_or(false));
        assert!(plan.understanding.contains("beat_planner -> writer"));
    }

    #[test]
    fn test_sanitize_continuation_single_writer_mode_collapses() {
        // plan_mode=single_writer 回退开关：保持旧塌缩单 writer 行为。
        let cls = WritingIntentClassification {
            is_continuation: true,
            is_prose_request: true,
            ..WritingIntentClassification::conservative_fallback()
        };
        let ctx = make_sanitize_ctx("继续写", cls);
        let mut plan = make_plan(vec![
            make_step("s1", "writer"),
            make_step("s2", "inspector"),
        ]);
        PlanGenerator::sanitize_plan_for_prose_request(
            &mut plan,
            ctx.intent_classification.as_ref(),
            &ctx,
            "single_writer",
        );
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].capability_id, "writer");
        assert!(plan.steps[0].depends_on.is_empty());
        assert!(plan.understanding.contains("collapsed to single writer"));
    }

    #[test]
    fn test_sanitize_continuation_beat_chain_idempotent() {
        // 已是 [beat_planner, writer] 链的 plan 不再重建（幂等）。
        let cls = WritingIntentClassification {
            is_continuation: true,
            is_prose_request: true,
            ..WritingIntentClassification::conservative_fallback()
        };
        let ctx = make_sanitize_ctx("继续写", cls);
        let mut writer = make_step("w", "writer");
        writer.depends_on = vec!["bp".to_string()];
        let mut plan = make_plan(vec![make_step("bp", "beat_planner"), writer]);
        PlanGenerator::sanitize_plan_for_prose_request(
            &mut plan,
            ctx.intent_classification.as_ref(),
            &ctx,
            "beat",
        );
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.steps[0].step_id, "bp");
        assert_eq!(plan.steps[1].step_id, "w");
        assert!(!plan.understanding.contains("sanitized"));
    }
```

同时把既有 6 处 sanitize 测试调用（:1103/:1122/:1146/:1169/:1188/:1203）补上第 4 参：非续写（`is_continuation: false`）的传 `"beat"`（行为不变）；若其中有 `is_continuation: true` 的用例，传 `"beat"` 并把断言从「单 writer 步」更新为「beat 链两步」（或改传 `"single_writer"` 保留旧断言，视该测试的回归意图而定——v0.30.14 误路由回归建议传 `"single_writer"` 保持原断言不动，新行为由上面 3 个新测试覆盖）。

- [ ] **Step 2: 跑测试确认失败**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib planner::tests 2>&1 | tail -20
```

预期输出：编译失败，含 `error[E0061]: this function takes 3 arguments but 4 arguments were supplied`（sanitize 新签名尚未实现）。

- [ ] **Step 3: 实现 sanitize beat 链 + make_beat_planner_step**

3a. `sanitize_plan_for_prose_request`（planner/mod.rs:247）签名加第 4 参 `plan_mode: &str`，并把第 2 步「续写塌缩为单 writer」（现 :273-288）整段替换为：

```rust
        // 2. 续写：plan_mode=beat（默认）生成 beat_planner -> writer 两步链，
        //    让 planner 资产理解（understanding）与节拍规划注入 writer；
        //    plan_mode=single_writer 保持旧塌缩行为（AppConfig 回退开关）。
        if cls.is_continuation {
            if plan_mode == "single_writer" {
                if plan.steps.len() != 1 || plan.steps[0].capability_id != "writer" {
                    log::warn!(
                        "[PlanGenerator] Sanitize: collapsing continuation plan ({} steps) to single writer: {}",
                        plan.steps.len(),
                        context.user_input
                    );
                    plan.steps = vec![Self::make_sanitized_writer_step(context)];
                    plan.understanding = format!(
                        "{} [sanitized: continuation collapsed to single writer]",
                        plan.understanding
                    );
                }
                return;
            }
            // beat 模式：已是 [beat_planner, writer] 链则幂等跳过
            let is_beat_chain = plan.steps.len() == 2
                && plan.steps[0].capability_id == "beat_planner"
                && plan.steps[1].capability_id == "writer";
            if !is_beat_chain {
                log::warn!(
                    "[PlanGenerator] Sanitize: rebuilding continuation plan ({} steps) as beat_planner -> writer chain: {}",
                    plan.steps.len(),
                    context.user_input
                );
                let mut writer_step = Self::make_sanitized_writer_step(context);
                writer_step.depends_on = vec!["beat_planner".to_string()];
                writer_step.parameters.insert(
                    "beat_plan".to_string(),
                    serde_json::Value::String("{{beat_planner}}".to_string()),
                );
                writer_step.parameters.insert(
                    "planner_understanding".to_string(),
                    serde_json::Value::String(plan.understanding.clone()),
                );
                plan.steps = vec![Self::make_beat_planner_step(context), writer_step];
                plan.understanding = format!(
                    "{} [sanitized: continuation rebuilt as beat_planner -> writer]",
                    plan.understanding
                );
            }
            return;
        }
```

3b. `make_sanitized_writer_step`（:327-353）之后新增：

```rust
    /// 构造 beat 链首步：beat_planner（单次 LLM 节拍规划，内部可降级，
    /// 见 PlanExecutor::execute_beat_planner）。long_running=false 使其受
    /// 90s 步超时约束（execute_beat_planner 内部另有 60s LLM 超时）。
    fn make_beat_planner_step(context: &PlanContext) -> PlanStep {
        let mut params = HashMap::new();
        if let Some(ref story_id) = context.current_story_id {
            params.insert(
                "story_id".to_string(),
                serde_json::Value::String(story_id.clone()),
            );
        }
        params.insert(
            "instruction".to_string(),
            serde_json::Value::String(context.user_input.clone()),
        );
        if let Some(ref preview) = context.current_content_preview {
            params.insert(
                "current_content".to_string(),
                serde_json::Value::String(preview.clone()),
            );
        }
        PlanStep {
            step_id: "beat_planner".to_string(),
            capability_id: "beat_planner".to_string(),
            purpose: "Beat planner: 规划本节拍的戏剧目标/冲突升级/新元素/伏笔操作".to_string(),
            parameters: params,
            depends_on: vec![],
            long_running: false,
        }
    }
```

3c. sanitize 函数文档注释（:224-246）第 2 条「续写（`is_continuation`）塌缩为单 writer 步」更新为「续写（`is_continuation`）：`plan_mode=beat` 时重建为 `beat_planner -> writer` 两步链，`plan_mode=single_writer` 时塌缩为单 writer 步」。

- [ ] **Step 4: executor.rs 调用点透传 plan_mode**

`src-tauri/src/planner/executor.rs` 现 :311-315 的调用替换为（AppConfig 加载模式参照同文件 :597-604）：

```rust
        // v0.31: plan_mode 开关（"beat" 默认 / "single_writer" 回退），
        // 字段由配套任务加入 AppConfig；加载失败回退 "beat" 保持新默认。
        let plan_mode = {
            let app_dir = self
                .app_handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
            crate::config::AppConfig::load(&app_dir)
                .map(|c| c.plan_mode)
                .unwrap_or_else(|_| "beat".to_string())
        };
        PlanGenerator::sanitize_plan_for_prose_request(
            &mut plan,
            context.intent_classification.as_ref(),
            context,
            &plan_mode,
        );
```

- [ ] **Step 5: 跑测试确认通过**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib planner 2>&1 | tail -12
```

预期输出：`test result: ok.`（planner::tests 与 planner::executor::tests 全绿，含 3 个新测试；既有 sanitize 测试在新第 4 参下通过）。

> 注意：此步依赖配套任务已在 `config/settings.rs` 添加 `plan_mode` 字段。若尚未添加，本步骤会编译失败 `no field 'plan_mode'`——此时先把 `AppConfig` 字段加上（`#[serde(default = "default_plan_mode")] pub plan_mode: String` + `fn default_plan_mode() -> String { "beat".to_string() }` + `Default` impl 处补 `plan_mode: default_plan_mode()`，参照 settings.rs:325-326/:361/:1074），或在任务编排上保证配套任务先完成。

- [ ] **Step 6: TimeSliced 组装注入 beat_plan**

`src-tauri/src/agents/orchestrator.rs` `execute_time_sliced` 中，progression 锚点注入（现 :1053-1055 `if !progression.is_empty() { prompt.push_str(&progression); }`）之后、`ending_anchor` 注入（现 :1056-1058）之前插入：

```rust
        // v0.31 资产融合：注入 beat_planner 的节拍规划文本（beat 链首步产出，
        // 经 writer 步骤参数 "beat_plan" -> AgentTask.parameters 透传）。
        // single_writer 模式或 beat_planner 降级（content 为空）时跳过。
        if let Some(beat_plan) = task.parameters.get("beat_plan").and_then(|v| v.as_str()) {
            if !beat_plan.trim().is_empty() {
                prompt.push_str("\n\n");
                prompt.push_str(beat_plan);
            }
        }
```

（`task.parameters` 的读取模式与同函数 :997-1000 `current_content_for_ctx` 一致；`prompt` 已是 `mut`，见 :1017。）

- [ ] **Step 7: 全量回归 + 提交**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib 2>&1 | tail -6
```

预期输出：`test result: ok.` 全绿。然后：

```bash
cd /Users/yuzaimu/projects/StoryForge && git add src-tauri/src/planner/mod.rs src-tauri/src/planner/executor.rs src-tauri/src/agents/orchestrator.rs && git commit -m "feat(planner): 续写计划改为 beat_planner→writer 两步链，新增 plan_mode 回退开关"
```

---

## Task 11：V119 strategy_json 迁移 + 向导落库 + 主流程优先读

**目标**：`stories` 表新增 `strategy_json TEXT`；向导选中的 beat_card_ids / story_engine_ids / pressure_relationship_id / emotional_payoff / conflict_arena 经 `apply_wizard_to_story` 落库；`build_selected_strategy` 优先读持久化值、缺失字段走启发式补齐；旧数据 NULL 行为与现状完全一致。

**Files:**
- Create: `src-tauri/src/db/migrations/V119__stories_add_strategy_json.sql`
- Modify: `src-tauri/src/db/models.rs`：`Story`（现 :1094-1111）新增 `strategy_json` 字段
- Modify: `src-tauri/src/db/dto.rs`：`UpdateStoryRequest`（现 :141-152）新增 `strategy_json` 字段
- Modify: `src-tauri/src/db/repositories/story_repository.rs`：`create_in_tx`（:41-57）、`get_all`（:75-105）、`get_all_with_counts`（:113-155）、`get_by_id`（:162-192）、`update`（:208-240）的 SELECT/构造补 `strategy_json`；`update_logline`（:195-206）之后新增 `update_strategy_json`
- Modify: `src-tauri/src/domain/strategy.rs`：`SelectedStrategy`（现 :11-45）为 `style_dna_ids`/`skill_ids`/`workflow_id`/`parameters` 补 `#[serde(default)]`（支持只含四元组字段的部分 JSON 反序列化）
- Modify: `src-tauri/src/creation_commands.rs`：`apply_wizard_to_story`（现 :381-463）新增 5 个可选参数并落库；:421 与 :1014 的 `UpdateStoryRequest` 字面量补 `strategy_json: None`
- Modify: `src-tauri/src/commands/orchestrator.rs`：`build_selected_strategy`（现 :1188-1329）在四元组推断（:1312）前叠加持久化值
- Modify: `src-tauri/src/commands/story.rs:86`、`src-tauri/src/db/repositories_tests.rs:83`：`UpdateStoryRequest` 字面量补 `strategy_json: None`
- Modify: `src-tauri/src/workspace/mod.rs`、`src-tauri/src/agency/eval_harness.rs`、`src-tauri/src/commands/orchestrator.rs` 测试夹具 `story_with_genre`（:1371-1389）：`Story` 字面量补 `strategy_json: None`（`Story` 新增字段的编译涟漪，逐处 grep `Story {` 确认）
- Modify: `src-frontend/src/types/index.ts`：`SelectedStrategy`（现 :213-221）新增 5 个可选四元组字段
- Modify: `src-frontend/src/utils/applyWizardToStory.ts`：`WizardData.selectedStrategy`（现 :19-23）改用共享类型；invoke 参数（现 :52-62）新增 5 项
- Test: `src-tauri/src/commands/orchestrator.rs` 内 `mod tests`（现 :1362 起，复用 `create_test_pool`/`ensure_genre_profiles_table`/`story_with_genre`）
- Test: `src-frontend/src/utils/__tests__/applyWizardToStory.test.ts`（新建，vitest，mock 惯例参照 `src-frontend/src/services/__tests__/settings.test.ts`）

**Interfaces:**
- Consumes:
  - `MigrationRunner` 自动扫描 `V{num}__{desc}.sql`（db/migrations/mod.rs:128-180，无需注册）
  - `crate::db::create_test_pool() -> Result<DbPool, _>`（connection.rs:15，`#[cfg(test)]`，跑全量迁移含 V119）
  - `crate::db::StoryRepository::{create, get_by_id, update}`（story_repository.rs）
  - `crate::domain::strategy::SelectedStrategy`（serde；Option 字段缺失自动 None，Vec/HashMap 字段需 `#[serde(default)]`）
  - 前端 `loggedInvoke`（`@/services/tauri`）
- Produces:
  - `Story { ..., pub strategy_json: Option<String>, ... }`
  - `UpdateStoryRequest { ..., pub strategy_json: Option<String> }`（`update()` 以 `COALESCE(?12, strategy_json)` 落库）
  - `StoryRepository::update_strategy_json(&self, id: &str, strategy_json: &str) -> Result<(), rusqlite::Error>`
  - `apply_wizard_to_story(story_id, genre, style_dna_id, genre_profile_id, methodology_id, beat_card_ids: Option<Vec<String>>, story_engine_ids: Option<Vec<String>>, pressure_relationship_id: Option<String>, emotional_payoff: Option<String>, conflict_arena: Option<String>, world_building, characters, writing_style, first_scene, pool, app_handle)`
  - `build_selected_strategy` 行为：持久化四元组字段优先，缺失字段由 `infer_narrative_quartet` 补齐

- [ ] **Step 1: 新建迁移文件**

`src-tauri/src/db/migrations/V119__stories_add_strategy_json.sql`：

```sql
-- V119: stories 表新增 strategy_json —— 持久化向导选中的创作策略四元组
-- （beat_card_ids / story_engine_ids / pressure_relationship_id /
--  emotional_payoff / conflict_arena），JSON 文本。NULL 表示旧数据，
-- build_selected_strategy 对 NULL 走既有启发式推断，行为不变。
ALTER TABLE stories ADD COLUMN strategy_json TEXT;
```

验证编号与命名：`ls src-tauri/src/db/migrations/ | tail -3` 最新 SQL 为 V114，`db/migrations/mod.rs:903` 最新 Rust 迁移为 V118，V119 > 118 合法；文件名 `V{num}__{desc}.sql` 模式由 `parse_filename`（mod.rs:377）解析。

- [ ] **Step 2: Story/UpdateStoryRequest/Repository/SelectedStrategy 结构改造**

2a. `db/models.rs:1108`（`logline` 字段后）加：

```rust
    /// PROBLEM 框架生成的 logline（v0.30.22）
    pub logline: Option<String>,
    /// v0.31: 向导持久化的创作策略四元组 JSON（SelectedStrategy 部分序列化）。
    /// NULL = 旧数据，build_selected_strategy 走启发式推断。
    #[serde(default)]
    pub strategy_json: Option<String>,
```

2b. `db/dto.rs:151`（`reference_book_id` 后）加 `pub strategy_json: Option<String>,`。

2c. `story_repository.rs` 五处改造：
- `create_in_tx`（:53 `logline: None,` 后）加 `strategy_json: None,`
- `get_all` / `get_by_id` 的 SELECT 列清单 `logline, created_at, updated_at` 改为 `logline, strategy_json, created_at, updated_at`；`created_str: row.get(12)` → `row.get(13)`、`updated_str: row.get(13)` → `row.get(14)`；构造中 `logline: row.get(11)?,` 后加 `strategy_json: row.get(12)?,`
- `get_all_with_counts` 同样处理（聚合计数下标 14-17 → 15-18）
- `update`（:215-238）：SQL 中 `reference_book_id = COALESCE(?11, reference_book_id), updated_at = ?12` 改为 `reference_book_id = COALESCE(?11, reference_book_id), strategy_json = COALESCE(?12, strategy_json), updated_at = ?13`；`params!` 在 `req.reference_book_id,` 后插 `req.strategy_json,`
- `update_logline`（:195-206）后新增：

```rust
    /// v0.31: 持久化向导选中的创作策略四元组（apply_wizard_to_story 调用）。
    pub fn update_strategy_json(&self, id: &str, strategy_json: &str) -> Result<(), rusqlite::Error> {
        let conn = self
            .pool
            .get()
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = Local::now().to_rfc3339();
        conn.execute(
            "UPDATE stories SET strategy_json = ?1, updated_at = ?2 WHERE id = ?3",
            params![strategy_json, now, id],
        )?;
        Ok(())
    }
```

2d. `domain/strategy.rs`：`SelectedStrategy` 的 `style_dna_ids`（:21）、`skill_ids`（:23）、`workflow_id`（:25）、`parameters`（:27）各补 `#[serde(default)]`，使只含四元组字段的部分 JSON（如 `{"beat_card_ids":[...],"emotional_payoff":"爽"}`）可反序列化。

2e. 编译涟漪：`creation_commands.rs:421` 与 `:1014`、`commands/story.rs:86`、`db/repositories_tests.rs:83` 的 `UpdateStoryRequest` 字面量补 `strategy_json: None`；`workspace/mod.rs`、`agency/eval_harness.rs`、`commands/orchestrator.rs:1371` 测试夹具的 `Story` 字面量补 `strategy_json: None`（以 `grep -rn "Story {" src-tauri/src` 逐处核对，编译器会报齐）。

- [ ] **Step 3: 写失败测试（Rust 读写回环 + NULL 回退 + 持久化优先）**

在 `src-tauri/src/commands/orchestrator.rs` 的 `mod tests`（:1362 起）中新增（`story_with_genre` 夹具已含 `strategy_json: None`；`GenreProfileRepository::create` 10 参签名照抄 :1426-1445 既有测试）：

```rust
    // ===== v0.31: strategy_json 持久化 =====

    #[test]
    fn test_strategy_json_round_trip_via_repository() {
        let pool = create_test_pool().expect("test pool");
        let repo = crate::db::StoryRepository::new(pool.clone());
        let story = repo
            .create(crate::db::CreateStoryRequest {
                title: "回环测试".to_string(),
                description: None,
                genre: Some("末世".to_string()),
                style_dna_id: None,
                genre_profile_id: None,
                methodology_id: None,
                reference_book_id: None,
            })
            .expect("create story");
        // 新故事 strategy_json 为 NULL（与旧数据一致）
        assert!(story.strategy_json.is_none());

        let json = r#"{"beat_card_ids":["beat_mentor_fallback"],"story_engine_ids":["engine_underdog"],"pressure_relationship_id":"rel_debt","emotional_payoff":"爽","conflict_arena":"公开审查"}"#;
        repo.update_strategy_json(&story.id, json)
            .expect("update strategy_json");
        let loaded = repo
            .get_by_id(&story.id)
            .expect("get_by_id")
            .expect("story exists");
        assert_eq!(loaded.strategy_json.as_deref(), Some(json));
        // 部分 JSON（只含四元组字段）可反序列化为 SelectedStrategy
        let parsed: crate::domain::strategy::SelectedStrategy =
            serde_json::from_str(loaded.strategy_json.as_deref().unwrap())
                .expect("deserialize partial strategy");
        assert_eq!(parsed.beat_card_ids, vec!["beat_mentor_fallback".to_string()]);
        assert_eq!(parsed.story_engine_ids, vec!["engine_underdog".to_string()]);
        assert_eq!(parsed.pressure_relationship_id.as_deref(), Some("rel_debt"));
        assert_eq!(parsed.emotional_payoff.as_deref(), Some("爽"));
        assert_eq!(parsed.conflict_arena.as_deref(), Some("公开审查"));
    }

    #[test]
    fn test_build_selected_strategy_null_strategy_json_unchanged() {
        // NULL（旧数据）回退：行为与现状一致——GenreResolver 自动匹配 +
        // infer_narrative_quartet 启发式补齐四元组。
        let pool = create_test_pool().expect("test pool");
        ensure_genre_profiles_table(&pool);
        let repo = GenreProfileRepository::new(pool.clone());
        repo.create(
            "末世流",
            "Post-apocalyptic",
            Some("[\"末世\", \"末世生存\"]"),
            Some("文明崩溃后的世界"),
            Some("快节奏"),
            Some("[]"),
            None,
            None,
            true,
        )
        .expect("create profile");
        let story = story_with_genre("末世"); // strategy_json = None
        let strategy = build_selected_strategy(&Some(story), &pool, InputClarity::Vague)
            .expect("应匹配到题材画像");
        assert!(strategy.genre_profile_id.is_some());
        // 启发式四元组仍生效（infer_narrative_quartet 从 reader_promise 等补齐）
        assert!(strategy.rationale.contains("体裁画像"));
    }

    #[test]
    fn test_build_selected_strategy_prefers_persisted_quartet() {
        // 持久化四元组优先于启发式：beat_card_ids/emotional_payoff 来自
        // strategy_json，未被 infer_narrative_quartet 覆盖。
        let pool = create_test_pool().expect("test pool");
        ensure_genre_profiles_table(&pool);
        let repo = GenreProfileRepository::new(pool.clone());
        repo.create(
            "末世流",
            "Post-apocalyptic",
            Some("[\"末世\", \"末世生存\"]"),
            Some("文明崩溃后的世界"),
            Some("快节奏"),
            Some("[]"),
            None,
            None,
            true,
        )
        .expect("create profile");
        let mut story = story_with_genre("末世");
        story.strategy_json = Some(
            r#"{"beat_card_ids":["beat_wizard_pick"],"emotional_payoff":"燃"}"#.to_string(),
        );
        let strategy = build_selected_strategy(&Some(story), &pool, InputClarity::Vague)
            .expect("应匹配到题材画像");
        assert_eq!(strategy.beat_card_ids, vec!["beat_wizard_pick".to_string()]);
        assert_eq!(strategy.emotional_payoff.as_deref(), Some("燃"));
        assert!(strategy.rationale.contains("向导持久化"));
    }
```

（`crate::db::CreateStoryRequest` 若未从 `db` 根 re-export，改用 `crate::db::dto::CreateStoryRequest`，以编译器为准。）

- [ ] **Step 4: 跑测试确认失败**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib commands::orchestrator 2>&1 | tail -20
```

预期输出：若 Step 2 未做则编译失败（`no field named 'strategy_json'`）；Step 2 已做而 `build_selected_strategy` 未改时，`test_build_selected_strategy_prefers_persisted_quartet` 断言失败（`beat_card_ids` 为空或非持久化值），`test_strategy_json_round_trip_via_repository` 因 `update_strategy_json` 不存在而编译失败——即「先红」成立。迁移本身由 `create_test_pool` 跑全量迁移验证（若 V119 SQL 有误，`create_test_pool` 直接 panic）。

- [ ] **Step 5: 实现 build_selected_strategy 持久化优先**

`commands/orchestrator.rs` `build_selected_strategy` 中，`infer_narrative_quartet` 调用（现 :1312-1317）**之前**插入：

```rust
    // v0.31: 优先读取向导持久化的策略四元组（stories.strategy_json）。
    // 放在 infer_narrative_quartet 之前：持久化字段先占位，启发式只补
    // 缺失字段（infer 对已是 Some 的字段不覆盖）。NULL / 解析失败
    // （旧数据）跳过本段，行为与现状完全一致。
    if let Some(ref json) = story.strategy_json {
        match serde_json::from_str::<crate::domain::strategy::SelectedStrategy>(json) {
            Ok(persisted) => {
                let mut loaded = false;
                if persisted.emotional_payoff.is_some() {
                    strategy.emotional_payoff = persisted.emotional_payoff;
                    loaded = true;
                }
                if persisted.pressure_relationship_id.is_some() {
                    strategy.pressure_relationship_id = persisted.pressure_relationship_id;
                    loaded = true;
                }
                if persisted.conflict_arena.is_some() {
                    strategy.conflict_arena = persisted.conflict_arena;
                    loaded = true;
                }
                if !persisted.story_engine_ids.is_empty() {
                    strategy.story_engine_ids = persisted.story_engine_ids;
                    loaded = true;
                }
                if !persisted.beat_card_ids.is_empty() {
                    strategy.beat_card_ids = persisted.beat_card_ids;
                    loaded = true;
                }
                if loaded {
                    rationale_parts.push("策略四元组（向导持久化）".to_string());
                }
            }
            Err(e) => {
                log::warn!(
                    "[build_selected_strategy] strategy_json 解析失败，回退启发式: {}",
                    e
                );
            }
        }
    }
```

`SelectedStrategy` 需 derive `Deserialize`（已有，strategy.rs:11）。`infer_narrative_quartet` 只对 `None`/空字段补齐（quartet_inference.rs:24 起），顺序天然满足「持久化优先、启发式补缺」。

- [ ] **Step 6: 跑测试确认通过**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib commands::orchestrator 2>&1 | tail -8 && cargo test --lib db:: 2>&1 | tail -4
```

预期输出：`test result: ok.`（3 个新测试通过；db 既有测试含 `UpdateStoryRequest` 字面量修复后全绿）。

- [ ] **Step 7: apply_wizard_to_story 落库通道**

`creation_commands.rs:382` 命令签名在 `methodology_id` 后新增 5 参（Tauri 按 snake_case 透传，前端同名传参）：

```rust
    methodology_id: Option<String>,
    // v0.31: 向导选中的策略四元组，持久化到 stories.strategy_json
    beat_card_ids: Option<Vec<String>>,
    story_engine_ids: Option<Vec<String>>,
    pressure_relationship_id: Option<String>,
    emotional_payoff: Option<String>,
    conflict_arena: Option<String>,
```

`spawn_blocking` 闭包前（:409-414 的 clone 区）补：

```rust
    let has_quartet = beat_card_ids.as_ref().map(|v| !v.is_empty()).unwrap_or(false)
        || story_engine_ids.as_ref().map(|v| !v.is_empty()).unwrap_or(false)
        || pressure_relationship_id.is_some()
        || emotional_payoff.is_some()
        || conflict_arena.is_some();
    let strategy_json_c = if has_quartet {
        serde_json::to_string(&serde_json::json!({
            "beat_card_ids": beat_card_ids.unwrap_or_default(),
            "story_engine_ids": story_engine_ids.unwrap_or_default(),
            "pressure_relationship_id": pressure_relationship_id,
            "emotional_payoff": emotional_payoff,
            "conflict_arena": conflict_arena,
        }))
        .ok()
    } else {
        None
    };
```

闭包内 `story_repo.update(...)`（:418-434）之后插入：

```rust
            // v0.31: 向导四元组落库（仅当选中过任一四元组字段）
            if let Some(ref json) = strategy_json_c {
                story_repo
                    .update_strategy_json(&story_id_clone, json)
                    .map_err(AppError::from)?;
            }
```

（`handlers.rs:161` 按命令名注册，无需改动。）

- [ ] **Step 8: 前端类型 + 传参 + 失败测试**

8a. `src-frontend/src/types/index.ts:213-221` `SelectedStrategy` 接口补：

```ts
  emotional_payoff?: string;
  pressure_relationship_id?: string;
  conflict_arena?: string;
  story_engine_ids?: string[];
  beat_card_ids?: string[];
```

8b. `applyWizardToStory.ts`：`WizardData.selectedStrategy` 的内联类型（:19-23）替换为 `selectedStrategy?: SelectedStrategy;`（从 `@/types/index` import）；invoke 参数对象（:52-62）在 `methodology_id` 行后补：

```ts
    beat_card_ids: data.selectedStrategy?.beat_card_ids ?? null,
    story_engine_ids: data.selectedStrategy?.story_engine_ids ?? null,
    pressure_relationship_id: data.selectedStrategy?.pressure_relationship_id ?? null,
    emotional_payoff: data.selectedStrategy?.emotional_payoff ?? null,
    conflict_arena: data.selectedStrategy?.conflict_arena ?? null,
```

8c. 新建 `src-frontend/src/utils/__tests__/applyWizardToStory.test.ts`：

```ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { applyWizardToStory } from '../applyWizardToStory';
import { loggedInvoke } from '@/services/tauri';

vi.mock('@/services/tauri', () => ({
  loggedInvoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(loggedInvoke);

const baseStory = {
  id: 'story-1',
  title: '测试故事',
  genre: '末世',
  style_dna_id: null,
  genre_profile_id: null,
  methodology_id: null,
} as any;

const baseWizardData = {
  worldBuilding: { concept: '废土', history: '百年战争' },
  characters: [],
  writingStyle: { name: '冷峻' },
  firstScene: { title: '开场' },
  genreInput: '末世',
} as any;

beforeEach(() => {
  mockedInvoke.mockReset();
  mockedInvoke.mockResolvedValue({
    story: baseStory,
    world_building: {},
    writing_style: {},
    first_scene: {},
    characters: [],
    ingested_entities: 0,
    ingested_relations: 0,
  } as any);
});

describe('applyWizardToStory 策略四元组落库', () => {
  it('向导选中的四元组全部传给后端', async () => {
    await applyWizardToStory(baseStory, {
      ...baseWizardData,
      selectedStrategy: {
        style_dna_ids: [],
        genre_profile_id: 'gp1',
        methodology_id: 'snowflake',
        beat_card_ids: ['beat_a'],
        story_engine_ids: ['engine_x', 'engine_y'],
        pressure_relationship_id: 'rel_debt',
        emotional_payoff: '爽',
        conflict_arena: '公开审查',
      },
    });
    expect(mockedInvoke).toHaveBeenCalledWith(
      'apply_wizard_to_story',
      expect.objectContaining({
        beat_card_ids: ['beat_a'],
        story_engine_ids: ['engine_x', 'engine_y'],
        pressure_relationship_id: 'rel_debt',
        emotional_payoff: '爽',
        conflict_arena: '公开审查',
      })
    );
  });

  it('未选策略时四元组传 null（不污染旧数据）', async () => {
    await applyWizardToStory(baseStory, baseWizardData);
    expect(mockedInvoke).toHaveBeenCalledWith(
      'apply_wizard_to_story',
      expect.objectContaining({
        beat_card_ids: null,
        story_engine_ids: null,
        pressure_relationship_id: null,
        emotional_payoff: null,
        conflict_arena: null,
      })
    );
  });
});
```

8d. 先跑前端测试确认失败再实现（8a/8b 前）：

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend && npx vitest run src/utils/__tests__/applyWizardToStory.test.ts 2>&1 | tail -10
```

预期失败：`expected "spy" to be called with ... objectContaining({ beat_card_ids: ['beat_a'] ... })`（参数对象尚无四元组键）。实现 8a/8b 后重跑：2 passed。

- [ ] **Step 9: 回归门槛 + 提交**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib 2>&1 | tail -4
cd /Users/yuzaimu/projects/StoryForge/src-frontend && npx tsc --noEmit && npx vitest run src/utils/__tests__/applyWizardToStory.test.ts 2>&1 | tail -4
```

预期：`cargo test` 全绿、`tsc` 无错误、vitest 2 passed。然后：

```bash
cd /Users/yuzaimu/projects/StoryForge && git add src-tauri/src/db src-tauri/src/domain/strategy.rs src-tauri/src/creation_commands.rs src-tauri/src/commands src-tauri/src/workspace src-tauri/src/agency src-frontend/src/types/index.ts src-frontend/src/utils && git commit -m "feat(story): V119 迁移 stories.strategy_json，向导四元组落库并优先于启发式读取"
```

---

## Task 12：创世向导融合资产 + 删除死提示词

**目标**：`novel_creation_*` 向导 4 个生成函数的 prompt 注入体裁画像内容（core_tone/anti_patterns/typical_structure）、推荐方法论 `system_prompt_extension`、四元组推荐；删除 7 个 Genesis 生成族 + 11 个死注册提示词的 md 文件与全部代码引用（含 `narrative/prompts.rs` Generate 模式清理）。

**Files:**
- Modify: `src-tauri/src/agents/novel_creation.rs`：`generate_world_building_options`（:54-131）、`generate_character_profiles`（:183-275）、`generate_writing_styles`（:304-384）、`generate_first_scene`（:387-474）prompt 组装后注入资产上下文；新增 `build_creation_asset_context` / `render_genre_profile_section`；`mod tests`（:490 起）新增断言
- Modify: `src-tauri/src/narrative/prompts.rs`：删 `PromptMode` 枚举（:10-27）、6 个函数的 Generate 臂与失效参数、`first_chapter_prompt`（:624-665）、3 个 Generate 保真测试（:693-823 整个 `concept_prompt_fidelity_tests` 模块）
- Modify: `src-tauri/src/narrative/analysis.rs`：import（:31）去掉 `PromptMode`；6 处调用点（:202/:312/:451/:660/:833/:923）适配新签名
- Modify: `src-tauri/src/prompts/registry.rs`：`test_v021_new_prompts_registered`（:635-665）移除 4 个已删 key
- Delete（18 个 md，路径已全部核实存在）：
  - `resources/prompts/creation/narrative_story_concept_generate.md`
  - `resources/prompts/creation/narrative_world_building_generate.md`
  - `resources/prompts/creation/narrative_character_generate.md`
  - `resources/prompts/creation/narrative_scene_generate.md`
  - `resources/prompts/creation/narrative_foreshadowing_generate.md`
  - `resources/prompts/creation/narrative_story_arc_generate.md`
  - `resources/prompts/creation/narrative_first_chapter_generate.md`
  - `resources/prompts/creation/narrative_first_scene_generate.md`
  - `resources/prompts/creation/narrative_genre_profile_generate.md`
  - `resources/prompts/creation/narrative_opening_skeleton.md`
  - `resources/prompts/creation/narrative_outline_extract.md`
  - `resources/prompts/commentator/commentator_paragraph.md`
  - `resources/prompts/deconstruction/deconstruction_story_arc.md`
  - `resources/prompts/knowledge/memory_knowledge_generation.md`
  - `resources/prompts/archive/methodology_character_analysis.md`
  - `resources/prompts/archive/methodology_scene_self_check.md`
  - `resources/prompts/strategy/strategy_reference_book_context.md`
  - `resources/prompts/writer/writer_reference_scene_fewshots.md`
- Test: `src-tauri/src/agents/novel_creation.rs` 内 `mod tests`

**Interfaces:**
- Consumes:
  - `crate::db::GenreProfileRepository::new(pool)` + `get_by_id(&str) -> Result<Option<GenreProfile>, _>`（commands/orchestrator.rs:1266-1267 既有用法）
  - `crate::strategy::GenreResolver::new().resolve_from_text(genre: &str, repo: &GenreProfileRepository) -> Result<Vec<GenreMatch>, _>`；`GenreMatch { profile_id, canonical_name, .. }`（commands/orchestrator.rs:1211-1216 既有用法）
  - `GenreProfile { genre_name, canonical_name, core_tone, anti_patterns_json, typical_structure_json, reader_promise, recommended_methodology_id, .. }`（models.rs:1424-1454）
  - `crate::domain::methodology::normalize_methodology_id(&str) -> &str`（domain/methodology.rs:66）；`MethodologyType`（serde `rename_all = "snake_case"`，:10-18，可 `serde_json::from_value` 解析 id）
  - `crate::creative_engine::methodology::MethodologyEngine::build_prompt_extension(config: &MethodologyConfig, pool: Option<&DbPool>) -> String`（methodology/mod.rs:50）
  - `crate::strategy::infer_narrative_quartet(&mut SelectedStrategy, Option<&str>, Option<&str>, InputClarity)`（commands/orchestrator.rs:1312 既有用法）+ `crate::strategy::quartet_inference::serialize_quartet_for_prompt`
- Produces:
  - `impl NovelCreationAgent { fn build_creation_asset_context(pool: &DbPool, genre_text: &str) -> String; fn render_genre_profile_section(profile: &GenreProfile) -> String }`
  - `narrative/prompts.rs` 新签名（Extract-only，已逐臂核实变量使用）：
    - `pub fn story_concept_prompt(context: &str, pool: Option<&DbPool>) -> String`
    - `pub fn world_building_prompt(story_title: &str, genre: &str, context: &str, pool: Option<&DbPool>) -> String`
    - `pub fn character_prompt(story_title: &str, genre: &str, context: &str, pool: Option<&DbPool>) -> String`（删 `world_concept`，Extract 臂不用）
    - `pub fn scene_prompt(story_title: &str, genre: &str, context: &str, pool: Option<&DbPool>) -> String`（删 `character_names`，Extract 臂不用）
    - `pub fn foreshadowing_prompt(story_title: &str, genre: &str, context: &str, pool: Option<&DbPool>) -> String`（删 `outline_summary`，Extract 臂不用）
    - `pub fn story_arc_prompt(story_title: &str, context: &str, pool: Option<&DbPool>) -> String`

- [ ] **Step 1: 删除 18 个 md 文件并确认引用清零**

```bash
cd /Users/yuzaimu/projects/StoryForge
git rm resources/prompts/creation/narrative_story_concept_generate.md \
  resources/prompts/creation/narrative_world_building_generate.md \
  resources/prompts/creation/narrative_character_generate.md \
  resources/prompts/creation/narrative_scene_generate.md \
  resources/prompts/creation/narrative_foreshadowing_generate.md \
  resources/prompts/creation/narrative_story_arc_generate.md \
  resources/prompts/creation/narrative_first_chapter_generate.md \
  resources/prompts/creation/narrative_first_scene_generate.md \
  resources/prompts/creation/narrative_genre_profile_generate.md \
  resources/prompts/creation/narrative_opening_skeleton.md \
  resources/prompts/creation/narrative_outline_extract.md \
  resources/prompts/commentator/commentator_paragraph.md \
  resources/prompts/deconstruction/deconstruction_story_arc.md \
  resources/prompts/knowledge/memory_knowledge_generation.md \
  resources/prompts/archive/methodology_character_analysis.md \
  resources/prompts/archive/methodology_scene_self_check.md \
  resources/prompts/strategy/strategy_reference_book_context.md \
  resources/prompts/writer/writer_reference_scene_fewshots.md
grep -rn "narrative_story_concept_generate\|narrative_world_building_generate\|narrative_character_generate\|narrative_scene_generate\|narrative_foreshadowing_generate\|narrative_story_arc_generate\|narrative_first_chapter_generate\|commentator_paragraph\|deconstruction_story_arc\|memory_knowledge_generation\|methodology_character_analysis\|methodology_scene_self_check\|narrative_first_scene_generate\|narrative_genre_profile_generate\|narrative_opening_skeleton\|narrative_outline_extract\|strategy_reference_book_context\|writer_reference_scene_fewshots" src-tauri/src src-frontend/src --include='*.rs' --include='*.ts' --include='*.tsx' --include='*.vue' -l
```

预期 grep 输出仅剩 `src-tauri/src/narrative/prompts.rs` 与 `src-tauri/src/prompts/registry.rs`（Step 2/3 清除）。PromptRegistry 运行时扫目录加载，md 删除即注销，无其他注册点。

- [ ] **Step 2: narrative/prompts.rs Generate 模式清理**

已核实：`PromptMode::Generate` 在 `analysis.rs` 零使用；Extract 臂均只用 `title/genre/text` 变量，`strategy_context`/`narrative_quartet` 仅供 Generate 臂；`first_chapter_prompt` 全仓库无调用方；`build_prompt_framework_catalog` 被 `synthesizer.rs:124` 使用必须保留。最小改动：

2a. 删除 `PromptMode` 枚举及 `verb()`（:10-27），文件头注释（:1-6）更新为「每个叙事元素的 Prompt 用于拆书提取（从文本分析）；Generate 模式已随 v0.31 Genesis 生成族提示词删除」。

2b. 6 个函数各删 Generate 臂、去 `match mode`，直接保留 Extract 臂的 `resolve_and_render(...)` 表达式作为函数体；签名按 Interfaces-Produces 改（`story_concept_prompt` 同时删仅供 Generate 的 `available_profiles` 参数及其 `profiles_json` 构造）。

2c. 删除 `first_chapter_prompt`（:624-665）整函数。

2d. 删除整个 `mod concept_prompt_fidelity_tests`（:693-823）：3 个测试全部是 Generate 保真测试，随 Generate 删除；`sample_profiles` 夹具仅服务这些测试，一并删。

2e. `analysis.rs`：:31 `prompts::{PromptMode, *}` 改 `prompts::*`；6 处调用点适配：
- :202 → `story_concept_prompt(&sample, Some(&ctx.pool))`
- :312-320 → `world_building_prompt(title, genre, &sample, Some(&ctx.pool))`
- :451-459 → `character_prompt(&title, &genre, &sample, Some(&pool))`
- :660-668 → `scene_prompt(&title, &genre, &sample, Some(&pool))`
- :833 → `story_arc_prompt(title, &sample, Some(&ctx.pool))`
- :923-931 → `foreshadowing_prompt(title, genre, &sample, Some(&ctx.pool))`

2f. `registry.rs` `test_v021_new_prompts_registered`（:639-657 的 `new_keys` 数组）移除 `"narrative_story_concept_generate"`、`"narrative_genre_profile_generate"`、`"narrative_world_building_generate"`、`"commentator_paragraph"` 四项（其余 key 保留）。

- [ ] **Step 3: 编译确认无悬空引用**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo check 2>&1 | tail -10 && cargo test --lib narrative:: prompts::registry 2>&1 | tail -8
```

预期输出：`cargo check` 无错误（有错误即漏改调用点，按编译器报错补齐）；`narrative` 与 `prompts::registry` 测试全绿（registry 数量型断言 :586-593 `>= 35` 与 :751-758 `>= 70` 是下界断言，删 18 个后仍成立，无需改）。

- [ ] **Step 4: 写向导资产注入的失败测试**

`agents/novel_creation.rs` `mod tests`（:490 起）新增（`GenreProfile` 夹具构造参照 narrative/prompts.rs:703-720 的既有样式）：

```rust
    fn sample_genre_profile() -> crate::db::GenreProfile {
        crate::db::GenreProfile {
            id: "apoc-id".into(),
            genre_name: "末世流".into(),
            canonical_name: "Post-apocalyptic".into(),
            aliases_json: None,
            core_tone: Some("压抑中见温情".into()),
            pacing_strategy: None,
            anti_patterns_json: Some(r#"["圣母主角","无敌开局"]"#.into()),
            reference_tables_json: None,
            typical_structure_json: Some(r#"["崩塌-流浪-聚落-抉择"]"#.into()),
            reader_promise: Some("爽,燃".into()),
            recommended_style_dna_ids: None,
            recommended_methodology_id: Some("hero_journey".into()),
            recommended_skill_ids: None,
            min_quality_tier: None,
            is_builtin: true,
            created_at: chrono::Local::now(),
        }
    }

    #[test]
    fn test_render_genre_profile_section_includes_assets() {
        // 向导 prompt 必须含体裁画像的 core_tone / anti_patterns / typical_structure
        let section = NovelCreationAgent::render_genre_profile_section(&sample_genre_profile());
        assert!(section.contains("末世流"));
        assert!(section.contains("压抑中见温情"), "应含 core_tone");
        assert!(section.contains("圣母主角"), "应含 anti_patterns");
        assert!(section.contains("崩塌-流浪-聚落-抉择"), "应含 typical_structure");
    }

    #[test]
    fn test_render_genre_profile_section_empty_when_no_assets() {
        // 画像无内容字段时返回空串（调用方跳过注入，不污染 prompt）
        let mut p = sample_genre_profile();
        p.core_tone = None;
        p.anti_patterns_json = None;
        p.typical_structure_json = None;
        assert!(NovelCreationAgent::render_genre_profile_section(&p).is_empty());
    }
```

跑测试确认失败：

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib agents::novel_creation 2>&1 | tail -8
```

预期输出：编译失败 `no function or associated item named 'render_genre_profile_section' found`。

- [ ] **Step 5: 实现资产上下文组装与 4 处注入**

5a. `impl NovelCreationAgent` 内（`parse_first_scene_response` 之后）新增：

```rust
    /// v0.31 资产融合：为向导 prompt 组装创作资产上下文——体裁画像内容
    /// （core_tone/反模式/典型结构）+ 推荐方法论 system_prompt_extension +
    /// 四元组推荐。画像解析失败返回空串（调用方跳过注入，记 debug）。
    fn build_creation_asset_context(pool: &DbPool, genre_text: &str) -> String {
        let repo = crate::db::GenreProfileRepository::new(pool.clone());
        let resolver = crate::strategy::GenreResolver::new();
        let profile = resolver
            .resolve_from_text(genre_text, &repo)
            .ok()
            .and_then(|matches| matches.first().map(|m| m.profile_id.clone()))
            .and_then(|id| repo.get_by_id(&id).ok().flatten());
        let profile = match profile {
            Some(p) => p,
            None => {
                log::debug!(
                    "[novel_creation] 未能从输入解析体裁画像，跳过资产注入: {}",
                    genre_text
                );
                return String::new();
            }
        };

        let mut sections = Self::render_genre_profile_section(&profile);

        // 推荐方法论的 system_prompt_extension
        if let Some(ref mid) = profile.recommended_methodology_id {
            let normalized = crate::domain::methodology::normalize_methodology_id(mid);
            if let Ok(mtype) = serde_json::from_value::<crate::domain::methodology::MethodologyType>(
                serde_json::Value::String(normalized.to_string()),
            ) {
                let config = crate::domain::methodology::MethodologyConfig {
                    methodology_type: mtype,
                    is_active: true,
                    current_step: None,
                    custom_params: serde_json::json!({}),
                };
                let ext = crate::creative_engine::methodology::MethodologyEngine::build_prompt_extension(
                    &config,
                    Some(pool),
                );
                if !ext.trim().is_empty() {
                    sections.push_str(&format!("\n【推荐方法论：{}】\n{}\n", mid, ext));
                }
            }
        }

        // 四元组推荐（纯启发式，不调 LLM）
        let mut strategy = crate::domain::strategy::SelectedStrategy::default();
        crate::strategy::infer_narrative_quartet(
            &mut strategy,
            Some(&profile.canonical_name),
            profile.reader_promise.as_deref(),
            crate::intent::InputClarity::Vague,
        );
        if let Ok(quartet) =
            crate::strategy::quartet_inference::serialize_quartet_for_prompt(&strategy)
        {
            if !quartet.is_null() {
                sections.push_str(&format!("\n【中文叙事四元组推荐】\n{}\n", quartet));
            }
        }
        sections
    }

    /// 渲染体裁画像段落（纯函数，便于单测）。三个内容字段全空返回空串。
    fn render_genre_profile_section(profile: &crate::db::GenreProfile) -> String {
        let mut body = String::new();
        if let Some(ref tone) = profile.core_tone {
            body.push_str(&format!("核心基调：{}\n", tone));
        }
        if let Some(ref anti) = profile.anti_patterns_json {
            body.push_str(&format!("反模式（必须避免）：{}\n", anti));
        }
        if let Some(ref structure) = profile.typical_structure_json {
            body.push_str(&format!("典型结构：{}\n", structure));
        }
        if body.is_empty() {
            String::new()
        } else {
            format!("\n【体裁画像：{}】\n{}", profile.genre_name, body)
        }
    }
```

5b. 4 处注入（均在 prompt 构建完成、`generate_for_task` 调用之前；把各函数 `let prompt = if ... } else { ... };` 改 `let mut prompt`，随后追加）：

- `generate_world_building_options`（:106 `};` 后）：genre 文本用 `user_input`

```rust
        // v0.31 资产融合：注入体裁画像/方法论/四元组（有则注入、无则跳过）
        let asset_ctx = Self::build_creation_asset_context(&self.pool, user_input);
        if !asset_ctx.is_empty() {
            prompt.push_str(&asset_ctx);
        }
```

- `generate_character_profiles`（:250 `};` 后）：genre 文本用 `&world_building.concept`
- `generate_writing_styles`（:359 `};` 后）：genre 文本用 `genre`
- `generate_first_scene`（:449 `};` 后）：genre 文本用 `&world_building.concept`

（后 3 处注入代码与上面相同，仅第 2 参不同。注入放在模板渲染**之后**而非 md 变量，原因：用户可在前端覆盖 `novel_creation_*` md，追加式注入对覆盖模板同样生效，且符合「有则注入、无则跳过」。）

- [ ] **Step 6: 跑测试确认通过 + 全量回归**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib agents::novel_creation 2>&1 | tail -8 && cargo test --lib 2>&1 | tail -4
```

预期输出：`agents::novel_creation` 2 个新测试 + 既有解析测试全过；`cargo test --lib` 全绿（确认 18 个提示词删除后无悬空引用、registry 下界断言仍成立）。

- [ ] **Step 7: 提交（两个 commit）**

```bash
cd /Users/yuzaimu/projects/StoryForge
git add src-tauri/src/narrative src-tauri/src/prompts/registry.rs resources/prompts && git commit -m "chore(prompts): 删除 7 个空转 Genesis 生成族与 11 个死注册提示词，清理 narrative Generate 模式"
git add src-tauri/src/agents/novel_creation.rs && git commit -m "feat(novel-creation): 创世向导 prompt 注入体裁画像、推荐方法论与四元组"
```

---

## 跨任务收尾（全部完成后）

- [ ] `cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test` 全绿（含非 lib 目标）
- [ ] `cd /Users/yuzaimu/projects/StoryForge/src-frontend && npx tsc --noEmit` 通过
- [ ] 启动应用冒烟：向导创建新书 → `stories.strategy_json` 有四元组；续写一次 → 日志出现 `[PlanGenerator] Sanitize: rebuilding continuation plan ... beat_planner -> writer chain`；`plan_mode=single_writer` 时回到单 writer

---

## Task 13：全量回归与收尾

**Files:**
- Modify: `CHANGELOG.md`（新增本次重构条目）
- Modify: `docs/plans/2026-08-04-asset-fusion-deep-restructure-design.md`（如有实施期偏差，回写标注）

- [ ] **Step 1：Rust 全量测试**

Run: `cd /Users/yuzaimu/projects/StoryForge/src-tauri && cargo test --lib`
Expected: 全绿（0 failed）；若有个别既有测试因本计划行为变更（如死提示词注册数断言、build_progression_anchor 签名）失败，回到对应 Task 的「既有测试适配」步骤修正，不得删除断言了事。

- [ ] **Step 2：前端类型检查与相关测试**

Run: `cd /Users/yuzaimu/projects/StoryForge/src-frontend && npx tsc --noEmit && npx vitest run src/utils/applyWizardToStory`
Expected: tsc 无错误；vitest 全绿。

- [ ] **Step 3：更新 CHANGELOG**

在 `CHANGELOG.md` 顶部新增条目，概述三块改造（续写链路资产贯通与扩张性写作合约、beat 驱动多步计划、推荐资产贯通与创世融合、死提示词清理），版本号按仓库惯例（当前 v0.30.51，建议 v0.31.0，因含行为级变更与 DB 迁移）。

- [ ] **Step 4：提交**

```bash
git add CHANGELOG.md docs/
git commit -m "docs: 资产融合重构收尾（CHANGELOG v0.31.0 条目）"
```
