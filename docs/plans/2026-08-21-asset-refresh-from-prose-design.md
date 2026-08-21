# 按正文重写生产资产（Asset Refresh）

**状态**：已批准并实施 v0.53.0。  
**日期**：2026-08-21  
**对照版本**：StoryMoss v0.52.0 → v0.53.0  
**用户原句**：幕前输入「将故事大纲按照现有正文重新写过」；同一版须能处理故事大纲 / 角色 / 世界观 / 场景大纲。

---

## 0. 结论（当前为何做不到）

幕前智能输入只有四类：创世、续写、改写、审查。没有「改资产、不改正文」。

| 分类结果 | 实际效果 |
| --- | --- |
| 续写（含 LLM 失败兜底） | Agency Append，往当前章追加正文 |
| 改写 | `final_content` 进纸面，不写 `story_outlines` |
| 审查 | `result_kind=audit_report` 弹窗，不改资产表 |
| `outline_planner` | prose 默认强制改 `writer`；即便跑完也不调用 `update_story_outline` |

`ensure_story_outline` 只在大纲缺失、或大纲点名了正文没有的人时，作为**续写副作用**补一份。大纲已接地但结构错、膨胀、过时，不会重写。观察（v0.51.0）只编译当前场 `【当前场大纲】` 块。

手改路径仍可用：幕后作品页保存故事大纲。那不是这条指令。

---

## 1. 目标与非目标

**目标**：用户用自然语言点名一类或多类生产资产，系统只根据 `scenes.content` 重写那些表/字段，纸面一字不改。

**同一版四个靶**：

| 靶 | 落点 |
| --- | --- |
| 故事大纲 | `story_outlines.content` 整份替换 |
| 角色 | `characters`（及点名关系）源感知合并 |
| 世界观 | `world_buildings` 源感知合并 |
| 场景大纲 | 当前打开场的 `scenes.outline_content`（只替换 `【当前场大纲】` 块） |

**非目标（本版不做）**：伏笔账本、合同红线、删除脏角色行、嵌入 OpenViking、热路径 IntentAnalyzer、改 `WriteTimeBundle::to_prompt()`。

**不得宣称**：续写质量 / 唱反调已修复。本功能只保证「指令改对表、不改正文」。

---

## 2. 不变量

1. `scenes.content` 是唯一叙事真相。本作业只读正文，禁止 `update_scene.content` / Append / NextChapter。
2. 有实质正文才跑；空章返回明确错误，不发明情节。
3. 落地人名、地名、转折必须能在正文（或本章正文，对场景大纲）里对上；对不上的丢掉（对齐 v0.49.0 `title_inventions_dropped_when_absent_from_prose`）。
4. `characters.source` / `world_buildings.source` 为 `user_created` 或 `manual` 的**已填字段**只填空，不覆盖。指令含「覆盖手改」才允许精炼这些字段。
5. 机器来源（`ingest` / `agency` / `auto_placeholder`）允许覆盖非空字段。
6. 禁止 DELETE 角色行（脏行债务不在本版清）。正文新出现的人名可 `source=agency` 注册。
7. `story_outlines` 无 `source` 列：用户说「重新写过」即整份替换，并走既有规划切断 + 转折点封顶（v0.52.0 `cap_story_outline_content`）。
8. 场景大纲保留 `【当前场大纲】` 之前的手写前缀（复用 `merge_current_scene_outline`）。
9. 创世/续写 `running` 时拒绝本作业，提示等底栏结束。观察 run 不挡。
10. 分类为本作业时：`is_continuation=false` 且 `is_prose_request=false`。禁止沿用「续写/创世则强制 prose」的后置纠正。
11. 前端 `result_kind="asset_refresh"`：toast / 短报告，**不** `appendAiContent`。

---

## 3. 意图：点名才动，不是一句话改四张表

「将故事大纲按照现有正文重新写过」只动故事大纲。

同一版**能**处理另外三类，靠同一条路由 + 靶解析：

| 输入信号 | 靶 |
| --- | --- |
| 故事大纲 / 整书大纲 / 书纲 | `story_outline` |
| 角色 / 人物卡 / 人设 | `characters` |
| 世界观 / 世界设定 | `world` |
| 场景大纲 / 本章大纲 / 当场大纲 | `scene_outline` |
| 全部设定 / 所有资产 / 按正文重写设定 | 四靶 |

无靶可解析：不默认全开，返回「请说明要重写故事大纲、角色、世界观还是场景大纲」。

解析用确定性关键词（0 LLM），分类器用 LLM 只判「这是不是资产作业」。二者都过才进热路径。关键词单独命中但分类器判续写：以后置纠正为准——输入含「按照现有正文」+（重写/重新写）+ 上表靶词，强制 `asset_refresh`，避免再被续写兜底吃掉。

---

## 4. 架构

