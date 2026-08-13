# 创世三角色唯一路径：续写资产强关联设计

日期：2026-08-13
状态：已实施
决策来源：续写越写越空（角色淡出 / 无方向 / 无冲突 / 无情感）全面审计；用户裁定创世与幕前/幕后续写只走 Agency 三角色，删除 TimeSliced 与 TriShot；幕前续写/文思活跃为同章追加（PersistMode::Append）。

## 1. 问题与裁定

### 1.1 症状

幕前续写随次数恶化：出场人物变少、情节梦游、无冲突、无情感。后台资产（方法论、提示词、角色/伏笔/风格/合同/记忆图谱）丰富，但默认续写没有把它们编译成「这一拍必须完成的任务」。

### 1.2 根因（代码核对，非会话推测）

1. **三条散文引擎并存**：创世走 Agency；幕前续写走 `PlanExecutor → TimeSliced`（可选 `beat_planner`）；幕后「续写一章」走 `AgencyCoordinator.run_continue`。资产能力不对齐。
2. **注入 ≠ 使用**：`WriteTimeBundle.to_prompt()` 倾倒 13+ 段；`ContextPrioritizer` 只在 Full 路径生效；末句硬锚点在 prompt 最末，模型跟着最后 1–2 人的下一动作走。
3. **写后不回流**：TimeSliced 只写 `scenes.content`。轮换账本/扩张债务读 `characters_present` / `character_conflicts` / `setting_location`，这些列续写从不更新 → 债务恒为 0。
4. **计量单位错误**：债务按「章」计。文思活跃在同一章连续追加，沉寂/冲突停滞永远达不到阈值。
5. **TimeSliced 角色卡缺情感四元组与关系**；Agency 续写有，但幕前走不到。

### 1.3 裁定（已确认）

| 项 | 决定 |
|---|---|
| 创世 / 幕前续写 / 幕后续写 | **只调用** `AgencyCoordinator` 三角色（管理 Producer / 主创 LeadWriter / 编辑审计 EditorAuditor） |
| TimeSliced、TriShot | 从创世与续写路由上 **删除**；有用能力并入三角色 |
| 幕前续写 / 文思活跃 | **PersistMode::Append**：正文追加进**当前章**（同一条 `scenes` 行）；戏剧上必须换场（见 §3.1） |
| 幕后「续写一章」或用户明确下一章 | **PersistMode::NextChapter**：新建一章（新 `scenes` 行） |
| 改写 / 审稿 | 仍走 planner，本次不删 |

### 1.4 架构不变量变更（META-0）

作废 v0.13.0「续写默认 TimeSliced 热路径」。新不变量：

- 创世与续写的唯一编排入口是 `AgencyCoordinator`。
- 幕前每次点续写 **不得新建一章**（Append 到当前章正文）。戏剧换场（换地点、换阵容、换冲突）是续写的硬任务，不是禁项。
- 新的一章只在三种情况下出现：用户明确下一章、幕后「续写一章」、或现有自动分章（按字数 / 按情节）切出。
- 主创在资产已齐时单次 `complete()` 出正文；编辑审计后台化，不挡正文上屏。
- 热路径禁止同步跑 Inspector/Rewrite 闭环（分时「质检下沉」精神保留，实现改为 Editor 后台，而不是第二条 Writer 引擎）。

异议已记录一次：若把今日 `run_continue`（tool_loop + 同步质检 + 按章 create 场景）原样接到幕前，文思活跃会新开几十章且每次等待数分钟。本设计用三角色编排 + Append + 单次主创 + 后台编辑，而不是原样复用慢路径。

## 2. 目标与非目标

**目标**：创世与续写只有一条编排路径；每一拍续写强制落实阵容、冲突、情感、大纲下一节点；写后回流场景元数据，使账本/债务有真源。

**非目标**：

- 不改写/审稿的 planner 路径。
- 不把 Inspector 同步塞回幕前等待。
- 不做 critic 打分回炉。
- 不在本设计内重做创世概念包/深度资产算法（创世已走三角色；只要求续写编译器与创世共享资产读取）。

## 3. 唯一路径

