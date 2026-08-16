# 续写三角色闭环：资产可见、当场大纲、审查约束下一拍

日期：2026-08-16
状态：已实施（v0.50.0）
决策来源：幕后代理工作室截屏（《帝国的烟火》续写第 2 章）——主创 `done 第2章草稿`、管理 `start 资产回流` 无 `done`、编辑 `gate:revise`、资产栏 `(空)`。用户裁定方案 D（A+B+C），并授权「确认后直接写文档实施、实施完推送发版」。

承接：

- `docs/plans/2026-08-13-agency-only-continuation-design.md`
- `docs/plans/2026-08-16-prose-grounded-outline-design.md`（v0.49.0）

本设计 **不改** 创世/续写唯一编排入口、`PersistMode`、`WriteTimeBundle::to_prompt()`、不接 `ContextPrioritizer`、不自动 DELETE 角色表脏行、不把 Ingest 拉回热路径、不自动改写已落库章节。不宣称真机「唱反调」已修复。

## 1. 问题

续写是三条互不相通的路径：

1. **主创（热路径）**：BeatCard → 正文 → `PersistMode::Append`。Append 不跑 `generate_chapter_outline`；只把 `next_outline_node` 合成 `进度：` 行写入 `scenes.outline_content`；写 `characters_present`，不建角色卡。
2. **管理（后台、尽力而为）**：`spawn_asset_ingest` 写生产表（`characters` / 关系 / 世界观 / 场景大纲 / 故事大纲），**不写当前 run 的 Asset 区**。工作室资产栏只读 `agency_board_items`。有正文时跳过按书名发明。`sync_scene_outline` 在大纲已非空时常无法覆盖。
3. **编辑（后台、无修订轮）**：`spawn_editor_qc` 把 `gate:revise` 写进 **本 run** Review 区。Append 不做修订。下一拍是新 run，BeatCard 不读 Review。

信号：三个 spawn 用裸 `app.emit`，不走 `AgencyRepository::log_activity`。Ingest 的 `start` 在拿到信号量**之后**才发；未拿到锁直接 return，时间线只有 `start` 或什么都没有。`BACKGROUND_LLM_SEMAPHORE` permit=1，ingest 与其它后台 LLM 互挤。

## 2. 目标与非目标

**目标**

- 后台任务（回流 / 质检 / 管理补齐）每条路径都有 `start` 和 `done`（成功 / 失败 / 超时 / 未获得锁），且写入 `agency_activity_log`。
- 本拍生产表变化投影到 **当前 run** Asset 区；工作室卡片可点进角色 / 故事 / 世界构建。
- Append 落库把 BeatCard 写成结构化 `【当前场大纲】`；下一拍 `compile_next_node` 先读它。
- 上一拍未解决的 `gate:revise` 最多 2 条进入下一拍 BeatCard 与主创 prompt。本拍增量若兑现某条，将该黑板条目标 `resolved`。

**非目标**

- 不恢复 TimeSliced / TriShot。
- 不把 Editor / Ingest 拉回同步等待。
- 不接 `ContextPrioritizer`，不改 `WriteTimeBundle::to_prompt()`。
- 不自动 DELETE 角色脏行。
- 不自动改写已落库的第 2 章。
- 本轮计划区仍空。不第四代理。

## 3. 不变量

| 项 | 不变量 |
|---|---|
| 路由 | 创世/续写只走 Agency；幕前 Append 不阻塞写作。 |
| 正文接地 | 有正文时禁止按书名发明；新名字必须出现在本章正文（复用 v0.49.0 `prose_ground`）。 |
| 回流失败 | 只 log + `done 失败`；绝不回滚已落库正文。 |
| 场景大纲 | BeatCard 写成的 `【当前场大纲】` 不被 ingest 覆盖。 |
| 架构 | `agency` 可依赖 `db` / `memory`；`memory` / `db` 不指回 `agency`。 |
| 热路径 LLM | 主创仍一次 `complete()`；不为本闭环加第四次同步 LLM。 |

## 4. 做法

黑板是本拍生产表变化的 **投影**，不是资产栏去查生产表。不在 Append 后加 Producer tool_loop。

### 4.1 后台信号

三个 spawn 都走 `emit` + `log_activity`。`start` 在任务 **spawn 当时** 发出，不等信号量。每条退出路径都有 `done`。

### 4.2 资产栏

Append 落库后立刻把本拍阵容 + 当前场大纲写入当前 run Asset 区。Ingest 成功后再投影角色/故事增量/世界观（姓名须在正文中）。键稳定（`character:{名}` / `outline:scene` / `outline:story` / `world:concept`），同键修订不堆重复卡。

空资产栏仅当零变化或回流失败（时间线须说明原因）。

`AiContextCards` 增加可选点击；`AgencyStudio` 按 `item_type` → `setCurrentView`。

### 4.3 当前场大纲（0 额外 LLM）

```
【当前场大纲】
在场：…
冲突：…
情感：…
下一拍：…
地点：…
```

`compile_next_node`：若最新章含该块且 `下一拍` 尚未出现在近文镜头，用之；否则走既有接地书纲 / 方法论回落。

### 4.4 审查 → 下一拍

`list_items_for_story(Review)` 取未 `resolved` 的 `gate:revise`，最多 2 条 issue 写入 BeatCard `【待兑现审查】`。本拍增量点名该冲突/角色则 `status=resolved`，剩余问题改写 JSON 后保留。

## 5. 验收（契约测试）

- Ingest / QC / resume 的退出标签：成功 / 失败 / 超时 / 未获得锁都产出 `done` 文案；`persist_activity` 写入后 `list_activities` 能读到 start+done。
- 生产表/节拍卡变化 ⇒ 当前 run Asset 区非空。
- Append 落库 ⇒ `outline_content` 含 `【当前场大纲】` 与 `下一拍`。
- 新 run 的 BeatCard `render_full` 含上一拍 `revise` 问题。
- 工作室资产卡点击调用 `setCurrentView('characters'|…)`。

## 6. 真机

须在同一《帝国的烟火》开头上再点续写。本版不得宣称唱反调已修复。