```
幕前指令
  → classify_writing_intent
  → parse_asset_refresh_targets(user_input)   // 0 LLM
  → 后置纠正（禁止当续写/prose）
smart_execute
  → 先于 should_agency_append_continue
  → run_asset_refresh(story_id, scene_id, targets, prose)
       Producer 单次 complete_json（只要点名的键）
       接地过滤
       按靶 persist
  → PlanExecutionResult { result_kind: asset_refresh, final_content: 摘要 }
幕前 toast，invalidate 故事/角色/世界/场景查询
```

角色：管理（Producer / Tool 档），不跑主创 Writer，不跑 tool_loop。

正文预算：`concat_story_prose` 全章拼接后按字符封顶（建议 12000）：优先保各章 `outline_content` 一行 + 近文章末双窗（开篇 600 + 近文 1800，与续写散文窗一致）。超预算从最早章正文砍，不砍当前打开章。场景大纲靶额外注入**当前场全文**（短章）或当前场双窗。

---

## 5. 契约表

| 步骤 | 输入 | 输出 | 失败 |
| --- | --- | --- | --- |
| 分类 | 用户句 | `task_type=asset_refresh`，非 continuation | LLM 失败且关键词命中 → 仍走本作业，不走续写兜底 |
| 靶 | 同一句 | `Vec<AssetRefreshTarget>` | 空 → UserAction 文案 |
| 正文门 | 故事 scenes | 有实质正文 | 无 → 请先写章节 |
| 占用门 | agency runs | 无 blocking creative | 有 → 等续写结束 |
| LLM | 靶 + 预算正文 | JSON，缺键视为该靶跳过 | 解析失败 → 不写库，报失败 |
| 故事大纲 | JSON.story_outline | UPDATE/INSERT `story_outlines`，cap | 人名未接地 → 丢掉发明段 |
| 角色 | JSON.characters[] | `asset_bridge` 合并；新名注册 | 未接地名丢弃 |
| 世界 | JSON.world | `world_buildings` 合并 | 空对象跳过 |
| 场景大纲 | JSON.scene_outline | merge 当前 scene | 无 scene_id → 该靶失败，其它靶仍可写 |
| 前端 | result_kind | toast，纸面不变 | 缺 kind 当正文是回归，测试锁死 |

---

## 6. JSON 形状（Producer）

与 ingest 字段对齐，便于复用 `sync_assets_from_analysis` 的合并函数，但**显式刷新**走独立 persist，避免误触发无界大纲追加。

```json
{
  "story_outline": "【核心冲突】…\n【转折点】…",
  "characters": [
    {
      "name": "阿苔",
      "identity": "…",
      "personality": "…",
      "emotional_core": "…",
      "physical_state": "…",
      "mental_state": "…"
    }
  ],
  "world": {
    "concept": "…",
    "rules": ["…"],
    "history": "…"
  },
  "scene_outline": "在场：…\n冲突：…\n下一拍：…"
}
```

未点名的键必须是 `null` 或不出现。Prompt 写明：只归纳正文；禁止按书名另起一套人。

---

## 7. 前端

对齐 `audit_report`：`FrontstageApp` 在打字机/`appendAiContent` 之前识别 `result_kind === 'asset_refresh'`，toast 摘要（「已按正文重写故事大纲」+ 各靶条数），`queryClient.invalidateQueries` 对应键。不打开审计弹窗。幕后再打开作品/角色页应已是新内容。

---

## 8. 测试（验收探针，未跑通不得称可上线）

1. `parse_asset_refresh_targets("将故事大纲按照现有正文重新写过") == [StoryOutline]`
2. 同句分类后置纠正：`is_continuation=false`，`is_prose_request=false`
3. `should_agency_append_continue` 对该分类为 false
4. persist：`story_outlines` 变、`scenes.content` 字节级不变
5. 大纲发明「金敏秀」而正文没有 → 丢掉
6. `user_created` 角色已填 `emotional_core` → 刷新后仍是原值
7. `ingest` 角色 → 允许被正文精炼
8. 场景大纲：前缀「用户手写」保留，`【当前场大纲】` 更新
9. 前端：`result_kind=asset_refresh` 不渲染进编辑器（vitest）
10. 「把角色和世界观按正文重写」→ 两靶，不动 `story_outlines`（除非本就该改）

真机：同一开头输入原句，看幕后大纲变、幕前字数不变。

---

## 9. 风险

- 长书 12000 字预算仍会漏早期章。接受：故事大纲以近文+各章大纲行为主；用户要全书密读需以后加「分章归纳再合成」（本版不做）。
- 整份替换故事大纲会盖掉幕后手改。这是「重新写过」的字面含义，USER_GUIDE 写明。
- Producer 与前台续写抢本地模型：本作业用户在等，用 Tool 档；若续写占用则直接拒绝，不排队。

---

## 10. 版本

落地为 **v0.53.0**。四源版本 + `FALLBACK_VERSION` + docs of record。