```
用户指令（幕前输入 / 文思活跃 / 幕后按钮）
    │
    ▼
smart_execute  或  agency_continue_chapter / agency_continue_batch
    │
    ├─ is_new_novel     → AgencyCoordinator.run_genesis
    ├─ is_continuation  → AgencyCoordinator.run_continue(persist, instruction)
    │                         ├─ Append { scene_id }      幕前 / 文思活跃
    │                         └─ NextChapter { chapter }  幕后续写一章 / 明确下一章
    └─ rewrite / audit  → PlanExecutor（不变）
```

`PlanExecutor.execute_writer` 在 `is_continuation || is_new_novel` 时 **不得** 再调用 `orchestrator.generate(TimeSliced|TriShot|Fast|Full)`。创世/续写在 `smart_execute` 入口就进入 Agency，与今日创世分支对称。

### 3.1 两个「场景」必须分开说

本产品里「场景」有两层，本设计先前混用，此处改准：

| 词 | 含义 | 续写时的要求 |
|---|---|---|
| **戏剧场次** | 地点、在场人物、本场冲突（读者感知的「换场」） | **必须随情节变换**。SceneBeatCard 的阵容/冲突/地点/扩张配额就是干这个的。连续多拍仍困在同一地点、同一两人对话，视为失败。 |
| **章 / `scenes` 行** | 编辑器里的一章、数据库一条 `scenes` 记录 | **不是**每点一次续写就新建一行。幕前续写把新正文追加进当前章；换场写在正文和元数据里（更新 `setting_location`、`characters_present`）。真正切成新的一章，走已有自动分章（`chapter_split_mode = word_count \| plot`）或用户/幕后显式「下一章」。 |

否则文思活跃点 20 次续写会冒出 20 章，编辑器不断跳章，和今天「一章里往下写、写长了再切章」的用法相反。

情节换场与切章的衔接：BeatCard 若本拍要求离开当前地点（`QuotaItem::NewScene` 或冲突迫使换场），正文必须换场，并更新当前行的 `setting_location`。若设置了「按情节」分章，空闲分章会在转换点把溢出段落切成新章（现有 `chapter_splitter` + 幕前自动切到新章），**不在主创返回的同步路径上立刻 create 新行**。

### 3.2 PersistMode

```rust
enum PersistMode {
    /// 将本拍正文追加到当前章（幕前续写 / 文思活跃）
    Append { scene_id: String },
    /// 新建第 N 章（幕后续写一章 / 用户明确下一章）
    NextChapter { chapter_number: i32 },
}
```

Append 契约：

- 必传当前 `scene_id`；缺失则失败，不猜测、不新建。
- 旧文以 `smart_execute` 传入的 `current_content` 为准（编辑器真源，幕前传 **HTML（`getHTML()`）**），不以 DB 旧行为准，避免覆盖未落库输入。
- 主创只产出**增量**（目标字数仍用 `continuation_target_words` 的 0.7x–1.3x 范围）。返回给前端的 `final_content` 仍是增量，现有 `appendAiContent` 不变。
- 落库全文 = `current_content` 清洗后的正文 + 本拍增量，写入该 `scene_id` 的 `scenes.content`。禁止为这一拍 `create` 新的一章。前端随后 `flushSceneSave` 仍可能用编辑器 HTML 再写一次；`persistSceneContent` 串行，后写覆盖前写，**允许格式化差异**（两端均 HTML 后，persist 与 `flushSceneSave` 仍可能因 TipTap 规范化 / `<p>` 包裹产生标记差异，后写覆盖前写不视为错误）。禁止的是 persist 用 `getText()` 把整章压成纯文本。
- 增量 **&lt;200 字拒落库**（垃圾稿）；主创熔断时 **≥200 字必须尝试写回**——同一门槛的两面。
- `chapter_number` 取该场景的 `sequence_number`，仅用于大纲进度与账本窗口。

NextChapter 契约：保持今日 `run_continue` 的 create+update 单事务装配，但上下文编译与 SceneBeatCard 与 Append 共用。

### 3.3 并发与文思活跃

`idx_agency_runs_one_active_per_story` 保留。一条故事同一时刻只能有一个 active run。

- 正文装配完成（Append 写回或 NextChapter 落库）后 **立即** `finish_run(completed)`，释放锁。
- 编辑审计在 run 结束后后台 spawn，不占用 active run。
- 幕前已有 `smartExecuteInFlightRef` 重入守卫；后端再遇唯一约束仍映射为明确错误，不静默丢稿。
- 文思活跃两次续写之间必须等上一次正文返回；不允许并行两个 Append。

