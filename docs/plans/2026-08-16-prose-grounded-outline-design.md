# 正文为大纲真相源：禁止按书名发明 + 管理失败后台续跑 + 方法论推进

日期：2026-08-16
状态：已确认，待实现（目标版本 v0.49.0）
决策来源：v0.48 真机《帝国的烟火》续写在婚礼刺杀高潮切到费迪南三世；用户裁定「唱反调，得大改」。澄清后锁定：**有正文时大纲只从章节归纳（方案 2）**；管理 Agent 熔断不得掐断用户、未完成补齐改后台；未选定方法论则落库场景结构规范（情节冲突）。

承接：

- `docs/plans/2026-08-13-agency-only-continuation-design.md`
- `docs/plans/2026-08-14-continue-prompt-asset-selection-design.md`
- `docs/plans/2026-08-15-continue-quality-closure-design.md`（v0.47 真机失败；v0.48 修镜头窗口/落库/幽灵；**本事故是新类：书名发明大纲 + 换场换主角**）

本设计 **不改** 创世/续写唯一编排入口、`PersistMode`、`WriteTimeBundle::to_prompt()` 全局语义、热路径不为选人加 LLM。不宣称八次真机探针已过，直到本版契约 + 真机重跑。

## 1. 问题

### 1.1 用户可感知症状

点续写后，增量与已有几千字开头不是同一部小说：镇北王府大堂里苏会山刚被刺死，下一拍却是帝都费迪南读密报、查火药账。

### 1.2 根因（库 + 日志 + 代码，已执行）

故事 `69f4c1fc-cd04-47dc-aefa-83edd527cbc1`《帝国的烟火》：

| 时刻 (CST) | 事实 |
|---|---|
| 09:53:06 | 建故事 + 第一章。正文是知启纪元 / 大奉 / 黑崎州城 / 苏会山。简介、logline、方法论皆空。无大纲、无角色卡。 |
| 09:53:44 | 第一次续写。`ensure_assets` 见角色表空，同步跑管理 Agent tool_loop「按书名补资产」。09:56:05 熔断，整次续写对用户失败。 |
| 09:56:26 | 第二次续写开始。历史黑板条目瞬间落库：费迪南三世 / 艾拉 / 塞尔吉奥（`source=agency`）+ 三卷灰烬/烟火节/火山大纲。 |
| 09:57 | 主创按该大纲写出费迪南增量（2629 字）。 |
| 10:00:23 | 写完后 ingest 才从正文抽出苏会山等人，**追加**【转折点】到费迪南大纲尾部。旧三卷不删。 |

机制：

1. **`ensure_story_outline` / 管理 Agent 补齐不读 `scenes.content`。** 输入是 `续写《标题》第 N 章` + 空简介。`story_info` 只返回标题/类型/简介。
2. **角色表空的反应是发明，不是提取。** 提取发生在正文落库**之后**（`spawn_asset_ingest`）。
3. **管理 Agent 熔断 = 用户失败**（`producer_out.aborted → return Err`），但黑板脏资产仍在，下次续写 `materialize` 免费落库。
4. **`compile_next_node` 从书大纲挑句**，只要点到本拍人名即可。假大纲里的帝都情节一旦带「苏会山」就可合法换 POV。
5. **大纲归纳走 PROBLEM 七元素，不走 `stories.methodology_id`。** 未选定方法论时也不会落到场景结构。

不是「文笔不好」，是资产真相源用错了。

## 2. 目标与非目标

**目标**：已有章节正文时，大纲、角色、世界观只能从正文来；情节往下发展必须服从该故事的创作方法论；管理 Agent 失败则后台续跑，不挡住这一拍续写；高峰未收束时续写留在本场。

**非目标**：

- 不恢复 TimeSliced / TriShot。
- 不把 Editor / Inspector 拉回同步等待。
- 不接 `ContextPrioritizer`。
- 不改 `WriteTimeBundle::to_prompt()`。
- 不自动 DELETE 角色表历史脏行（注入与落库门闩即可；用户可在幕后手删）。
- 不新增 NER 模型。门闩的「姓名」= 本轮角色卡 `name` ∪ 已登记 `characters.name`。
- 真创世（全书尚无 ≥200 字正文）仍允许从前提生产资产。
- 不宣称本设计落地后五症状已修复，直到契约测试绿 **且** 真机续写不再按书名换主角。

## 3. 不变量

