# OpenViking × StoryMoss 长篇记忆对照与引进方案

**状态**：已批准方案 B 并实施 v0.52.0（P0 轨迹 + P1 分层卡 + P2 任务点名准入 + 近文 1500 + 大纲封顶）。P3 freshness 节流 ingest、P4 ContextPrioritizer 未做。  
**实施日期**：2026-08-21  
**对照版本**：StoryMoss v0.51.6；OpenViking（volcengine/OpenViking，docs.openviking.ai，VikingMem arXiv:2605.29640）  
**证据标签**：StoryMoss 记忆链路 = **inspected**（源码 + ARCHITECTURE）；OpenViking = **inspected**（官方文档/README，未跑其服务）；融合收益 = **assumed**（未做对照实验）。

---

## 0. 怎么读这份对照

先读 §同 / §异 / §同一任务走两边。看懂「像在解决同一类问题、但不是同一类产品」之后，再看 §5 三条路径。本节不替你选。

---

## 0.1 同：两边都在对抗同一类失败

长上下文一旦「全塞进提示词」，模型会丢中间、编造、复述、把旧设定当当前状态。两边都拒绝扁平聊天记录堆叠，都把记忆拆成**可寻址的对象**，都有**写后抽取**和**读时裁剪**。

| 共同问题 | 两边各自的应对（形同） |
|---|---|
| 全书/全历史塞不进窗口 | 草苔：拍级预算 6000 + 前文双窗。Viking：L0/L1/L2 按需加深 |
| 扁平切片检索丢掉结构 | 草苔：角色表/大纲/场/拍是显式结构。Viking：目录树 + URI，先找目录再找文件 |
| 写完要沉淀，不能只留对话 | 草苔：Ingest 两步 → KG + 资产桥。Viking：session.commit → schema 抽取 |
| 机器摘要会和用户手改打架 | 草苔：`user_created` 永不覆盖。Viking：Resource 用户添加、相对静态；Memory 才是 Agent 可改 |
| 需要知道「这次带了什么」 | 草苔：有 prompt/调用日志，缺「为何选中这张卡」。Viking：retrieval trajectory |
| 父级摘要会过期 | 草苔：大纲无界追加（弱点）。Viking：freshness / pending_child_changes（仍承认 bubble 过勤） |

所以：**同的是「上下文工程」**——对象化、分层、事后抽取、读时裁剪。不是「都是小说记忆系统」。

---

## 0.2 异：产品对象、真相、取数钥匙 三条分水岭

**1. 记的是谁的世界**

- 草苔记的是**书里的世界**：谁在场、欠什么债、红线、这一拍必须发生什么。用户是作者。
- OpenViking 记的是**Agent 的世界**：用户偏好、文档资料、可调用技能、任务轨迹。用户是操作者。

同一词「实体」：草苔 = 角色/地点（小说人物）；Viking Memory `entities/` = 用户认识的人/项目/组织（真人通讯录）。不能对着映射。

**2. 什么东西不许被摘要替换**

- 草苔：`scenes.content` 是唯一叙事真相。L0/L1 若存在，也只能是索引，写回正文即架构违约。
- Viking：Resource / Memory / Skill 在 URI 上等价。L2 是「该节点的全文」，没有「小说正文高于角色卡」这一条法律。Agent 读到的 L1 摘要可以当工作记忆用。

**3. 取数的钥匙是什么**

- 草苔热路径的钥匙是**镜头**：近 500 字里点了谁的名 → 硬准入。大纲下一节点、未还债务是编译进去的，不是向量搜出来的。0 LLM。
- Viking 热路径的钥匙是**本轮意图字符串**：「帮我写 RFC」→ IntentAnalyzer 改写成 0–5 条 TypedQuery → 向量定位目录 → 递归 → rerank。常常再花一次 LLM。

这决定了：把一本小说丢进 Viking，续写查询是用户说的「续写」，不一定是章末 500 字；草苔相反，用户只说「续写」，真正的 query 是正文尾巴。

---