## 4. 三角色在续写中的职责

| 角色 | 做什么 | 不做什么 |
|---|---|---|
| **管理 Producer** | `ensure_assets`（已有则跳过 LLM）；Rust 编译 SceneBeatCard；仅当本章 `outline_content` 为空且 PersistMode::NextChapter 时才 LLM 生成本章大纲 | 不跑 tool_loop 选资产；Append 不强制生成新章节大纲 |
| **主创 LeadWriter** | 单次 `complete()` 写本拍正文；输入 = 编译后的上下文 + SceneBeatCard + 末句锚点 + 用户指令 | 禁止 board_read 轮询；tool_loop 仅在单次失败后的散文回退仍失败时作为最后手段 |
| **编辑审计 Editor** | 后台 `evaluate_gate`；结果 `genesis-qc-result` 同类事件 + toast | 不阻塞幕前上屏；失败则 salvage（≥600 字降级放行）或不改已落库正文 |

创世 `genesis_fastpath` 的 producer-first → writer 单次 → `assemble_only` → `spawn_editor_qc` 是续写应对齐的骨架。续写补的是 SceneBeatCard 与 Append 落库。

## 5. SceneBeatCard（这一拍的硬任务）

Producer 在写前用纯 Rust 从 DB 编译，**默认 0 次 LLM**。缺关键字段时才允许一次不超过 15s 的补全调用；失败则用规则卡降级，不阻断续写。

```rust
struct SceneBeatCard {
    /// 本拍上场 3–8 人，每人一句行动目的；必须含 ≥1 名沉寂角色（若故事角色 ≥3）
    cast: Vec<CastMember>,          // name + purpose
    /// 本拍冲突：加压 / 反转 / 代价显现，点名冲突双方与赌注
    conflict_move: ConflictMove,
    /// 本拍情感：点名角色伤口/需求，或一对关系的真实情感；情绪起止
    emotion_beat: EmotionBeat,
    /// 从故事大纲抽出的「本拍应推进到的下一节点」（具体句子，≤200 字）
    next_outline_node: String,
    /// 扩张债务触发项（0–4）；空则整段省略
    expansion_quota: Vec<QuotaItem>,
}
```

编译规则（确定性，可单测）：

1. **阵容**：当前场景末段已出现的角色 ∩ 角色表，补沉寂最长者 1 人，再补关系张力最高的对手/盟友，裁到 3–8。禁止把全员标成「登场角色」。
2. **冲突**：优先 `character_relationships` 中含仇敌/对立/背叛/欺骗/复仇及英文 enemy/rival/conflict 的边；否则用规范状态 `active_conflicts`；再否则用逾期伏笔制造对峙。必须产出一条可执行动作，不得输出空段。
3. **情感**：优先上场角色的 `emotional_core/trigger/wound/need`；否则用关系 `emotional_bond` 不对等；再否则用「本拍必须让 X 的需求受阻」。
4. **下一节点**：故事大纲中尚未被近 3 章 `outline_content` 覆盖的下一段（规则：最长公共子串/关键词覆盖）；找不到则「在硬约束内把当前冲突推进一步」，仍非空。
5. **债务**：按 **连续续写次数**（Append 也计次）而非章数。阈值：冲突 2 次、角色 3 次、场景 3 次、伏笔 3 次。次数记在 `stories.asset_history_json` 旁新增的轻量计数（见 §7），不新表也可以：用 `agency_activity_log` 或 scene 更新次数。选定：**在 `scenes` 上不改 schema**；用现有 `stories.asset_history_json` 扩展字段 `{append_beats, last_conflict_beat, last_cast_refresh_beat, last_location_beat, last_foreshadow_beat}`。无历史视为 0（旧书不误伤）。

SceneBeatCard 注入为主创 prompt 的 **Critical**：全文开头一份完整卡，全文结尾一份四行摘要（双重锚定）。末句硬锚点仍在最后，但不得压过这四条——结尾顺序固定为：`BeatCard摘要 → 末句锚点`。

## 6. 上下文编译器（并入 TimeSliced 有用部分）

Agency 续写不再手写第二套 `build_writer_context_from_db` 长函数作为真源。真源改为：

