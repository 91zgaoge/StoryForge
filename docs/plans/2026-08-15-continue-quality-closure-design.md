# 续写质量闭合：角色 / 推进 / 前后文

日期：2026-08-15
状态：已发版 v0.48.0（v0.47.0 真机 §8.2 已失败；0.48.0 修镜头/落库/幽灵分叉；须在 0.48.0 上重跑真机，不得宣称症状已修复）
决策来源：续写仍存在人物丢失错配、情节推进缓慢、前后文逻辑断裂；对照 v0.41–v0.45 现网代码全面审计后，用户裁定 **P0–P3 一次出设计与实施方案**。

承接：

- `docs/plans/2026-08-13-agency-only-continuation-design.md`（Agency 唯一路径 + SceneBeatCard + Append）
- `docs/plans/2026-08-14-continue-prompt-asset-selection-design.md`（按拍选取资产）
- `docs/plans/2026-08-14-prompt-runtime-assembly-design.md`（`assemble_continue_beat`）

本设计 **不改** 创世/续写唯一编排入口、`PersistMode`、热路径 0 LLM 选资产、`WriteTimeBundle::to_prompt()` 全局语义。不宣称设计 §13 八次真机探针已过，直到 P3 跑通。

## 1. 问题

### 1.1 用户可感知症状

| 症状 | 读者看到什么 |
|---|---|
| 人物丢失 / 错配 | 在场的人消失；不该出现的人突然在场；称呼对不上角色表（阿砚 / 沈砚） |
| 情节推进缓慢 | 连续多拍同一地点同一两人对话；大纲下一节点不推进；冲突不加压 |
| 前后文断裂 | 章中已发生的换场/承诺被忘掉；跨章线索丢；末句接得上但剧情接不上 |

### 1.2 根因（代码核对，非会话推测）

架构（Agency + 节拍卡 + 资产筛选 + 开篇/近文双窗）已对准三类问题。复发是因为**编译器与写回空转**：

1. **债务被自己清零**：`persist::touch_refresh_beats` 只要卡上写得出冲突/阵容/地点就把 `last_*_beat` 设成当前拍号。缺口恒为 0，扩张配额几乎永不触发。设计 §11.6「连续 2 次 Append 冲突列仍空 → 第三次卡含冲突配额」无测试保护。
2. **下一节点会退回大纲首句**：`compile_next_node` 用候选句前 8 字是否出现在近场 `outline_content` 判断覆盖；`merge_progress_line` **覆盖**上一条「进度：」；全覆盖兜底 `candidates.first()`。
3. **冲突不看本拍阵容**：`compile_conflict` 取关系表第一条敌对边，可点名场外仇敌。
4. **写回的是计划不是事实**：`characters_present` 写成 `BeatCard.cast`（含被强制塞入的沉寂/张力角色），下一拍信以为真。
5. **配额渲染成 Debug 英文**：`format!("{:?}", QuotaItem)` → `ForeshadowMove`，不是中文硬任务。
6. **散文回退走创世模板**：`write_beat_once` 过短时 `assemble_genesis_prose_fallback`，丢掉节拍卡与末句锚点。
7. **在场判定是子串**：`text.contains(角色名)`；无别名；短名误伤。
8. **沉寂/张力强制入场**：不问能否到达本场。
9. **地点冻结**：卡取全书最后一条非空 `setting_location` 再写回同一值。
10. **规范状态白加载**：Bundle 已有 `active_conflicts` / `character_goals`，`render_continue_assets` 不拼接。
11. **末句锚点声明最高优先级**，压过节拍推进。
12. **无未决线程账本、无生成后验收**。角色动态位置靠异步 LLM ingest，下一拍常为空。

`.recovery/` 里 8 月 4–6 日会话发生在 Agency 唯一路径上线前，只作历史症状参考，不作本版失败证据。v0.41 设计 §13 真机探针从未跑过。

## 2. 目标与非目标

**目标**：每一拍续写在现有 Agency 热路径上，强制落实「谁在场、冲突做什么、推进到哪、前文因果不断」；写回反映正文事实；债务按正文变化计拍；连续 8 次幕前 Append 有可执行验收。

**非目标**：

- 不恢复 TimeSliced / TriShot 续写。
- 不把 Inspector / Editor 拉回同步等待。
- 不在热路径加 LLM 选资产（沿用 v0.42.0）。
- 不接 `ContextPrioritizer`（仍记债务；本设计用节拍卡 + 状态网双锚 + 降低末句优先级）。
- 不清理 `characters` 跨故事脏行；不改 `story_outlines` 表内无界追加（注入层去重已有）。
- 不新增角色别名列 / 不新表（别名由姓名规则 + 正文点名编译，0 迁移）。
- P2 生成后重试最多 **一次** `complete()`；失败仍落库（≥200 字），不丢稿。