## 0.3 概念对照（形似 ≠ 同一件东西）

| 草苔 | OpenViking | 关系 |
|---|---|---|
| `scenes.content` | 某个 Resource 文件的 L2 | 都是「全文」；只有草苔把它定为法律 |
| 角色/世界观/大纲表 | Resource 目录，或 Memory `entities/` | 都是结构化长期对象；草苔面向写作编辑，Viking 面向 Agent 浏览 |
| 节拍卡（BeatCard） | 当前工作目录 + 这一次 find 结果 | 都是「这一轮允许看见什么」；草苔是编译器产出，Viking 是检索器产出 |
| Ingest 两步思维链 | Parser + SemanticQueue，或 session 抽取 | 都是写后沉淀；草苔字段是小说（四元组/关系/伏笔），Viking 字段是用户/任务 |
| `asset_bridge` 源感知合并 | Memory 的 LLM merge/delete/skip | 都处理新旧冲突；草苔禁止删用户行，Viking 允许模型删记忆 |
| MemoryPack Working / Episodic / Semantic | 会话近轮 + 归档 L0/L1 + 长期 Memory | 认知分层同构；草苔这套**未接续写热路径** |
| `continue_assets` 字数硬切 | 先 L0 再决定是否 read L2 | 都是预算；草苔切的是已编译文本，Viking 切的是加载深度 |
| 提示词 / PlanExecutor 技能 | `viking://…/skills/SKILL.md` | 都是「怎么干活」；草苔技能不进记忆库 |
| `creative_workflow.log` | retrieval trajectory | 都要可观测；粒度不同 |
| 合同红线 MASTER_SETTING | Resource 里的规则文档，或 soul.md 原则 | 都是硬约束；草苔强制双重锚定设计（尚未接到 Agency） |

---

## 0.4 同一件事走两边：点「续写」

假设第 6 章已有几千字，用户点续写，不打指令。

**草苔实际路径（inspected）**

1. 钥匙 = 本章近文，不是「续写」两个字。
2. 近 500 字点名 → 本拍阵容硬名单。
3. 0 LLM 编译节拍卡：下一节点、冲突、债务、最多 2 条未解决审查。
4. 准入者完整卡 ≤8，其余一行「禁止新编这些名字」，前文 600+1800，总预算 6000。
5. 主创写增量，落 `scenes.content`。
6. 稍后 Ingest 分析正文，回流机器来源的角色/大纲；手改字段不动。

缺什么：未点名但大纲写了必须还的债，可能进不了完整卡；大纲文本无界变长，只能硬切；日志说不清「为什么是这 8 人」。

**若把同一本书当 Viking Resource 树（assumed：按其文档推理，未跑服务）**

1. 钥匙 = 用户话语「续写」+ 近几轮 session。
2. IntentAnalyzer（LLM）改写成若干 TypedQuery（资源/记忆/技能）。
3. 向量先命中「像续写」的目录（可能是大纲、也可能是某张旧角色卡、甚至某份技能）。
4. 递归子目录，返回一批 L0；Agent 再 `read` 若干 L2。
5. 没有「近 500 字点名」这条物理门闩。章末尸体是否还在场，取决于检索有没有把那一段当 L2 读进来。
6. commit 后抽取的是「用户偏好 / 实体 / 事件」，默认不是「这一场谁必须留下」。

缺什么：小说不变量（正文真相、在场窗口、用户卡优先）不是一等公民；多一次意图 LLM；桌面应用要常驻 Python 服务。

**同**：两边都不会把 20 万字正文整段塞进主创。  
**异**：草苔用镜头编译这一拍；Viking 用意图检索这一轮。

---

## 1. 本项目现在怎么记一本长篇

### 1.1 三层真相，不是一层向量

