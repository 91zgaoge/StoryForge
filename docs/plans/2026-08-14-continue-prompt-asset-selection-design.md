# 续写提示词：按拍选取创作资产

日期：2026-08-14
状态：已发版 v0.42.0（真机 §8 未跑）
决策来源：v0.41.1 幕前续写第 26 章诊断（提示词约 24559 字符 / 估算 12279 tokens，前端 600s 超时）；用户裁定「非本拍必要的资产不进提示词」；未上场角色采用 **A. 一行名单**（禁止新编姓名，不给完整人设）。

承接：`docs/plans/2026-08-13-agency-only-continuation-design.md` §6.3（ContextPrioritizer 本期降级）与已知债务「注入 ≠ 使用」。

## 1. 问题

### 1.1 症状

长篇续写提示词常到 1–2 万字。模型把 token 耗在重复大纲和无关角色卡上；小窗口本地模型直接 400；推理模型空正文后再 fallback，直到前端超时。

### 1.2 根因（代码核对）

`write_beat_once` 当前顺序：

1. `build_writer_context_from_db` → `WriteTimeBundle::load_sync` + `to_prompt()`：**全表角色、全表关系、整份 `story_outlines.content`、世界观全文、近 3 场前文**。
2. `generate_chapter_outline` 再把**全部角色名**灌进大纲 LLM。
3. `compile_beat_card` 已经按正文选了 ≤8 人的本拍阵容。
4. `render_writer_user_prompt` 把头尾节拍卡和 **未筛选的 Bundle 全文**拼在一起。

节拍卡是「这一拍用谁」；Bundle 仍是「这部书有什么」。两者同时出现，任务卡被淹没。

另外：`memory/asset_bridge.rs` 的 `sync_story_delta` 对 `story_outlines.content` 只追加不去重收口，同一「核心冲突 / 转折点」会堆成超长 blob。跨故事脏角色若已写入本故事 `characters` 表，全量倾倒会一并送进模型（诊断中的周奕辰 / 陆南星）。

### 1.3 裁定（已确认）

| 项 | 决定 |
|---|---|
| 选取方式 | **确定性**，0 次额外 LLM。不用 `asset_retrieval_plan`（失败即全量，热路径已证明会撑爆）。 |
| 准入真源 | `SceneBeatCard` 阵容 + 本拍文本线索（见 §3）。 |
| 未录取角色 | **A. 一行名单**：`【本拍未上场（禁止新编下列姓名）】甲、乙、丙`。不给人设/情感四元组。 |
| 脏角色 | 从未出现在近场正文、`characters_present`、故事大纲中的名字，**名单也不列**。 |
| 范围 | Agency 续写热路径（`write_beat_once` / `write_chapter` 共用的上下文编译）。创世首章、划词改写不动。 |
| ContextPrioritizer | 本设计不做排序接入；选取先落地。排序仍记债务。 |

## 2. 目标与非目标

**目标**：主创 `complete()` 的用户提示词只含本拍用得上的完整资产；未上场已知角色以名单约束命名；提示词体量随「这一拍」缩放，不随角色表/大纲累积线性膨胀。

**非目标**：

- 不新增检索 Agent / 不在续写热路径加 LLM 选 key。
- 不改 `WriteTimeBundle.to_prompt()` 的全局语义（改写 Full 路径仍可全量）。
- 不清理历史 `characters` 脏行（只在注入层忽略）；不改 ingest 写库策略以外的大纲追加（注入层去重；追加收口另登记债务）。
- 不把 ContextPrioritizer 接到 Agency。
- 不宣称设计 §13 八次真机探针已过。

## 3. 准入名单

`AdmittedCast`：本拍给完整卡的角色名集合，上限 **8**（与现 BeatCard `truncate(8)` 对齐）。

并入来源（并集，去重，再截断）：

1. `compile_beat_card` 已有规则：正文末段点名 ∩ 角色表、沉寂回归 1 人、最高张力对手/盟友、不足 3 人时补位。
2. 当前场景 `characters_present`（按名；id 混杂行忽略无法匹配的 token）。
3. 用户指令里出现的角色名。
4. `next_outline_node` 与**已生成的本章大纲**里出现的角色名。
5. 冲突双方 `conflict_move.parties`。

截断时保留顺序：正文在场 > 冲突双方 > 大纲/指令点名 > 沉寂回归 > 补位。

**名单（roster）**：角色表中有名、未进 `AdmittedCast`、且满足「本故事相关」：

- 近 5 场 `scenes.content` 或 `characters_present` 出现过；或
- `story_outlines.content` 出现过。

否则视为脏数据，提示词完全不出现该名。

## 4. 各资产注入规则

在 Agency 层对 Bundle **筛选后再渲染**，不改 `to_prompt()` 全量契约。建议新纯函数（可单测、0 I/O）：

`render_continue_assets(bundle, admitted, roster, location, next_node) -> String`

