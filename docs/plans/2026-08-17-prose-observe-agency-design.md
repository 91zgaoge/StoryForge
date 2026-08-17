# 手写/粘贴正文触发三角色观察

日期：2026-08-17
状态：已落地（v0.51.0）
决策来源：正文进编辑器后自动保存、自动分章会跑，代理工作室三角色不开工。用户裁定方案 1：专用观察编排。

承接：

- `docs/plans/2026-08-13-agency-only-continuation-design.md`
- `docs/plans/2026-08-16-agency-continue-loop-design.md`

## 1. 问题

手写或粘贴正文走 `update_scene` → 30s 空闲自动分章 + AutoIngest/auto_commit。三角色（管理 / 主创 / 编辑审计）只在创世或点续写时建 run、写时间线。工作室看不到他们在动，节拍卡与审查也不会根据作者已写正文准备下一拍。

## 2. 已拍板约束

| 项 | 决定 |
|---|---|
| 谁动 | 三个都要动 |
| 主创 | 不改正文；编译当前场大纲 + 下一拍节拍卡（0 LLM） |
| 管理 | 资产回流（角色/关系/世界观/大纲） |
| 编辑 | 审查正文，只写审查区，不改写 |
| 触发 | 与自动分章同一 30s 空闲窗口 |
| 重跑 | 该 scene 比上次观察多出 ≥200 字；上一轮观察还在跑则跳过 |
| Run | 每故事一条 premise=`观察` 的 run，后续追加同一时间线 |
| 让路 | 该故事有进行中的创世/续写则本轮不观察 |

## 3. 架构

```
update_scene（content 变更）
  → 不再立刻 spawn SceneIngestor（避免与观察双烧）
  → schedule_commit_and_split（30s）
       → maybe_split
       → auto_commit（合同投影 / mini_review 照旧）
       → decide_post_commit_work
            ├ Observe → run_observe（管理 ingest → 主创编译 → 编辑后台审查）
            ├ Ingest  → SceneIngestor::spawn_ingest_now（未达 200 字或创世/续写占用）
            └ Skip    → 观察已在跑 / 无正文
```

不走 `PersistMode::Append/NextChapter`，不加续写拍数，不写 `scenes.content`。

观察 run id 稳定为 `observe-{story_id}`。status 用 `observing` / `idle`（**不用** `pending`/`running`，以免撞上 V109「每故事一个进行中 run」部分唯一索引，挡住用户点续写）。水印在 `result_json`：`{"kind":"observe","by_scene":{scene_id: chars}}`。

## 4. 三角色写回

**管理**：复用资产回流（IngestPipeline + 资产桥 + KG）。姓名须在正文中。手工字段不覆盖。投影到观察 run 资产栏。失败 `done 失败`，不回滚正文。

**主创**：等管理本轮结束（失败也编，用现有资产）。`compile_beat_card_located`。`【当前场大纲】` 写入 `outline_content`：已有手写前缀保留，只替换/追加该块。可回写出场/地点。投影节拍卡到资产栏。

**编辑**：与管理并行（只看正文）。`evaluate_gate_impl`，不发 `genesis-qc-result`（避免当成创世质检 toast）。审查进观察 run Review 区；下次续写 `load_open_review_issues` 已按故事读取。

## 5. 失败与静默

- 观察 LLM 标签 `bg-observe-editor` 加入 `is_silent_background`（管理走既有「记忆-内容分析」静默）。
- 创世/续写 `status=running` 且 premise≠观察 → 本轮 Ingest 而非 Observe。
- 进程内 per-story 锁防止同窗叠两个观察。
- 水印在观察轮结束时更新（含管理失败），避免同一段反复重烧。
- 已知限制：整章替换但字数未多 200 不重跑；auto_commit 的 KG/mini_review 仍可能与观察管理各烧一轮（不在本版拆）。

## 6. 验收探针

| 契约 | 期望 |
|---|---|
| `should_observe` | 0→199 否；0→200 是；500→650 否；500→700 是；分 scene 独立 |
| `merge_current_scene_outline` | 空手写块；前缀+旧块只换块；无标记则追加 |
| `apply_observe_writer` | 有 `【当前场大纲】`；`scenes.content` 不变；拍数不变；复用同一 observe run id |
| `has_blocking_creative_run` | 续写 running / 创世 pending 为真；仅观察 observing 为假；observing 与续写 running 可并存（不撞 V109） |
| `should_spawn_ingest_on_update` | 有 content 变更则保存当时不立刻 ingest |

真机：粘贴 ≥200 字，停手 30s，工作室「观察」run 出现管理/主创/编辑 start+done，正文不被改写。