| 层 | 落点 | 谁写 | 谁读 | 不变量 |
|---|---|---|---|---|
| **L-叙事** | `scenes.content` | 用户 + Agency 主创落库 | 一切下游 | 唯一叙事真相。禁止再写 `chapters.content`。有正文时禁止按书名发明角色/大纲（v0.49.0） |
| **L-生产** | `characters` / `character_relationships` / `world_buildings` / `story_outlines` / `scenes.outline_content` / 伏笔 | 用户手工；Agency materialize；Ingest 经 `asset_bridge` | 节拍卡、幕后续写、工作室资产栏 | `user_created` / `manual` 永不被机器覆盖 |
| **L-索引** | `kg_entities` / `kg_relations` / `memory_items` / VIEW `story_memory_facts` | Ingest 两步思维链 | MemoryFacade top-5、幕后记忆页、**未接 Agency 热路径** | 失败软降级；物理表不 DROP |

旁路还有合同（`MASTER_SETTING` 红线）、Agency 黑板（Asset / Draft / Review）、观察 run（停手 30s 编译当场大纲与下一拍，不改正文）。

### 1.2 写入：两步 Ingest + 单向桥

`memory/ingest.rs` 按 llm_wiki：**分析 → 生成结构化知识**（角色情感四元组、双向关系、世界观增量、场景大纲、故事增量）。落 KG 之后，`memory/asset_bridge.rs` **单向** upsert 进生产表。源感知合并：只精炼 `ingest` / `agency` / `auto_placeholder`。同 story 进程内锁防重复角色行。

已知弱点（已登记，不是本方案要假装修掉的）：

- `story_outlines` 无 `source` 列，机器提取会无界追加。
- Ingest token 不计 `AgencyBudget`；取消不传播给已 spawn 的 ingest。
- 后台 ingest 与前台主创曾抢同一本地模型（v0.51.6 只修了角色映射被创作模型置顶，硬件串行仍在）。

### 1.3 读出：拍级编译，不是全书检索

Agency 续写热路径（v0.41+ / v0.42）**不**调用 `WriteTimeBundle::to_prompt()` 全表倾倒。`agency/continue_assets.rs`（0 LLM、0 I/O）按 SceneBeatCard：

- 完整卡 ≤8 人；未上场一行名单；脏名不进。
- 大纲去重、世界观截取、资料预算 6000 字。
- 长章前文双窗：开篇 600 + 近文 1800；本拍阵容只看近 500 字镜头。
- 红线 / 下一节点 / 未解决 revise（最多 2 条）进节拍卡。

`WriteTimeBundle` 仍服务改写 Full 路径：红线 → 故事大纲 → 世界观 → 核心角色 → 场景大纲。

另有一套**更完整但热路径基本闲置**的记忆栈：

- `MemoryOrchestrator`：Working / Episodic / Semantic + 任务类型预算。
- `QueryPipeline`：CJK 二元组 → 图谱扩展 → token 预算 → 带引用组装。
- `HybridSearch`：BM25 + 向量 RRF。LanceDB 持久化 blocked，SQLite 向量兜底。
- `ContextPrioritizer`：Critical / High / Normal / Background + 双重锚定。**未接到 Agency 热路径**（登记债务）。

### 1.4 结构方法一句话

**场景优先的合同+资产编译记忆**：正文为真相，资产为法律，KG 为索引；热路径用确定性节拍卡做「这一拍准入」，冷路径用 Ingest 回流。不是「把小说切块进向量库再 top-k」。

这与 OpenViking 的差异，本质是 **domain compiler（小说拍） vs context database（通用 Agent）**。

---

## 2. OpenViking 怎么记上下文

来源：官方 README、docs.openviking.ai architecture / context-layers / retrieval。未 clone、未跑服务。

### 2.1 定位

「Self-evolving Context Database for AI Agents」。统一 Agent Memory、Knowledge RAG、Skills。配套论文 VikingMem（VLDB 2026）。

针对的五个痛点：上下文碎片、长任务体积膨胀、扁平 RAG 差、检索不可观测、记忆只能堆聊天记录。

### 2.2 结构