1. 复用 `WriteTimeBundle::load_sync`（已有红线/大纲/世界观/伏笔/方法论/风格/账本）。
2. **补齐 TimeSliced 缺的**：角色 `emotional_core/trigger/wound/need`、`character_relationships`（Agency 今日已有，Bundle 没有 → 扩展 `CoreCharacter` 与 `to_prompt` 段）。
3. **ContextPrioritizer 分级排序本期降级**：Critical 由 BeatCard 头尾双锚承担，不在本实施把 Prioritizer 接到 Agency 热路径。
4. 前文：Append 用当前场景全文尾部（现有 `build_continuation_context` / 末句锚点）；NextChapter 用最近 1–2 场尾部。
5. 用户指令进 BeatCard 之上的「创作方向」段，冲突时以卡为准、保留指令核心意图（沿用 v0.30.32 调和句）。

`build_writer_context_from_db` 删除或改为对编译器的薄封装，禁止第三套字段清单。

## 7. 写后回流（合上 v0.34.0 空转）

每次续写正文落库后，同一事务或紧随的 `spawn_blocking` 必须更新：

| 列 | 来源 |
|---|---|
| `scenes.content` | Append 追加 / NextChapter 新写（已经做） |
| `scenes.characters_present` | 本拍 BeatCard.cast 的 **角色名**（与账本一致，禁止只写 UUID） |
| `scenes.character_conflicts` | 本拍 conflict_move 结构化 |
| `scenes.setting_location` | BeatCard 或正文抽取的地点；空则保留旧值 |
| `scenes.outline_content` | NextChapter：本章大纲；Append：把本拍 `next_outline_node` 追加为进度行，不覆盖用户手写大纲 |

已知遗留：库内既有行的 `characters_present` 可能混有角色 id 与名字；本期账本按名匹配，ingest/BeatCard 写入口径为名。旧 id 行需后续迁移，不在本实施清理。

ingest / asset_bridge 继续跑，但 **不得** 再把出场只写进大纲文本却不写 `characters_present`。`sync_scene_outline` 在写入文本的同时更新 JSON 列（仅当列为空或 source 为机器时，与现有 source 守卫一致）。

轮换账本 `last_seen` 按 **角色名** 匹配 `characters_present`（今日按 id 匹配空数组是第二刀）。债务 `last == 0` 不再视为「旧书零干扰」：若角色表非空且 `characters_present` 全空，视为角色债务已达阈值。

## 8. 从废路径并入 / 删除清单

### 8.1 并入三角色

| 来源 | 并入点 |
|---|---|
| WriteTimeBundle 装配 | §6 编译器 |
| 末句硬锚点、推进锚点 | 主创 prompt；推进改为下一节点句子 |
| ContextPrioritizer | §6（本期降级，BeatCard 双锚承担 Critical） |
| 轮换账本、扩张债务、资产菜单 | SceneBeatCard.expansion_quota；债务改按拍计 |
| beat_planner「这一拍」语义 | SceneBeatCard（Rust 先，不默认加 LLM） |
| `sanitize_novel_output`、trim 自重复/重叠 | 落库前 `cleanup_prose_for_persist`（已有，Append 必须走） |
| 续写目标字数 | 主创任务字数范围 |
| TriShot 质检下沉、8% 自重复闸门 | Editor 后台；写后闸门沿用 genesis |
| Agency 情感四元组与关系注入 | Bundle 扩展，两条路径变一条 |

### 8.2 删除（创世/续写范围）

- `AgentOrchestrator::execute_time_sliced` 与 `execute_trishot`：创世/续写零引用后删除（改写走 Full/Fast，不走这两函数）。
- `PlanExecutor` 续写 `auto → TimeSliced`、`tri_shot` 分支；`sanitize_plan_for_prose_request` 的 `beat_planner → writer` 续写重建。
- 设置项 `generation_mode` 的 `time_sliced` / `tri_shot` / `auto` 续写语义删除。UI 只保留改写相关（有选中文本时 Full/Fast）。`plan_mode` 的 beat/single_writer 对续写失效，可隐藏或标注「已废弃」。
- 幕前 `smart_execute` 续写再进 PlanExecutor writer 的路径。

Fast/Full **仅**保留给选中文本改写（`has_selected_text`）。无选中文本不得落入 Full。

## 9. 前端契约

幕前 `handleSmartGeneration` / `handleRequestGeneration`（文思活跃）：