## 3. 不变量（相对既有设计）

沿用并收紧：

| 项 | 不变量 |
|---|---|
| 路由 | 创世/续写只进 `AgencyCoordinator`。划词改写仍走 PlanExecutor。 |
| 落库 | 幕前 / 文思活跃 = `PersistMode::Append`；幕后续写一章 = `NextChapter`。 |
| 增量 | 返回前端仍是本拍增量；≥200 字才落库。 |
| 热路径 LLM | 主创一次 `complete()`；P2 探针失败可再一次；编辑后台。禁止为选人/选资产加调用。 |
| 资产筛选 | `render_continue_assets` 预算 ≤6000；完整卡 ≤8；脏名不进名单。 |
| 前文窗口 | 短章全文；长章开篇 600 + 近文 1800。不叠更早几章全文（P2 用未决线程补偿，不恢复三场正文）。 |
| 架构边界 | `agency` 可依赖 `db` / `creative_engine` / `prompts`；不把 `db` 指回 `agency`。 |

META-0：作废「有卡字段就视为本拍已刷新债务」。新不变量：**只有正文兑现了变化，才清对应债务**。

## 4. 阶段总览

每阶段结束后幕前续写必须仍可运行。阶段 0 关闭「结构上必现」；阶段 1 关闭身份与场次；阶段 2 关闭因果账本与验收重试；阶段 3 关闭「未跑探针却宣称已修复」。

```
P0 编译器修洞 ──→ P1 身份与场次 ──→ P2 拍级状态网 ──→ P3 八拍验收
     0 额外 LLM         0 额外 LLM         至多 +1 次 complete      mock 必过 / live 人工
```

---

## 5. P0 — 编译器修洞

### 5.1 债务只在正文兑现时刷新

`increment_append_beat` 仍每次成功 Append/NextChapter +1。

删除「只要 `SceneUpdate` 含冲突/阵容/地点就 `touch_refresh_beats`」。改为纯函数（0 I/O）：

```
fn beat_refresh_flags(
    increment: &str,
    matched_names: &[String],
    prev_present: &[String],
    prev_location: Option<&str>,
    new_location: Option<&str>,
    conflict_parties: &[String],
    foreshadow_needles: &[String],
) -> RefreshFlags { conflict, cast, location, foreshadow }
```

规则（全部针对 **本拍增量**，不含旧文）：

| 旗标 | 为 true 当且仅当 |
|---|---|
| `conflict` | 冲突双方（≥2）都在 `matched_names` 中，**或** 增量含加压词（对峙/反转/代价/冲突/对打/逼迫，词表可单测） |
| `cast` | `matched_names` 作为集合与 `prev_present` 不等（有人新进或离场） |
| `location` | `new_location` 非空且与 `prev_location` 不等 |
| `foreshadow` | 任一条 `foreshadow_needles`（待回收/逾期伏笔标题截断 8 字）出现在增量中 |

`last_*_beat` 只在对应旗标为 true 时写成当前 `append_beats`。否则保持旧值。旧书 `last_*=0` 且 `append_beats>0` 仍按现有 `from_beats`：缺口 = `append_beats`（不再当零干扰）。

契约：同一 scene 连续 2 次 Append，增量不含冲突双方也不含加压词 → 第三次 `compile_beat_card` 的配额含 `ConflictEscalation`。

### 5.2 配额中文硬任务

`SceneBeatCard` 增加 `expansion_quota_text: Option<String>`。值为 `ExpansionDebt::quota_text()` 改写单位后的文本：把「章」换成「拍」，标题改为「【本拍扩张任务（硬性要求，必须落实）】」。

**不改** `ExpansionDebt::quota_text` 原文（planner 改写路径仍用「章」）。agency 侧新增 `quota_text_for_beats(debt) -> Option<String>`。

`render_full` 渲染该中文段。禁止 `format!("{:?}", QuotaItem)` 进主创 prompt。

### 5.3 下一节点不得回绕

`merge_progress_line`：**追加** 一条尚未存在的 `进度：{node}`，不覆盖历史。重复 node（trim 后相同）不追加。单场 `outline_content` 硬上限 2000 字，超出丢掉最旧的「进度：」行、保留手写大纲前缀。