- **一种 URI**：`viking://resources|user|agent/...`，Agent 用 `ls` / `tree` / `find` 浏览，而不是只对扁平向量 query。
- **三类上下文**：Resource（文档）、Memory（用户偏好/实体/经历）、Skill（可执行技能）。
- **三层信息（目录 sidecar，不是每个文件一份）**：
  - L0 `.abstract.md`：约 100 token / 默认 256 字，向量召回。
  - L1 `.overview.md`：约 2k–4k 字，rerank 与导览。
  - L2 原文：按需 `read`。
- **写入**：Parser 建树（无 LLM）→ 异步自底向上生成 L0/L1 → 向量索引只存 URI+向量，正文在 AGFS。
- **读取**：IntentAnalyzer（0–5 条 TypedQuery）→ 目录递归检索（优先级队列 + 分数传播）→ rerank。
- **会话提交**：压缩旧轮 → 按 schema 抽 6 类记忆（profile / preferences / entities / events / cases / patterns）→ LLM 去重（merge/delete/skip）。
- **可观测**：每次检索留 trajectory。
- **新鲜度**：sidecar `freshness`（sampled / pending_child_changes）；文档承认当前每次成功都会向上 bubble，计划按 freshness 合并刷新。

部署默认是独立 Python HTTP（`:1933`），不是嵌入式库。

### 2.3 它不是什么

不是小说引擎。没有「场景 / 节拍 / 合同红线 / 正文不得被摘要覆盖」。L2 可以是任意文档；StoryMoss 的 L2 必须是 `scenes.content`。

---

## 3. 差异对照

| 维度 | StoryMoss | OpenViking | 对长篇写作的含义 |
|---|---|---|---|
| 产品对象 | 单机小说桌面应用 | 通用 Agent 上下文库 | 嵌对方运行时 = 错对象 |
| 真相源 | `scenes.content` | 文件系统节点（资源/记忆/技能等价） | 绝不能让摘要层盖过正文 |
| 组织单位 | 故事 → 章/场 → 拍（BeatCard） | 目录树 + URI | 拍 ≈ 他们的「当前工作目录」 |
| 热路径 | 确定性编译 + 字数预算 | 向量目录检索 + 按需加深 | 我们已有准入名单；缺「未准入资产的 L0 可检索」 |
| 冷路径 | 两步 Ingest + 资产桥 | Parser + SemanticQueue + session extract | 我们更懂小说字段；他们更懂异步摘要与去重策略 |
| 分层 | MemoryPack Working/Episodic/Semantic（闲置）；BeatCard 截断 | L0/L1/L2 写时生成、读时加深 | 分层思想可移植；实现应对齐拍/资产，不要对齐聊天 session |
| 冲突处理 | 用户编辑优先；冲突进 MemoryWarning | LLM 决定 merge/delete/skip | **禁止**让 LLM 删用户资产 |
| 可观测 | `creative_workflow.log` 记 prompt/调用，**不记「为何选中这张卡」** | retrieval trajectory | 引进价值高 |
| 膨胀 | `story_outlines` 无界追加；6000 字硬切 | 父目录 L0/L1 聚合子摘要 | 引进价值高 |
| 依赖 | Rust + SQLite；LanceDB blocked | Python server + AGFS + 向量 | 禁止引入 Python sidecar |
| Agency 接线 | 热路径不走 QueryPipeline / ContextPrioritizer | 检索是一等公民 | 分层加载应接 `continue_assets`，不是另起一套 RAG |

---

## 4. 值得引进 / 不值得引进

### 4.1 值得（按杠杆排序）

1. **资产 L0/L1/L2（SQLite 列或 sidecar 表，不是 viking 文件）**  
   每个角色/世界观/大纲节点：L0 一行（向量/名单用）、L1 半卡（rerank/节拍候选）、L2 全卡（仅准入者）。直接打 `story_outlines` 膨胀和「冲突行里塞 UUID」这类 Lost-in-the-Middle。

2. **拍 = 工作目录的分层检索**  
   先 L0 在「本故事资产树」里定位（本拍阵容、地点、未兑现债务），再只对命中节点加载 L1/L2。保留现有确定性准入（镜头 500 字点名）为硬门闩，检索只补「门闩没点到但 L0 高相关」的债务/伏笔/旧冲突。

