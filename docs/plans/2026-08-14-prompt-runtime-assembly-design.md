# Prompt Runtime Assembly：提示词当运行时组装

日期：2026-08-14
状态：已落地（v0.45.0）
决策来源：对照 DeepSeek Harness「Prompt as Runtime」的学习结论；用户裁定 **Approach C（先组装器）**，且 **P0 连创世主创内联一起收**（不推迟到 P3）。

承接：v0.41.0 Agency 唯一续写路径、v0.42.0 节拍卡资产准入、v0.44.0 墨纸/机械视觉（无关本设计）。本文件不改生成路由、PersistMode、`--ai-*`、落地页。

---

## 1. 问题与裁定

### 1.1 症状

StoryMoss 已有半套提示词运行时，但没有单一组装入口：

| 缝 | 现状 |
|---|---|
| Instruction | `RoleSpec` + `agency_*_system.md`；热路径主创却用 **内联 system 字符串**（创世 `writer_first_chapter` / `writer_prose_fallback`、续写 `write_beat_once`） |
| Context | 续写已有 `ContinueContextParts` + `SceneBeatCard` + `assemble_continue_user_prompt`；创世 user 是 `format!` 拼前提/概念/资产/写作要求 |
| Tools | `AgentTool::{name,description,args_schema}` + `catalog_for_role`；JSON action 协议硬编码在 `ToolLoop::run` 的 `head` |
| Variables | `TemplateEngine` 未知 `{{var}}` **fail-open**（保留原样） |
| Preview | `preview_prompt_composition` 默认仍报 TimeSliced / TriShot，与 Agency-only 续写/创世热路径不符 |

DeepSeek 的启发是：Capability = Implementation + Interface + Model-facing Instructions；Instruction ≠ Context ≠ Tools。要对齐的是 **一个组装函数**，不是抄 `/identity` `/persona` 目录树。

### 1.2 已确认裁定

| 项 | 决定 |
|---|---|
| 路径 | **C. 先组装器**。行为保持抽取 → 工具自带用法 → 内置模板 CI fail-closed |
| P0 范围 | 续写热路径 **加上** 创世主创内联（`writer_first_chapter` + `writer_prose_fallback`）。用户原话：「P0 连创世内联一起收」 |
| 创世内联含义 | **主创写正文的两条内联**，不是创世里每一次 LLM（`concept_pack` / `producer_depth_assets` 等仍留 P3） |
| 创世 system 资产 | P0 **不**改接到 `agency_lead_writer_system.md`。该文件仍讲 board_read / tool_loop，换过去会改模型行为 |
| 继承 | 不做 prompt inheritance，直到 Writer 与 LeadWriter 真共享约 80% |
| ContextPrioritizer | 本轮不接 Agency。节拍卡双锚点是 Critical；Prioritizer 只在 Lost-in-the-Middle 回归时作为 assembler 策略再议 |

### 1.3 与既有文档

| 文档 | 本设计如何对待 |
|---|---|
| Agency 多代理框架设计 | 黑板 / 三角色 / PersistMode 不动。组装器是提示词拼装，不是协调器 |
| v0.42.0 续写按拍选取 | `assemble_continue_user_prompt` / BeatCard / 准入 ≤8 仍是 Context 编译器。本设计在其 **之后** 接 Instruction |
| `WriteTimeBundle::to_prompt()` | **禁止改**。Agency 热路径不走它；改它会搅 TimeSliced 遗留与测试 |

---

## 2. 目标与非目标

**目标**

一个函数：`Instruction + Context + Tools → (system, user)`。创世主创、续写 `write_beat_once`、`ToolLoop` head 都走它。幕后提示词面板的场景预览与热路径一致。

**非目标**

- 不建 `/identity` `/persona` 目录树
- 不做提示词继承
- 不把 ContextPrioritizer 接到 Agency
- 不改 `WriteTimeBundle::to_prompt()`
- 不改 IPC 形状（不给 `PromptCompositionPreview` 加字段）
- 不改 PersistMode、`--ai-*` 变量名、落地页业务
- 不把 `producer_depth_assets` / `concept_pack` 等其余创世内联塞进 P0

---