`compile_next_node`：

1. `covered` = 近 3 场 `outline_content` 全文（含全部进度行）。
2. 候选句：按 `。！？；\n` 切，长度 ≥8 字（不是 4）。
3. 覆盖判定：候选全文或前 **20** 字出现在 `covered` 中（不是 8 字）。
4. 返回第一条未覆盖候选，截断 200 字。
5. 全部覆盖或无候选：返回固定句「在硬约束内把当前冲突推进一步，不得原地复述末句。」**禁止** `candidates.first()`。

### 5.4 冲突必须落在本拍阵容

`compile_conflict` 在阵容编完之后调用，签名增加 `cast: &[CastMember]`。

只采用两端都能在 `cast` 名字里解析到的敌对关系（关系表 `source` 经 id→名，`target` 用名）。多条时取第一条（保持确定性）。

无场内敌对边：降级句「{阵容第一人} 必须在本拍让需求受阻并付出代价，不得只靠对话过渡」，`parties` = 阵容前 1–2 名。P0 **不**为这条去读伏笔表（避免 `beat_card` 新依赖）；逾期伏笔进状态网是 P2。

禁止点名完全不在 `cast` 里的人。

P0 **不改** 沉寂/张力强制入场（P1 改）。冲突在入场补位之后计算，故张力塞进来的人仍可能成为冲突双方——这是 P0 可接受的残余，由 P1 关掉。

### 5.5 写回事实出场

`persist_append_with_card` / `scene_fields_from_card`：

`characters_present` = `match_character_names(角色表名, increment)`（P0 先用全名 `contains`，别名在 P1）。

- 非空 → 写这些名（禁止 UUID）。
- 空 → 保留 scene 原 `characters_present`，**不**用卡上计划阵容覆盖。
- 冲突 JSON：仅当 `beat_refresh_flags.conflict` 为 true 才写新 `character_conflicts`；否则保留旧列。
- 地点：P0 仍可写卡上 `setting_location`（P1 改为近文抽取）。债务刷新仍按 5.1，写列 ≠ 清债务。

更新既有测试 `append_writeback_sets_characters_present_names`：增量正文必须点名「阿岩」「林雪」。

### 5.6 续写散文回退不得走创世模板

`write_beat_once` 主创过短（<200）时：

1. 仍先 `sanitize_novel_output`。
2. 回退改为对 **同一** `assemble_continue_beat` 的 system+user 再 `complete()` 一次，user 前追加一行：「只输出小说正文，承接末句，落实节拍任务，禁止分析/提纲/创世开篇。」
3. 仍短才 `Err`，不调用 `writer_prose_fallback`（创世模板）。
4. `write_chapter`（批量 NextChapter）熔断回退同样禁止创世模板；可复用同一 continue user。

`writer_prose_fallback` 保留给 **创世** `genesis_fastpath` / legacy。续写路径零引用。

契约：续写回退请求的 user 含 `【本章节拍任务】` 与末句锚点；不含「故事前提：」创世段。

---

## 6. P1 — 身份与场次

### 6.1 别名点名（0 迁移）

新增 `continue_assets::aliases_for(name) -> Vec<String>` 与 `match_character_names(names, text) -> Vec<String>`。

别名规则（最长优先，禁止单字）：

| 姓名字数 | 别名 |
|---|---|
| 2（沈砚） | 沈砚、阿砚（阿+末字） |
| ≥3（司马昭） | 全名、末两字、阿+末字 |

消歧：

- 别名不得长度为 1。
- 若别名等于另一角色的**全名**，该别名只归属全名角色。
- 扫描按别名长度降序；命中一次即绑定规范名，不再用更短别名抢同一跨度。

`present_in_text`、准入 `names_in_text`、P0 写回、P2 探针全部改走 `match_character_names`。

### 6.2 沉寂 / 张力不得无故传送

`compile_beat_card` 阵容：

1. `present` = 近文窗口点名（别名匹配）。
2. **不要**无条件 `push` 沉寂最长者。
3. **不要**无条件 `push` 全表最高张力双方。仅当张力边至少一端已在 `present` 时，把另一端加入，purpose = `张力对手（类型），须已在场或写明闯入本场`。
4. 若 `expansion_quota` 含 `CharacterMove`：加入沉寂最长且不在场者 1 人，purpose 必须含「沉寂回归」且「须写明入场（来到/被召/闯入）」。
5. 人数 <3 且角色表 ≥3：从角色表补位，但 purpose = `补位，仅当能出现在本场时上场`。仍 truncate 8。