3. **检索轨迹写入诊断日志**  
   每条续写记录：准入名单、L0 命中、被预算裁掉的 URI、最终注入层。对照「唱反调」时可证明是编译漏了还是模型无视。

4. **freshness / pending_child_changes**  
   Ingest 成功不立刻重写整本故事大纲 L1；标记父节点 stale，空闲窗口再聚合。减轻「回流超时重试抢 GPU」。

5. **Schema 驱动抽取 + 显式去重决策（仅机器行）**  
   OpenViking 的 merge/skip 可映射到 `REFINABLE_SOURCES`；`user_created` 仍禁覆盖。有助于大纲去重，替代无界追加。

### 4.2 明确不引进

| 项 | 原因 |
|---|---|
| OpenViking Server / Python SDK / `viking://` | 进程模型、许可证需另审、与 architecture_guard 和本地 LLM 调度冲突 |
| Skill 文件系统 | 本项目技能已在提示词注册表 / PlanExecutor |
| 会话 6 类用户画像记忆 | 对象是作者偏好不是角色弧 |
| 用 LLM 删除冲突记忆 | 违反用户编辑优先 |
| 热路径 IntentAnalyzer（额外 LLM） | 续写已有 600s / 本地模型争用；检索必须 0 LLM |
| 替换 `scenes.content` 为摘要 | 违反场景优先不变量 |
| 把 Agency 改回 `WriteTimeBundle::to_prompt()` 全倾倒 | v0.42 刚切断 |

### 4.3 与已有债务的关系

本项目研究前沿已是 Context Rot 量化（`sf-research-frontier`）。OpenViking 的 L0/L1 是**工程对抗**，不是测量。引进后仍须：探针「红线/在场/地点是否被违反」，否则不得宣称续写质量已修复。

---

## 5. 三条融合路径

### 方案 A — 嵌 OpenViking sidecar

把小说资产同步进 `viking://resources/story/{id}/...`，续写 `find()`。

- 优点：少造轮子、自带 trajectory。
- 缺点：Python 常驻、双真相源、Ingest 再写一份、许可证与打包、热路径延迟、本地模型再加一层。
- **否决。**

### 方案 B — 思想移植（推荐）

在 SQLite 为生产资产增加 L0/L1 列（或 `asset_layers` 表），`continue_assets` 改为「硬门闩 + L0 补漏 + 按层加载」。轨迹打 `creative_workflow.log`。Ingest 写 L2 事实后异步刷新 L0/L1，受 freshness 节流。不引入 OpenViking 代码。

- 优点：守住场景优先、用户编辑优先、0 LLM 热路径、无新运行时。
- 缺点：要自己写聚合与轨迹；第一期不做向量目录递归（LanceDB 仍 blocked）。
- **推荐。**

### 方案 C — 只观测，不改编译

只加 retrieval-style 日志，解释当前 BeatCard 为什么是这 8 人。

- 优点：风险最低。
- 缺点：不解决大纲膨胀和中间段落腐烂。
- 可作为 B 的 P0，但不能当完整引进。

---

## 6. 方案 B 实施方案（批准后才执行）

不变量（全程）：

- `scenes.content` 仍是唯一叙事真相；L0/L1 不得写回正文。
- `user_created` / `manual` 字段机器只填空。
- Agency 热路径 0 LLM 检索；L0/L1 由冷路径生成。
- 不改 `WriteTimeBundle::to_prompt()` 全局语义。
- 不引入 Python / OpenViking crate。
- 不宣称续写质量/唱反调已修复，直到设计验收探针（含真机）跑过。
- 改符号前 GitNexus `impact`；提交前 `detect_changes`。

### P0 — 观测（约 1 个小版本）