## 3. 不变量（禁止破坏）

1. **`agency` 可以依赖 `prompts`；`prompts` 不得依赖 `agency`。** 组装器是哑拼接器：只吃 `Layer` 文本，不认识 BeatCard / Board / RoleSpec。
2. P0 对已接线路径 **字符串等价**：`assemble(...)` 产出的 `(system, user)` 与当前 `format!` 字节级相同（含空资产段、anti-repeat 后缀拼接方式）。
3. 续写 user 的编译顺序仍是：卡全文 → Bundle → 指令 → 卡摘要 → 末句锚点（`render_writer_user_prompt`）。组装器不重排 Context 内部。
4. `write_beat_once` 自重复 ≥8% 时，在 **已组装 system 后面**追加 ` 禁止重复同一段落或意象循环，不得首尾回环。`（现有行为）。
5. 两个 Tauri 窗口、PersistMode、`--ai-*` 17 名、色调同步：本设计零触及。

---

## 4. 模块切分

```
prompts/assembly.rs     Layer + assemble() → AssembledPrompt { system, user }
                        另：创世/续写/tool_loop 的 layer 工厂（常量正文，行为保持）
prompts/registry.rs     preview：agency_continue / agency_genesis 为主；
                        timesliced / trishot_call3 作别名
agency continue compile Context（已存在：beat_card / continue_assets / assemble_continue_user_prompt）
agency/roles            Instruction 来源（tool_loop 角色 system 仍走 RoleSpec）
agency/tools            Tools catalog；P1 起 usage_guidance
agency/tool_loop        head 走 assemble；不再手写 format!(catalog+protocol+task)
agency/coordinator      writer_first_chapter / writer_prose_fallback / write_beat_once
                        调工厂，不再内联拼 system/user
```

`assemble()` 规则：

- 重复 `id` → `Err`
- `required` 且 `body.trim()` 空 → `Err`
- 非 required 且空 → 跳过
- 同 `Slot` 层按输入顺序用 `\n\n` 拼接

---

## 5. 分层与 P0 接线

### 5.1 API（`prompts/assembly.rs`）

```rust
pub enum Slot { System, User }
pub enum LayerKind { Instruction, Context, Tools }

pub struct Layer {
    pub id: &'static str,
    pub kind: LayerKind,
    pub slot: Slot,
    pub body: String,
    pub required: bool,
}

pub struct AssembledPrompt {
    pub system: String,
    pub user: String,
}

pub fn assemble(layers: &[Layer]) -> Result<AssembledPrompt, AssembleError>;
```

### 5.2 P0 抽出的内联正文（禁止改字）

**创世 `writer_first_chapter`**

- system：`你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾、不得发明与资产冲突的角色或设定。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。`
- user 层：`故事前提：{premise}` / `概念设定：{concept_json}` / `创作资产：\n{assets}` / `写作要求：第一章正文，1500-2500 字，…`

**创世/续写熔断 `writer_prose_fallback`**

- system：较短版（无「不得发明与资产冲突…」句）
- user 层：无概念 JSON；`chapter_key` 仍只用于落库 key，不进 prompt（与现状一致）

**续写 `write_beat_once`**

- system：再短一截，末尾多「须在节拍任务硬约束内落实指令。」
- user：整段交给已有 `assemble_continue_user_prompt` → 单层 Context（不在 P0 拆 `render_writer_user_prompt`）

**`ToolLoop::run` head**（user 槽）

- Tools：`catalog_for_role(role)`
- Instruction：现有 JSON action 协议段落
- Context：`任务：\n{task}`

`ToolLoop` 的 `system_prompt` 参数仍由调用方传入（`RoleSpec` 解析的 md）。P0 只组装 **head**，不把角色 md 吞进 assembly。

### 5.3 预览诚实化

`preview_prompt_composition`：

| scene 入参 | 规范化 `scene` | 标签 |
|---|---|---|
| `agency_continue`、`timesliced`、默认 | `agency_continue` | Agency 续写 |
| `agency_genesis`、`trishot_call3`、`trishot`、`genesis` | `agency_genesis` | Agency 创世 |
| `pipeline_review`、`review` | `pipeline_review` | 审稿流水线（不动） |

