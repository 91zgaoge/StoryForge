# 资产回流（Ingest → 生产资产表桥接）实施计划

> 来源：用户提供的全面审计结论与修复方案（2026-08-11），关键论断已逐条复核属实。
> 目标：让后台资产 agent（IngestPipeline）从已生成正文提炼的资产，写入续写 writer 实际读取的生产资产表。

## 审计结论（已复核）

资产 agent = `IngestPipeline`（`src-tauri/src/memory/ingest.rs`），唯一读已生成正文做结构化提取的后台进程。失效根因 5 条：

1. **落库表错位（最致命）**：提取结果写 `kg_entities`/`kg_relations`（orchestrator.rs:694-707 已核实）；续写 writer（`build_writer_context_from_db` coordinator.rs、WriteTimeBundle、build_progression_anchor）只读生产资产表 `characters`/`character_relationships`/`world_buildings`/`story_outlines`/`scenes.outline_content`/`story_contracts`。唯一到生产表的是伏笔（foreshadowing_tracker，ingest.rs:330）。
2. **提取 prompt 浅且字段错配**：`resources/prompts/memory/memory_content_analysis.md`（v0.26.28）要求 `type`/`relation`/`description`/`summary`，schema 是 `entity_type`/`relation_type`（ingest.rs:80-103），无 description 字段——结构化字段大量反序列化为空。
3. **新角色被丢弃**：`persist_character_states`（ingest.rs:1223）按名匹配 characters 表，未注册直接 `continue`。
4. **Agency 续写路径不跑 ingest**：`handle_gate` 装配落库（coordinator.rs:3679 SceneRepository）后只有 `spawn_editor_qc`（:1667/:2328）；仅 orchestrator/TriShot 路径（orchestrator.rs:665-719）跑 ingest。
5. **资产只创世播种、永不回流**：`world_buildings`/`story_outlines`/`scenes.outline_content`/`story_contracts` 仅创世/wizard 写一次；`ensure_assets` 行数门控，从不读正文精炼。

## 实施任务

### Task 1（组件 1+2+4）：提取 schema 升级 + 资产桥 + 源感知合并

**1a. 重写 `resources/prompts/memory/memory_content_analysis.md`**（version bump 0.37.0），与 schema 严格对齐并提取写作级字段：

- 角色：name/entity_type=Character/role_type/personality/background/goals/fears/appearance/gender/age/emotional_core/emotional_trigger/emotional_wound/emotional_need/importance_score
- 关系：source/target/relation_type/description/dynamic/emotional_bond/emotional_intensity/reverse_emotional_bond/reverse_emotional_intensity
- 世界观增量：world_building:{concept,rules[{name,description,rule_type,importance}],history,cultures[{name,description,customs[],values[]}]}
- 本场景大纲：scene_outline:{dramatic_goal,key_events[],conflict_type,setting_location,setting_time,atmosphere,characters_present[],emotional_tone}
- 故事增量：story_delta:{core_conflict,turning_points[]}
- 保留 events/sentiment/foreshadowing/themes；格式约束（禁 markdown 围栏、纯 JSON）。

**1b. 扩展 ingest.rs 结构体**（全部 `#[serde(default)]` 向后兼容）：

- `AnalyzedEntity` 增角色画像字段（role_type/personality/background/goals/fears/appearance/gender/age/4 情感属性/importance_score）
- `AnalyzedRelation` 增 description/dynamic/emotional_bond/emotional_intensity/reverse_emotional_bond/reverse_emotional_intensity
- `ContentAnalysis` 增 `world_building: Option<WbDelta>`/`scene_outline: Option<SceneOutlineDelta>`/`story_delta: Option<StoryDelta>`（新建 struct）
- 同步 `analyze_content` 内联默认 prompt（ingest.rs:481 起）与新 .md 一致（fallback 不弱于 .md）

**1c. 新建 `src-tauri/src/memory/asset_bridge.rs`**：`sync_assets_from_analysis(pool, story_id, scene_id, &ContentAnalysis) -> usize`，复用 materialize_assets（agency/materialize.rs）的 upsert 模式：

- 角色：story_id+name UPDATE-then-INSERT 写 characters（background/personality/goals/4 情感属性），`source='ingest'`，`is_auto_generated=1`；新角色自动注册（修根因 3）
- 关系：按名解析双方 character_id，按 (story_id,source,target) 去重写 character_relationships（type/description/dynamic/双向情感）
- 世界观：ON CONFLICT(story_id) upsert world_buildings——规则按 name 去重追加、history/cultures 仅空时填、concept 不覆盖用户值
- 场景大纲：scene_id 存在时 UPDATE scenes.outline_content（空或 auto 时填，用户已设保留）
- 故事大纲：upsert story_outlines——core_conflict 设置/追加、turning_points 追加；无行则建
- **源感知合并（组件 4）**：仅填空字段或精炼 `source IN ('ingest','agency','auto_placeholder')` 的行；`source='user_created'/'manual'` 的字段保留

**1d. 接入 run_ingest**：在 analyze_content 之后、KG 持久化之前调用桥接；并调整顺序——先 upsert 角色（桥接），再 persist_character_states（新角色先注册，状态才能匹配）。

**1e. 单测**：桥接 upsert（新角色注册/既有精炼/用户编辑保留）、关系去重、世界规则追加去重、场景 outline 填充、故事大纲追加、源感知边界；结构体反序列化（新字段+旧 JSON 兼容）。

### Task 2（组件 3）：Agency 续写路径接入

- coordinator.rs 仿 `spawn_editor_qc`（:2438）新增 `spawn_asset_ingest(run_id, story_id, scene_id, content)`：测试环境（app_handle=None）no-op；生产 tokio::spawn 构造 LlmService+IngestPipeline 跑 ingest+桥接，独立 deadline，`emit_activity(Producer, "资产回流")`
- `handle_gate` 装配落库 scenes.content（:3679 附近）后调用；`run_batch_inner` 同理
- orchestrator.rs:665-719 既有 ingest 调用点补桥接调用（注意此处 IngestContent.scene_id=None，桥接场景大纲跳过即可）

### Task 3（组件 5）：验证与文档

- `cargo test --lib` 全绿 + `cargo check` / `cargo +nightly fmt` / `cargo clippy --lib` 零新增 / `python3 scripts/architecture_guard.py` / `npx tsc --noEmit` / `npx vitest run`
- 按 AGENTS.md 强制规则更新：README.md、CHANGELOG.md、AGENTS.md、PROJECT_STATUS.md、ROADMAP.md、ARCHITECTURE.md、TESTING.md、docs/USER_GUIDE.md
- 版本 0.36.1 → **0.37.0**（含 `landing/src/hooks/useLatestRelease.ts` FALLBACK_VERSION）

## 不改动（职责边界）

- kg_entities/kg_relations 记忆层保留，桥接单向 正文→生产资产表
- 拆书 narrative_* 层不动
- writer 读取路径不改（生产表填上后自动强关联）

## 测试基线

- rust: 1287 passed / 2 ignored
- vitest: 404 passed / 3 skipped