- 分类为续写后，`smart_execute` 必须带 `scene_id`（当前场景）。后端 Append。
- 返回的 `final_content` 仍是 **本拍增量**（不是全章）。现有 `appendAiContent` 不变。
- 创世仍走现有 Agency 进度事件（`agency-agent-activity` / `agency-run-progress`）。续写同样发这套事件，幕前顶栏可显示「管理编译本拍 / 主创写作 / 编辑后台审查」。
- 文思活跃：上一次 `smart_execute` 未返回前禁止下一次；`smartExecuteInFlightRef` 已有，保持。

幕后「续写一章」：`agency_continue_chapter` 走 NextChapter；批量 `agency_continue_batch` 每章一次 NextChapter，串行。

## 10. 失败与降级

| 失败 | 行为 |
|---|---|
| 无 scene_id 的幕前续写 | UserAction 错误：「请先打开一个章节」 |
| 故事无角色 | 与今日 QuickPreflight 一致：可建占位主角一次，不阻断 |
| SceneBeatCard 某槽规则编译为空 | 该槽用降级句，卡整体仍注入 |
| 主创单次 empty / CoT 泄露 | `sanitize_novel_output`；空则一次散文回退；仍空则 Err，不写库 |
| 编辑审计超时/失败 | 已落库正文保留；toast 降级放行 |
| 并发 active run | 明确错误，前端保持 in-flight 禁用 |
| 自重复 ≥8% | 一次 anti-repeat 重试（genesis 闸门），再失败仍落库但记警告 |

熔断不等于丢稿：Append 在主创已产出 ≥200 字时必须尝试写回；&lt;200 字拒绝落库。同一门槛，不是两套规则。

## 11. 测试契约

每个测试保护一条用户可感知契约，实现细节可变。

1. **路由**：`is_continuation` 的 `smart_execute` 不调用 `execute_time_sliced` / `execute_trishot`（可用测试替身或模块级调用计数）。
2. **Append 不建章**：给定已有 scene，续写后 `scenes` 行数不变，`content` 为旧文+增量。
3. **NextChapter 建章**：幕后续写后 `sequence_number` 新行存在。
4. **BeatCard 阵容**：角色 ≥3 且有沉寂数据时，卡内人数 3–8 且含沉寂名。
5. **回流**：续写后该 scene 的 `characters_present` 含本拍 cast 名。
6. **债务按拍**：同一 scene 连续 2 次 Append 且冲突列仍空 → 第三次卡含冲突配额。
7. **情感注入**：角色有 `emotional_wound` 时主创 prompt 含该伤口（编译器单测）。
8. **创世回归**：现有 `run_genesis` 测试不回退。
9. **active run**：装配完成后可立即开始下一次 Append；编辑后台未结束不阻塞。

## 12. 实施顺序（本设计的阶段，不是任务清单）

阶段可独立交付，每阶段结束后幕前续写仍可运行。

1. **路由 + Append**：`smart_execute` 续写进 `run_continue(Append)`；先用今日 `build_continue_writer_context` + 单次主创 + 后台编辑，保证同章追加可用。
2. **编译器合一**：Bundle + 情感/关系 + Prioritizer 替换手写上下文。
3. **SceneBeatCard + 回流 + 按拍债务**。
4. **删除 TimeSliced/TriShot 续写引用与设置语义**；补齐测试。

阶段 1 不承诺四症状痊愈，只承诺唯一路径与同章追加。症状由阶段 3 关闭。

## 13. 验收

同一故事连续 ≥8 次幕前续写（含同章连续）：

| 指标 | 目标 |
|---|---|
| 单次增量具名角色数 | 中位 ≥3，且 ≥1 来自沉寂名单（角色总数 ≥3 时） |
| 可识别冲突动作 | ≥80% 续写含加压/反转/代价 |
| 情感节拍 | ≥80% 点名伤口/需求/关系张力 |
| 大纲推进 | 抽检不原地复述末句；`next_outline_node` 出现在 prompt Critical 段 |
| 章数 | 8 次幕前 Append 在自动分章未触发时不增加 `scenes` 行数；正文中应出现地点/阵容变化 |

证据：`creative_workflow.log` 的 prompt 含 BeatCard；DB 列 `characters_present` 非空。未跑通上述探针不得宣称四症状已修复。