更新测试 `beat_card_cast_includes_silent_character_when_three_exist`：无 `CharacterMove` 时沉寂名可以不在 `cast`；有配额时必须在且 purpose 含「入场」。

### 6.3 地点从近文兑现

已知地点表 = 本故事所有 `scenes.setting_location` 非空去重 ∪ 当前卡上旧地点。

`detect_location_shift(known, prev, increment) -> Option<String>`：在增量中出现的已知地点里，取**最后一次出现**的那个；若与 `prev` 不同则返回它。未知地名不发明。

写回 `setting_location`：有 shift 用新值；否则保留旧值（不把冻结的「全书最后一条地点」反复写回）。

`compile_beat_card` 的 `setting_location` 改为：**当前 Append 场景自己的地点**（传入 scene 或从 `current_content` 所属 scene 读），不是 `get_by_story` 倒序第一条非空。NextChapter 用最近一场地点作为「上一场」，新地点仅当增量检测有 shift。

为此 `compile_beat_card` 增加参数 `current_scene_location: Option<&str>`（或从 pool+scene_id 读）。`write_beat_once` 在 Append 时传入当前 scene 的地点。

### 6.4 活跃冲突 / 角色目标进筛选器

`render_continue_assets` 拼接列表在关系段之后、本章大纲之前插入：

- `active_conflicts`：原文截断 400 字；若能按录取名过滤则过滤，不能则整段保留（规范状态文本多为散文）。
- `character_goals`：按录取角色名行过滤；截断 400 字。

两段计入 6000 预算，优先级低于红线/录取卡/名单，高于前文（超预算时先裁前文）。

`ContinueAssetsInput` 不新加字段也可以：直接从 `input.bundle.active_conflicts` / `character_goals` 读。测试夹具今日为 `None`，补一条非空断言。

---

## 7. P2 — 拍级状态网

### 7.1 `BeatState`

新文件 `src-tauri/src/agency/beat_state.rs`，0 LLM、0 I/O（除编译时已有的 DB 快照由调用方传入）。

```
struct OpenThread { text: String }  // ≤80 字
struct BeatState {
    present: Vec<String>,
    locations: Vec<(String, String)>, // 角色规范名 → 地点
    threads: Vec<OpenThread>,         // ≤5
}
```

编译输入：录取名、本场地点、`next_outline_node`、逾期伏笔、近文 1800 字、近 3 场进度行。

未决线程来源（去重，先到先得，满 5 停）：

1. `next_outline_node`（一条）。
2. 逾期伏笔各一条（标题 ≤80 字）。
3. 近文中含期限/强制信号的句子（「必须」「之前」「否则」「子时」「七日」），每句 ≤80 字，最多补满。

`locations`：在场者全部映射到本场地点；不在场者不写（避免把人钉在错误地点）。

渲染：

```
【本拍状态网】
在场：…
地点：角色=地点；…
未决：1. … 2. …
必须承接未决，禁止忘掉已在场者，禁止把未决线程当已解决除非本拍写明解决。
```

头尾双锚：`render_writer_user_prompt` 在节拍卡全文之后立刻插入状态网全文；尾部在节拍摘要之后、末句锚点之前插入四行摘要（在场 / 地点 / 未决首条 / 推进）。

### 7.2 末句锚点降权

`ending_anchor` 文案去掉「最高优先级，覆盖上方任何开场指令」。改为：

「正文已写到此处，下一句必须无缝衔接（禁止另起开篇/醒来/失忆）。人物、地点、未决问题以节拍任务与状态网为准；末句只约束句法衔接。」

顺序固定：`节拍卡全文 → 状态网全文 → 资产 → 指令 → 节拍摘要 → 状态网摘要 → 末句锚点`。

测试 `writer_prompt_order_is_card_then_body_then_summary_then_ending` 扩展为断言状态网在摘要之前、末句在最后，且不含「最高优先级」。

### 7.3 生成后探针 + 一次重试

纯函数 `probe_increment(increment, card, state, quota) -> BeatProbe`：

| 检查 | 失败缺口文案 |
|---|---|
| `match_character_names` 命中 < 2 且 `card.cast` ≥ 2 | 增量点名在场者不足 2 人 |
| 配额含 `ConflictEscalation` 且双方未同时出现、亦无加压词 | 未落实冲突加压 |
| 配额含 `NewScene` 且地点未 shift | 未离开当前场景 |
| 配额含 `CharacterMove` 且沉寂名未出现 | 沉寂角色未入场 |
| `state.present` 中近文已在场者，增量既未点名也未写离场 | 丢掉已在场者 |