- 在 `render_continue_assets` / `assemble_continue_beat` 出口打结构化日志：`admitted[]`、`roster[]`、`outline_chars`、`world_chars`、`prior_window`、裁掉的角色名。
- 契约测试：日志字段存在且不含冲突 UUID 亦可（若本拍已有 `compile_conflict` 纯名）。
- **验收**：同一故事连续续写，诊断里能回答「本拍为什么是这 8 人」。

### P1 — 资产分层列（约 1–2 个小版本）

- 迁移：`characters.l0_abstract` / `l1_overview`（或独立 `story_asset_layers(story_id, kind, asset_id, layer, body, generated_at, pending)`）。世界观、大纲节点、伏笔同构。
- L2 = 现有全卡字段。L0 规则生成优先（姓名+身份+一句话状态，0 LLM）；L1 在 Ingest/观察空闲时由后台模型刷新，失败保留旧 L0。
- `continue_assets`：未上场名单改用 L0；准入者注入 L1，仅当前冲突双方 / 地点实体升 L2。预算仍 6000，但是**层预算**不是硬切全文。
- **验收**：`continue_assets` 单测：8 人准入时未准入者不上 L2；大纲 L1 超长时 L0 仍在；用户手改角色卡后 L1 标记 pending 且不覆盖手工字段。

### P2 — 硬门闩 + L0 补漏（约 1 个小版本）

- 镜头 500 字点名仍是硬准入。
- 额外：对 `open_loop` / 逾期伏笔 / 本场 `outline_content` 下一拍点名的实体做 **L0 字符串/BM25 匹配**（不用新 LLM、暂不用 LanceDB）。命中且不在脏名列表 → 升入准入，仍 ≤8（冲突/下一拍优先于远景角色）。
- **验收**：节拍卡写了「必须还债 X」而近 500 字没点名时，X 仍能进完整卡；无关旧角色不进。

### P3 — freshness 节流 Ingest 上卷（约 1 个小版本）

- 父级 `story_outlines` L1 不在每次 ingest 追加正文；只 `pending_child_changes++`。
- 与 v0.51.0 观察同一 30s 空闲窗：无前台 creative run 时再聚合 L1。
- Ingest 仍不得在 `agency_writer` 进行中抢同一本地 endpoint（延续 v0.51.6 角色映射；P3 加「前台 creative 期间不 spawn ingest」若仍缺）。
- **验收**：取消续写后不自动重跑超时 ingest；故事大纲表不再每次 +一段规划散文。

### P4 — ContextPrioritizer 接 Agency（有测量才开默认）

- Critical = 红线 + 本拍 L1 冲突/地点；High = 准入 L2；Normal = 未准入 L0 名单；Background = 旧章 L0。
- 先实验 flag，用 `sf-research-frontier` 的违反率探针，未达 M1 不写进 ARCHITECTURE 既成事实。

### 明确不做（本方案范围外）

- 总监第四模型。
- 目录递归向量检索（等 LanceDB 解阻再评估）。
- 自动 DELETE 脏角色行。
- 把 QueryPipeline 接到续写热路径。

### 验证命令（每期）

```
cd src-tauri && cargo test --lib
cd src-frontend && npx tsc --noEmit && npx vitest run
python3 scripts/architecture_guard.py
```

真机：同一开头连续 ≥3 次续写，对照轨迹日志看阵容/债务，**不得**仅凭单测宣称质量修复。

### 文档与发版

每期仍按 AGENTS.md：README / CHANGELOG / AGENTS / PROJECT_STATUS / ROADMAP / ARCHITECTURE / TESTING / USER_GUIDE + `FALLBACK_VERSION`。仅在用户批准实施后 bump。

---

## 7. 批准清单

请明确回复下面三项（缺一不可实施）：

1. 路径：B / 仅 P0（方案 C）/ 否决引进。
2. 范围：P0 only / P0–P1 / P0–P3 / 含 P4 实验 flag。
3. 确认：**不**嵌入 OpenViking 进程；**不**用本方案对外宣称续写质量已修好。

批准后下一步：按 `writing-plans` 把选定范围拆成可执行任务，再改代码。