| 项 | 不变量 |
|---|---|
| 正文阈值 | 任一故事全部场景去标记后合计 ≥200 字 → **有正文**。 |
| 真相源 | 有正文时，标题 / 简介 / logline **不得**引入正文未出现的主角、地点、势力。 |
| 方法论 | 有正文归纳大纲时必须加载 `stories.methodology_id` 对应 prompt。空则落库 `scene_structure`（场景结构规范：目标→冲突→灾难 / 反应→困境→决定），`methodology_step=1`。已有 id **不覆盖**（含 `custom_*`）。 |
| 大纲两层 | **已发生**：只归纳正文里的人/地/事。**往下发展**：方法论下一拍，种子只能是正文已有阵容。 |
| 管理失败 | Producer tool_loop 熔断 ≠ 用户失败。能过门闩的黑板资产落库；未完成补齐 `spawn` 后台续跑。禁止再抛「资产补齐未完成」挡住续写。 |
| 留场 | 无用户换场指令、且配额不含 `NewScene` 时，增量不得以场外已登记角色开篇当 POV。 |
| 路由 / 落库 / 增量 | 仍只走 Agency；幕前 Append；≥200 字才落库；返回增量。 |
| 热路径 LLM | 主创一次 `complete()`；探针失败可再一次。有正文且角色表空时，允许 **一次** 提取（把现有 ingest 提前），超时 fail-open。禁止再为「按书名发明」同步跑满轮 tool_loop。 |
| 架构边界 | `agency` 可依赖 `db` / `creative_engine` / `prompts` / `memory`；`db` 不指回 `agency`。 |

## 4. 有正文时的资产路径

替换 `ensure_assets` 在 `character_count == 0` 时的同步「标题发明」tool_loop。

```
有正文？
  ├─ 否 → 真创世/空书：现有 producer 生产资产；熔断改为 salvage + 后台续跑（不 Err 掐死，若确实零资产则续写仍可走正文窗口）
  └─ 是 →
        1. 若 methodology_id 空：UPDATE stories SET methodology_id='scene_structure', methodology_step=1
        2. 角色表空或全是未过门闩的发明：对已有章节跑提取（复用 IngestPipeline；热路径一次；超时/失败则本拍不造假卡）
        3. materialize / ingest 落库前：姓名必须在正文中出现（match_character_names）
        4. 大纲缺失或判定未接地：单次 complete() 归纳，输入 = 开篇+近文摘录 + 方法论 prompt；禁止 PROBLEM 当骨架
        5. 跳过阻塞式「按书名补齐」tool_loop
        6. 关系/世界观仍缺：spawn 后台管理 Agent 续跑（同一门闩）
```

`story_info` 在有正文时必须附带开篇摘录（建议 800 字）+「禁止发明正文未出现的姓名」。即使后台管理 Agent 仍调该工具，也看不到「只有书名」的真空。

## 5. 门闩（落库与注入）

纯函数，0 I/O，可单测：

- `concat_scene_prose(html_or_text) -> plain`
- `name_in_prose(name, prose) -> bool`（复用 `continue_assets::match_character_names`）
- `filter_names_to_prose(names, prose) -> Vec<String>`
- `outline_is_grounded(outline, prose, candidate_names) -> bool`：`candidate_names` 里出现在大纲中的姓名，必须也出现在正文；一个未接地则整份大纲对续写视为缺失。
- `DEFAULT_METHODOLOGY_ID = "scene_structure"`

**落库**：`materialize_assets` 写角色前过滤；`ensure_story_outline` 落库前 `outline_is_grounded`，失败则不写、打 warn、后台再试。

**注入**：`render_continue_assets` / `compile_next_node` 若当前大纲未接地，当空大纲处理，不得把费迪南三卷当硬约束。旧行可留在表里供幕后手改。

**ingest 追加**：`sync_story_delta` 仍可追加【转折点】，但追加段也要过门闩；不得把未接地的旧三卷「修成合法」——未接地大纲在注入层直接跳过。

## 6. 管理 Agent 失败 → 后台续跑

对齐 `spawn_editor_qc` / `spawn_asset_ingest`：

- `ensure_assets` 里 `producer_out.aborted`：**先** `list_zone(Asset)` → 门闩 → `materialize`；**禁止** `return Err(circuit_break_message(...资产补齐未完成))`。
- `spawn_producer_resume(run_id, story_id)`：独立 300s deadline；`BACKGROUND_LLM_SEMAPHORE`；测试环境 `app_handle=None` no-op；同一 `story_id` 同时只跑一个（复用现有后台串行）。
- 后台任务：从正文补齐未进表人物、接地大纲、世界观；全部过门闩。
- 事件：`agency-agent-activity` `Producer start/done 后台补齐`；可选 toast「创作资产后台补齐中」，**不弹 Fatal、不设 isGenerating**。
- 创世「完全无正文」的 producer 生产：同样熔断不 Reckless 丢草稿；本设计优先保证 **续写补齐** 这条已实证的失败路径。空书创世 producer 熔断也改为 salvage + 后台，避免同类掐死。