层列表改为热路径真实层（instruction / beat_card / continue_assets 或 premise / concept / assets / task），**不再**列出 `writer_system` / `trishot_synthesizer` / `orchestrator_timesliced_writer` 假装还在跑。

前端 `PromptsPanel` 下拉：Agency 续写 / Agency 创世 / 审稿流水线；默认 `agency_continue`。不改 IPC struct。

---

## 6. 阶段

| 阶段 | 做什么 | 行为 |
|---|---|---|
| **P0** | `assembly.rs`；接线创世两条 + `write_beat_once` + ToolLoop head；preview 诚实化；golden 锁段标题与整串 | **字符串等价** |
| **P1** | `AgentTool::usage_guidance() -> Option<&'static str>` 默认 `None`。只给 `board_read`（资产已注入勿轮询全文）、`board_write`（draft 区，勿覆盖 user_created）、`asset_query`（勿倾倒全表） | catalog **会变长**（有意） |
| **P2** | 运行时 `TemplateEngine` 仍 fail-open。CI：bundled `agency_*` / `writer_*` / `scene_outline` 填完声明 `variables` 后不得剩 `{{ident}}`。`resolve_prompt_with_vars` 对剩余 `{{ident}}` `log::warn` | 运行时不硬失败 |
| **P3（本计划外）** | 其余创世内联；ContextPrioritizer 仅当 Lost-in-the-Middle 回归 | — |

---

## 7. GitNexus（实施前再跑一遍；索引可能落后）

| 符号 | 风险（本设计撰写时） | 含义 |
|---|---|---|
| `writer_first_chapter` | **LOW**（1 直接调用） | 创世快速路径主创 |
| `preview_prompt_composition`（registry） | **LOW**（2 直接） | IPC + 面板 |
| `ToolLoop.run` | **MEDIUM**（11 直接） | 改 head 前必须再 `impact`；只换拼接，不改循环控制 |

`assemble_continue_user_prompt` 可能不在旧索引里。改任何符号前必须 `impact({target, direction:"upstream"})`。HIGH/CRITICAL 先报告用户。

---

## 8. 验收

**P0**

- `assemble`：重复 id 失败；required 空失败；空可选层跳过
- 创世首章 / 散文回退 / `write_beat_once` 的 `(system, user)` 与抽取前 `format!` 金标相等
- anti-repeat 仍是 `system + " 禁止重复同一段落或意象循环，不得首尾回环。"`
- ToolLoop head 与抽取前 `format!` 相等
- preview `agency_continue` / `agency_genesis` 不含 TimeSliced/TriShot prompt_id；别名映射到规范 scene
- 面板默认请求 `agency_continue`
- `python3 scripts/architecture_guard.py`：`prompts` 禁止 `use crate::agency`
- `cargo test --lib` 只增不减；相关 vitest / tsc / format:check 绿

**P1**

- 无 `usage_guidance` 的工具 catalog 与 P0 金标相同
- 三个工具的 catalog 含「用法:」行

**P2**

- 声明变量填 `x` 后，目标 bundled 文件无残留 `{{ident}}`
- 故意留一个未声明 `{{ghost}}` 的夹具测试必须失败（该测试用内联字符串，不改真实 md）

**明确不宣称**

- 不宣称续写/创世质量变好（P0 字符串等价）
- 不宣称 v0.42.0 §8 真机探针已过
- 不宣称 `agency_lead_writer_system.md` 已用于热路径主创 complete()

---

## 9. 明确拒绝

| 拒绝 | 原因 |
|---|---|
| P0 把创世 system 换成 `agency_lead_writer_system.md` | 会改模型行为（tool_loop 口吻） |
| P0 收 `producer_depth_assets` / `concept_pack` | 「创世内联」= 主创写正文；其余另开 |
| 给 `PromptCompositionPreview` 加字段 | 避免 IPC manifest |
| 改 `to_prompt()` | 非热路径，爆破面大 |
| 组装器认识 BeatCard | 违反 prompts↛agency |
| 运行时未知变量改 fail-closed | 用户覆盖提示词会直接打挂生成；只 CI 内置文件 |
