# 戏剧工艺融合（AI-drama-pound）

日期：2026-08-29
状态：已落地（v0.58.0：工艺 + 短剧格式路径）
来源：[POUND0423/AI-drama-pound](https://github.com/POUND0423/AI-drama-pound)（MIT）
决策：用户批准「工艺融入 + 短剧模式」；不嵌对方 skill 进程；不把主创拉回 ToolLoop。

承接：

- `docs/plans/2026-08-13-agency-only-continuation-design.md`
- `docs/plans/2026-08-15-continue-quality-closure-design.md`
- `docs/plans/2026-08-27-continue-director-lock-design.md`
- `docs/plans/2026-08-25-grok-bot-control-plane-fusion-design.md`

## 1. 问题

对方是 Codex 短剧编剧 skill，不是应用。本仓库是长篇小说桌面应用。缺的是戏剧推进不变量（每拍必须改变一项、潜台词、反转有线索、审阅形状），不是又一套三角色。短剧作为可选 `story_format`，默认 novel。

## 2. 目标与非目标

**目标：**

1. 节拍卡编译「本拍必须改变」的 `ChangeDelta`（0 LLM）。
2. 续写短合同加一对原地踏步范例；创世/大纲加可见行动与不可逆转折。
3. 编辑审计 `blocking_issues` 可读 `impact`/`fix`；维度含 stall / reversal。
4. 探针仅在「增量是近文复述且未出现改变项词」时 gap；旁白不误杀。
5. `stories.story_format`：`novel`（默认）| `short_drama`；显式短剧词才切换。

**非目标：**

- 不 vendoring 对方 skill；不默认繁体；不分镜/影片提示词。
- 不把剧本场次标头灌进小说续写；主创仍 `complete()` 零工具。
- 不自动删角色表脏行；不宣称真机续写质量已修好。

## 3. 不变量

1. `scenes.content` 唯一叙事真相源。
2. `continue_beat_complete_does_not_require_tools` 仍绿。
3. 三档路由不变。导演仍可选 Tool 档 JSON。
4. `prompts` 不得依赖 `agency`。`DRAMA_BEAT_SYSTEM` 放 `prompts/assembly.rs`。
5. `CONTINUE_BEAT_SYSTEM` 非空行 11–19（加第 6 条与一对原地踏步范例后）。
6. 取消停候选链；`<90s` 不重试。死人/拆人/亲缘探针保留。
7. 分类默认 novel：误标短剧会毁掉纸面。

GitNexus（实施前 executed）：`compile_beat_card` MEDIUM（11 直接调用）；`probe_increment` MEDIUM；`write_beat_once` LOW。

## 4. 一期：工艺

### 4.1 ChangeDelta

`SceneBeatCard.change_delta: ChangeDelta { kind, summary }`。kind：信息/关系/目标/风险/情绪。

编译：敌对双方在场 → 风险（复用 `compile_conflict` 加压/对峙）；否则用下一节点 → 信息；再否则情感。`render_full()` 含 `必须改变：{kind} — {summary}`。冻结件随卡走。

### 4.2 合同

`CONTINUE_BEAT_SYSTEM` 第 6 条 + Wrong/Right（全员震惊 vs 一件不可逆落地）。创世 system 加「开篇须有可见行动与未解问题」。`agency_problem_outline.md`：转折不可逆、反转可回看、章尾下一步问题。

### 4.3 编辑

`blocking_issues` 对象可含 `impact`/`fix`；缺键空串，不 Failed。维度 prompt 补 stall、reversal。顶栏 fail-open 保持 v0.56.2。

### 4.4 探针

`probe_increment_ex(..., prior_tail, story_format)`。小说：近文复述且改变项关键词均未出现 → `本拍未兑现必须改变`。摘要抽不出词则不 gap。短剧另要求增量含「内景」或「外景」。既有 `probe_increment` 保持五参，内部转 novel + 空尾。

## 5. 二期：短剧格式

V131：`story_format TEXT NOT NULL DEFAULT 'novel'`，`production_constraints TEXT`。`CreateStoryRequest` 不加字段（避免百处字面量）；创世/新建后 `update_story_format`。`looks_like_short_drama_request`：含短剧/竖屏/分集剧本，且不是「长篇」而无短剧词。

`assemble_continue_beat_for(user, format)`：short_drama 用 `DRAMA_BEAT_SYSTEM`（场次标头、禁止镜号）。幕前仍写 `scenes.content`。

## 6. 验收

一期：`change_delta_from_hostile_cast`、`beat_card_render_includes_must_change`、`continue_system_has_stall_example`、`editor_issue_parses_impact_and_fix`、`probe_gaps_when_increment_is_tail_recap`、`probe_does_not_gap_literary_aside_when_not_recap`。既有零工具/导演锁/死人重演/11–19 行。

二期：迁移默认 novel；`looks_like_short_drama_request` 玄幻长篇为假；短剧组装含内景；vitest 制作限制仅短剧显示。

## 7. 许可

CHANGELOG / ARCHITECTURE 注明工艺来源 AI-drama-pound。运行时 prompt 用简体，不复制繁体原文。