## 7. 方法论如何驱动情节

归纳大纲与 `compile_next_node` 都读方法论：

- `scene_structure`（默认）：已发生部分标成目标场景或反应场景。末句若是灾难（死、刺、败、崩），下一拍硬任务 = 本场仍在场者的反应→困境→决定，**不得换场换主角**。
- `hero_journey` / `snowflake` / `character_depth` / `high_density_world_building` / `custom_*`：注入对应已注册 prompt；下一节点仍须点名本拍在场者。
- PROBLEM（`agency_problem_outline`）不再作为有正文时的大纲骨架。无正文创世可继续用 PROBLEM + 前提。

`compile_next_node` 不再用「书大纲里碰巧含本拍人名的下一句」作为换 POV 许可证。书大纲句子仅当：（1）接地；（2）点名本拍在场者；（3）不把场外角色写成动作主语。否则走方法论默认下一拍文案。

## 8. 续写留在本场

- 配额无 `NewScene`、用户指令未明确换场时：`ending_anchor` 保持「禁止另起开篇」；新增探针缺口 `增量以场外角色开篇`：增量首 80 字点名了角色表中**不在**本拍 shot 窗口（`PRIOR_CAST_CHAR_CAP`）的人，且未点名任何在场者。
- 探针失败：已有一次 `complete()` 重试；仍失败且 ≥200 字仍落库（不丢稿），但该增量 **不得** 写入「已覆盖书大纲节点」的进度。
- 用户指令含明确换场/换线（「切到帝都」「写费迪南」）时放行。

## 9. 脏数据

本机《帝国的烟火》三卷费迪南大纲 + agency 三角色不自动 DELETE。下次续写：

- 注入层当未接地 → 当空大纲；
- 有正文则按 §4 重归纳接地大纲（UPDATE 同一行或仅注入新文、旧文留表——实现选 **注入用新归纳、UPDATE content 为接地版**，幕后可见校正后的大纲）。
- 费迪南角色行：不注入本拍（不在 shot、且不在正文），`render_continue_assets` 已有未上场名单「不得当主角」；再加门闩：未在正文出现的登记名 **不进完整卡、不进未上场名单**（名单本身会提醒模型去写他）。未接地名从 roster 剔除。

## 10. 阶段

```
P0 门闩纯函数 + 有正文禁止标题发明 + 熔断不 Err
  → P1 从正文归纳大纲（方法论默认 scene_structure）+ 脏大纲跳过注入
  → P2 compile_next_node / 探针留场 + 后台 spawn_producer_resume
  → P3 契约测试 + 文档发版（真机另跑）
```

每阶段结束后幕前续写必须仍可运行。

## 11. 验收（契约）

必须有测试保护用户结果，不是只保护函数名：

1. **书名不能造人**：正文含「苏会山 / 大奉 / 镇北王府」，标题「帝国的烟火」，角色表空。过 `filter_names_to_prose` / 模拟 materialize 后，不得出现「费迪南」；必须能保留「苏会山」。
2. **未接地大纲不注入**：三卷费迪南大纲 + 苏家正文 → `outline_is_grounded == false`；`compile_next_node` 不得返回费迪南句。
3. **默认方法论**：`methodology_id` 空 + 有正文 → 落库 `scene_structure` / step 1；已有 `hero_journey` 不覆盖。
4. **管理熔断不失败续写**：`ensure_assets` 在 producer aborted 时返回 `Ok`（有正文或可 salvage）；测试环境不 spawn。
5. **留场探针**：shot 在场苏亦铁/曹元佩，增量以「费迪南三世扣下情报」开篇 → 探针含场外开篇缺口。
6. `cargo test --lib` 相关新测全绿；`tsc` / vitest 无回归。

真机：同一开头再点续写，增量须留在王府大堂，不得帝都开篇。未跑不得宣称已修复。

## 12. 风险与 META-0

- GitNexus 索引落后，`compile_beat_card` / `ensure_assets` 的 impact 可能报 UNKNOWN。改这些符号前仍跑 impact；HIGH/CRITICAL 则停。按名称谨慎改 `ensure_assets`、`compile_next_node`、`materialize_assets`、`probe_increment`。
- 热路径多一次 ingest：仅角色表空且有正文。超时 fail-open，主创仍能只靠正文窗口写。
- 把未上场名单里删掉未接地名，可能让模型自由发明新名。用既有「禁止新编」纪律 + 探针场外开篇对 **已登记** 场外名；全新胡编名不在本设计范围（既有问题）。