| 段 | 规则 |
|---|---|
| 世界观红线 | 必进。Critical。截断 800 字（现有 `extract_redline_text` 口径可保留）。 |
| 故事大纲 | **禁止**原文倾倒。去重后只留：一条核心冲突 + 与 `next_node` 重叠的至多 3 条转折点 + `next_node` 本身。硬上限 1200 字。重复的「【核心冲突】/【转折点】」只保留首次出现。 |
| 世界观设定 | 概念截断 400 字；规则最多 5 条，优先名称/描述含本拍地点或录取角色名的；历史/文化默认不进。硬上限 800 字。 |
| 登场角色 | **仅** `AdmittedCast` 的完整卡（身份/状态/性格/情感四元组）。标题改为「本拍角色（须遵循当前状态）」。 |
| 未上场名单 | roster 非空时一行，逗号分隔，前缀固定：「本拍未上场（禁止新编下列姓名，亦不得当主角使用）」。上限 40 名，超出在末尾加「等」。 |
| 情感关系 | 只保留两端都在 `AdmittedCast` 的行。一端在名单、一端在录取：不进完整关系行（避免把未上场者写成人设）。 |
| 本章/场景大纲 | 当前场 `outline_content` 或本拍刚生成的章节大纲，截断 800 字。 |
| 已推进进度 | 近 3 场 outline 各 200 字，保持现状。 |
| 前文 | Append：当前章末 **800** 字（现 1500）+ 末句锚点。NextChapter：最近 1 场末 800 字。不再默认叠 3 场 × 1500–2000。 |
| 张力/弧光 | 只渲染录取角色参与的条目；各硬上限 400 字。 |
| Logline | 保留，一条。 |
| 伏笔 | 待回收 top 3 + 逾期 top 1（Bundle 已有上限），保持。 |
| 风格/方法论/题材表/few-shot | 本拍不进（Background）。需要时由改写路径走全量 Bundle。 |

**硬预算**：`render_continue_assets` 输出 ≤ **6000** 字。超则按上表从下往上截：前文 → 进度 → 世界观概念 → 大纲转折点。红线、录取角色卡、未上场名单、节拍卡本身不得因预算删光；角色卡过多时按准入顺序丢掉末位完整卡，把该名移入名单。

节拍卡头尾双锚与末句锚点仍由 `render_writer_user_prompt` 负责，**不计**入 6000（它们是任务不是资料）。

## 5. 编排顺序

`write_beat_once` / `write_chapter` 改为：

```
compile_beat_card(current_content)           // 准入 v1
generate_chapter_outline(filtered chars)     // 大纲 LLM 只看见录取卡 + 名单，不再全表
admitted = v1 ∪ names_in(outline ∪ instruction)
render_continue_assets(bundle, admitted, roster, …)
render_writer_user_prompt(assets, card, instruction, current_content)
```

`generate_chapter_outline` 的 `characters` 变量改为：录取角色一行人设摘要 + 未上场名单，禁止再 `get_by_story` 全表拼接。

`build_writer_context_from_db` 改为薄封装：load bundle → 调用 `render_continue_assets`。无 beat 时（测试/缺卡）录取 = 角色表前 8 人（现 `chars.first()` 主角偏好），避免测试全空。

## 6. 失败与降级

- Bundle 加载失败：与今日相同，空串 + warn；节拍卡仍在。
- 角色表为空：无完整卡、无名单；不发明角色的约束靠大纲/红线。
- 录取 0 人但角色表非空：录取主角 1 人（BeatCard 已有空卡补主角）。
- 大纲去重后为空：注入 `next_outline_node` 一句，不强行灌 blob。
- 预算截断：打 `log::info`（录取人数、名单人数、各段字数、是否截断），便于对照 `creative_workflow.log`。

热路径禁止为「选资产」再调 LLM。

## 7. 测试契约

纯函数优先（不启 LLM）：

1. **20 个角色，正文只点 3 个**：完整卡恰好这 3 个（±沉寂/张力补位）；其余相关者在一行名单；点名「必须严格遵循」的全员列表不得出现。
2. **脏角色**：表内有「周奕辰」，近场与大纲均未出现 → 完整卡和名单都不含该名。
3. **关系**：录取甲乙、丙在名单 → 只有甲–乙关系行。
4. **大纲堆叠**：10 次相同「【核心冲突】皇权裂痕」→ 输出只出现一次；总长 ≤ 1200。
5. **预算**：人造超长前文时，输出 ≤ 6000，且红线 + 录取卡 + 名单仍在。
6. **流程**：`write_beat_once` 在 mock 下，发给主创的 user prompt 不含未录取者的「情感内核」字段。

`test_build_continue_writer_context` 改为断言筛选后的段，而不是「角色表有谁 prompt 就有谁」。

## 8. 验收

用同一诊断故事（或同等长篇）再续写一拍，从 `creative_workflow.log` 取主创 prompt：

| 指标 | 目标 |
|---|---|
| 完整角色卡数量 | ≤ 8，且 ⊆ 本拍正文/大纲/指令点名 ∪ 张力/沉寂补位 |
| 未上场名单 | 一行；不含完整人设；不含近场从未出现的脏名 |
| 故事大纲段 | 无连续重复的「【核心冲突】」块；≤ 1200 字 |
| 用户提示词 | 显著短于诊断的 24559 字（目标 < 8000 字含节拍卡与大纲，不含系统工具清单） |

未达上表不得宣称「上下文已智能选取」。八次真机质量探针仍属 §13 债务。

## 9. 已知债务（本设计不做）

- ingest 继续追加 `story_outlines`；注入层去重不能替代表内收口。
- `characters` 跨故事污染行不删除。
- 本地模型连接超时 60s×2（v0.41.2 已登记）。
- ContextPrioritizer 未接 Agency。