`gaps` 非空且增量 ≥200 字：对同一 system，user 追加「【缺口（必须在正文里补上，不要解释）】」+ gaps，再 `complete()` **一次**。两次结果取探针缺口更少者；平手取更长者。仍有缺口：落库并 `log::warn`，不 Err。

增量 <200：走 P0 续写回退，不在回退上再套探针重试（避免 3 次 LLM）。回退后若 ≥200 再跑一次探针（只记录，不再 complete）。

### 7.4 规则回写角色位置

Append/NextChapter 成功且本场地点非空：对 `characters_present` 中每个规范名，`CharacterRepository::update_character_state` 只填 `location = Some(本场地)`，其余 `None`（COALESCE 保留）。失败 `log::warn`，不阻断落库。

不在场者的 location **不在本拍清空**（避免误删）。ingest 仍异步跑，不得覆盖本拍刚写的 location（ingest 源感知：若 `last_updated` 在本拍之后且 location 非空，ingest 跳过 location 列）。若 ingest 改动成本高：本设计允许 ingest 稍后覆盖，P2 最低要求是「下一拍 Bundle 能读到本拍写的 location」。以 `update_character_state` 的 COALESCE 为准；ingest 若整行 REPLACE 会打架——实施时读 `persist_character_states`：若是 INSERT OR REPLACE 整行，则只在 location 为空时让 ingest 写 location。

---

## 8. P3 — 八拍验收

### 8.1 确定性 mock（CI 必跑）

扩展 `agency/eval_harness.rs`（或 `agency/tests.rs`）一条流程测试，**不启真实 LLM**：

种子：3 名角色、仇敌边、故事大纲 ≥3 句节点、1 个 scene。Mock writer 队列 8 段增量，每段 ≥200 字，交替：前 2 段只重复对话不含冲突双方；第 3 段落实冲突；后几段换地点、点名第三人。

断言：

1. 8 次 `PersistMode::Append` 后 `scenes` 行数仍为 1。
2. 第 3 次发给主创的 user 含「扩张任务」或「冲突」配额中文（前 2 次增量未兑现冲突）。
3. `compile_next_node` 在进度行累积后不返回大纲第一句（除非第一句仍是未覆盖的下一节点）。
4. 第 3 次及之后 `characters_present` 来自增量点名，不含从未在增量出现的第四人。
5. 第 6 次增量含新已知地点时 `setting_location` 更新。
6. 主创 user 含状态网（P2 之后）；不含「最高优先级」。

P0 落地时本测试可先只断言 1–3；P1 补 4–5；P2 补 6。P3 把它们收成一条命名为 `eight_beat_append_quality_contract` 的测试，作为设计验收探针的 **CI 替代物**。

### 8.2 真机探针（人工闸门）

沿用 v0.41 §13 指标，在同一用户故事上连续 8 次幕前续写（需 LLM）：

| 指标 | 目标 |
|---|---|
| 单次增量具名角色数 | 中位 ≥3（角色表 ≥3 时） |
| 可识别冲突动作 | ≥80% |
| 不原地复述末句 | 抽检通过 |
| `next_outline_node` 在 Critical 段 | 每次 prompt 都有 |
| scenes 行数 | 自动分章未触发时不增加 |
| 正文应出现地点或阵容变化 | 8 次内 ≥1 次 |

证据：`creative_workflow.log` + DB。未跑通 **不得** 在 CHANGELOG / README 宣称三症状已修复。P3 实施只提供检查清单与 mock 契约；真机由人类在发版前跑，结果写入 `docs/audits/` 或 ROADMAP。

**v0.47.0 真机（executed，2026-08-16 晨 CST，《帝国的烟火》）失败**：三次幕前续写；`gaps` 含丢掉曹元佩/苏亦铁/苏明远仍落库；连续续写丢弃幽灵并用同一 `current_content_len: 6615` 覆盖；`compile_next_node` 把书大纲皇权/毒杀句塞进喜宴。v0.48.0 针对这几条机械原因修补，**不是** §8.2 通过。

### 8.3 v0.48.0 真机失败后的修补（相对本设计）

| 项 | 设计原文 | v0.48.0 |
|---|---|---|
| 在场窗口 | 近文 1800 | 散文仍 1800；阵容只看末 500 |
| 阵容补位 | 沉寂/张力门闩 | 删除按角色表顺序「补位上场」 |
| 下一节点 | 不回绕首句 | 再跳过不点名本拍在场者的书大纲句 |
| 落库底稿 | 客户端快照 + 增量 | 取 DB 与客户端更长者 |
| 未确认幽灵 | 未规定（v0.26.22 丢弃） | 下一拍续写前写入正文 |
| NewScene × 丢人 | 可同时报 | NewScene 时不报丢掉已在场者 |

## 9. 失败与降级

| 失败 | 行为 |
|---|---|
| 别名冲突 | 全名优先；单字丢弃 |
| 已知地点表为空 | 不 shift；保留旧 `setting_location` |
| 状态网线程 0 条 | 仍输出在场+地点；未决用 `next_outline_node` 兜底一句 |
| 探针重试仍有缺口 | 落库 + warn |
| `update_character_state` 失败 | warn，正文已落库 |
| Bundle 无 active_conflicts | 整段省略，与今日一致 |

熔断不等于丢稿：≥200 字必须尝试写回（沿用 v0.41 §10）。

## 10. 测试契约汇总

每个测试保护用户可感知结果，不是实现细节。

**P0**

1. 连续 2 次 Append 增量无冲突兑现 → 第三次卡含冲突配额中文，不含 `ConflictEscalation` Debug 串。
2. `merge_progress_line` 累积两条不同进度，不覆盖。
3. `compile_next_node` 在全部候选已在进度行中时返回固定推进句，不是大纲首句。
4. 场内无敌对、场外有仇敌 → `conflict_move.parties` 不含场外人。
5. 增量点名甲乙 → `characters_present` 为甲乙；增量无名 → 保留旧列。
6. 续写过短回退的 user 含节拍卡，不含创世「故事前提」。

**P1**

7. 「阿砚」命中角色「沈砚」；「白」不单独命中「白芷」。
8. 无 CharacterMove 时沉寂角色可不在 cast；有则在且 purpose 含入场。
9. 增量最后出现的已知地点与旧地点不同 → 写回新地点。
10. `render_continue_assets` 在 bundle 有 `character_goals` 且录取含该名时，输出含该目标句。

**P2**

11. 状态网含 next_node 与逾期伏笔；在场者地点=本场。
12. prompt 顺序：卡 → 状态网 → … → 摘要 → 状态网摘要 → 末句；无「最高优先级」。
13. 配额要求换场但增量未换 → `gaps` 非空。
14. persist 后在场角色 `character_states.location` = 本场地。

**P3**

15. `eight_beat_append_quality_contract`（§8.1）。

创世既有测试不回退。`cargo test --lib` 全绿。无前端逻辑变更则不强制 vitest；若未改 TS，tsc 仍应绿。

## 11. 文件边界

| 文件 | 职责 |
|---|---|
| `agency/continue_assets.rs` | 别名、点名、地点 shift、活跃冲突/目标段、预算 |
| `agency/beat_card.rs` | 阵容/冲突/节点/配额中文/末句降权/prompt 顺序 |
| `agency/beat_state.rs` | **新建** BeatState 编译与渲染、探针 |
| `agency/persist.rs` | 债务旗标、进度行累积、事实写回、位置回写 |
| `agency/coordinator.rs` | 续写回退改走 continue 组装；P2 探针重试；传入本场地点 |
| `creative_engine/expansion/debt.rs` | **不改** `quota_text` 原文 |
| `prompts/assembly.rs` | 不改创世 fallback；续写不调用它 |

GitNexus：改 `compile_beat_card` / `persist_append_with_card` / `write_beat_once` / `render_continue_assets` / `writer_prose_fallback` 前对每个符号 `impact({target, direction:"upstream"})`。HIGH/CRITICAL 先告知。

## 12. 已知债务（本设计不做）

- ContextPrioritizer 接入 Agency；Lost-in-the-Middle 量化探针（研究前沿）。
- 更早几章全文 / 跨章摘要模型。
- `characters` 脏行清理；`story_outlines` 表内收口。
- `characters_present` 历史 id/名混杂行的一次性迁移（写入口径为本期名；账本已按名匹配）。
- 本地模型连接 60s×2。
- 真机 8 次幕前续写（§8.2）——v0.47.0 已失败；v0.48.0 须重跑。P3 只提供清单与 mock。

## 13. 验收声明

P0–P2 以 §10 契约测试为「可上线」门槛。P3 mock 为设计验收探针的 CI 替代。真机 §8.2 未在当前版本跑通前，文档必须写「症状不得宣称已修复」。v0.47.0 真机失败已记录于 §8.2。
