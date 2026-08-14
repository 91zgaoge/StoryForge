# Prompt Runtime Assembly Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 按 `docs/plans/2026-08-14-prompt-runtime-assembly-design.md` 落地「提示词当运行时」：P0 哑组装器 + 创世主创内联 + 续写 `write_beat_once` + ToolLoop head 字符串等价接线，并让场景预览与 Agency 热路径一致；P1 三个工具自带用法；P2 内置模板 CI fail-closed。

**Architecture:** `prompts/assembly.rs` 只拼接 `Layer` 文本，不依赖 `agency`。Context 仍由 `assemble_continue_user_prompt` / BeatCard 编译。创世与续写热路径的 system/user 经工厂函数走 `assemble()`。`WriteTimeBundle::to_prompt()` 不动。

**Tech Stack:** Rust（`cargo test --lib`）、既有 vitest / tsc / prettier、`scripts/architecture_guard.py`。零新 crate。

**需求来源：** `docs/plans/2026-08-14-prompt-runtime-assembly-design.md`。

---

## Global Constraints

- 仓库 `/Users/yuzaimu/projects/StoryForge`。在 **新分支** `feat/prompt-runtime-assembly` 上做（可 worktree `.worktrees/prompt-runtime-assembly`）。**不要用** `.worktrees/ink-paper` 或 `.worktrees/ink-paper-deepen`。
- 中文 conventional commit。不 `--no-verify`。**不推送、不打 tag、不 bump 版本**，发版等用户指令。
- **Commit 步骤**：仅当用户在本会话明确说「提交」时执行各 Task 末的 commit。未授权则做完代码+测试停在工作区。
- 改任何符号前必须 GitNexus `impact({target, direction:"upstream"})`。HIGH/CRITICAL 先报告用户。撰写时：`writer_first_chapter` LOW；`preview_prompt_composition` LOW；`ToolLoop.run` **MEDIUM**（11 直接调用）——Task 5 只换 head 拼接，禁止改循环控制。
- **`prompts` 不得 `use crate::agency`。** `agency` 可以 `use crate::prompts::assembly`。
- P0 **字符串等价**。禁止把创世/续写 complete() 的 system 换成 `agency_lead_writer_system.md`。
- 禁止改 `WriteTimeBundle::to_prompt()`、PersistMode、IPC struct 字段、`--ai-*` 名、落地页。
- P0 不收 `producer_depth_assets` / `concept_pack`。
- 行号会漂，执行以**锚点代码**定位。
- 准入线：`cd src-tauri && cargo test --lib` 只增不减（基线撰写时 1367 passed / 2 ignored）；`cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check`；仓库根 `python3 scripts/architecture_guard.py`。P0 无前端逻辑变更时 vitest 基线 589 / 3 skipped（Task 6 改 PromptsPanel 后只允许增加）。
- P0 验收通过才能开 P1。P1 验收通过才能开 P2。

---

## File map

| 文件 | 职责 |
|---|---|
| `src-tauri/src/prompts/assembly.rs` | **新建。** `Layer` / `assemble` / 创世·续写·tool_loop 工厂与常量 |
| `src-tauri/src/prompts/mod.rs` | `pub mod assembly` |
| `src-tauri/src/prompts/engine.rs` | P2：`leftover_mustache_idents`；运行时仍 fail-open |
| `src-tauri/src/prompts/registry.rs` | preview 改 Agency 场景；P2 CI 走 bundled |
| `src-tauri/src/agency/coordinator.rs` | 三条 complete() 改调工厂 |
| `src-tauri/src/agency/tool_loop.rs` | `run` 的 `head` 走 `assemble_tool_loop_head` |
| `src-tauri/src/agency/tools.rs` | P1：`usage_guidance` + catalog 追加 |
| `src-frontend/src/pages/settings/PromptsPanel.tsx` | 下拉默认 Agency 续写 |
| `src-frontend/src/pages/settings/__tests__/PromptsPanel.test.tsx` | 默认 scene 断言 |
| `scripts/architecture_guard.py` | `prompts` 禁止导入 `agency` |

不要改：`domain/write_time_bundle.rs` 的 `to_prompt`、`landing/`、`tauri.conf.json`、版本号。

---

### Task 1: `assemble()` 哑拼接器

**Files:**
- Create: `src-tauri/src/prompts/assembly.rs`
- Modify: `src-tauri/src/prompts/mod.rs`（在 `pub mod registry;` 旁加 `pub mod assembly;`）

**GitNexus:** 新建模块，无旧符号。确认 `mod.rs` 只加模块声明。

- [ ] **Step 1: 写失败测试（文件尚不存在，先建模块骨架再写测）**

`src-tauri/src/prompts/mod.rs` 现为：

```rust
pub mod commands;
pub mod engine;
pub mod registry;
pub use engine::TemplateEngine;
```

改为：

```rust
pub mod assembly;
pub mod commands;
pub mod engine;
pub mod registry;
pub use engine::TemplateEngine;
```

新建 `assembly.rs`，先放空 `assemble` 让测试能编译并失败——**不要**在这一步写工厂函数。测试放在文件底部 `#[cfg(test)] mod tests`：

```rust
use super::*;

fn layer(id: &'static str, slot: Slot, body: &str, required: bool) -> Layer {
    Layer {
        id,
        kind: LayerKind::Context,
        slot,
        body: body.to_string(),
        required,
    }
}

#[test]
fn assemble_joins_system_and_user_with_blank_line() {
    let out = assemble(&[
        layer("a", Slot::System, "SYS-A", true),
        layer("b", Slot::System, "SYS-B", true),
        layer("c", Slot::User, "USR-C", true),
        layer("d", Slot::User, "USR-D", true),
    ])
    .unwrap();
    assert_eq!(out.system, "SYS-A\n\nSYS-B");
    assert_eq!(out.user, "USR-C\n\nUSR-D");
}

#[test]
fn assemble_rejects_duplicate_id() {
    let err = assemble(&[
        layer("same", Slot::System, "x", true),
        layer("same", Slot::User, "y", true),
    ])
    .unwrap_err();
    assert_eq!(err, AssembleError::DuplicateId("same"));
}

#[test]
fn assemble_rejects_missing_required() {
    let err = assemble(&[layer("need", Slot::System, "  \n", true)]).unwrap_err();
    assert_eq!(err, AssembleError::MissingRequired("need"));
}

#[test]
fn assemble_skips_empty_optional() {
    let out = assemble(&[
        layer("keep", Slot::User, "BODY", true),
        layer("skip", Slot::User, "  ", false),
    ])
    .unwrap();
    assert_eq!(out.user, "BODY");
    assert!(out.system.is_empty());
}
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cd src-tauri && cargo test --lib prompts::assembly:: -- --nocapture
```

Expected: 编译失败（缺类型）或测试失败（`assemble` 未实现）。

- [ ] **Step 3: 最小实现**

`assembly.rs` 全文件（本 Task 不含创世工厂；工厂在 Task 2）：

```rust
//! Prompt assembly：Instruction + Context + Tools → (system, user)。
//! 哑拼接器。本模块不得依赖 `agency`。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    System,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerKind {
    Instruction,
    Context,
    Tools,
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub id: &'static str,
    pub kind: LayerKind,
    pub slot: Slot,
    pub body: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembledPrompt {
    pub system: String,
    pub user: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssembleError {
    DuplicateId(&'static str),
    MissingRequired(&'static str),
}

impl std::fmt::Display for AssembleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "prompt assembly: duplicate layer id `{id}`"),
            Self::MissingRequired(id) => {
                write!(f, "prompt assembly: required layer `{id}` is empty")
            }
        }
    }
}

impl std::error::Error for AssembleError {}

pub fn assemble(layers: &[Layer]) -> Result<AssembledPrompt, AssembleError> {
    let mut seen = std::collections::HashSet::new();
    let mut system_parts: Vec<&str> = Vec::new();
    let mut user_parts: Vec<&str> = Vec::new();

    for layer in layers {
        if !seen.insert(layer.id) {
            return Err(AssembleError::DuplicateId(layer.id));
        }
        let body = layer.body.trim();
        if body.is_empty() {
            if layer.required {
                return Err(AssembleError::MissingRequired(layer.id));
            }
            continue;
        }
        match layer.slot {
            Slot::System => system_parts.push(body),
            Slot::User => user_parts.push(body),
        }
    }

    Ok(AssembledPrompt {
        system: system_parts.join("\n\n"),
        user: user_parts.join("\n\n"),
    })
}
```

注意：`trim()` 会吃掉「创作资产：\n」末尾换行，但段本身仍非空。金标拼接用 trim 后的 body 再 `\n\n` join。创世工厂的 body 写成 trim 后仍含完整段标题（见 Task 2）。

**不要**在 `assemble` 里按 `kind` 排序。顺序 = 调用方传入顺序。

- [ ] **Step 4: 跑测试确认通过**

```bash
cd src-tauri && cargo test --lib prompts::assembly:: -- --nocapture
```

Expected: 4 passed。

- [ ] **Step 5: Commit**（仅用户说「提交」时）

```bash
git add src-tauri/src/prompts/mod.rs src-tauri/src/prompts/assembly.rs
git commit -m "$(cat <<'EOF'
feat: 新增 prompts::assembly 哑拼接器

EOF
)"
```

---

### Task 2: 创世 / 续写 / ToolLoop 工厂 + 字符串金标

**Files:**
- Modify: `src-tauri/src/prompts/assembly.rs`（追加常量、工厂、金标测试）

**GitNexus:** 仍在新模块。

- [ ] **Step 1: 写失败测试（工厂尚不存在）**

在 `assembly.rs` 的 `tests` 模块 **追加**（保留 Task 1 四测）。金标必须与 `coordinator.rs` / `tool_loop.rs` 当前字面量逐字相同：

```rust
#[test]
fn genesis_first_chapter_matches_legacy_format() {
    let premise = "一部间谍小说";
    let concept = "{\"logline\":\"x\"}";
    let assets = "【世界观】双星";
    let out = assemble_genesis_first_chapter(premise, concept, assets).unwrap();
    assert_eq!(
        out.system,
        "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾、不得发明与资产冲突的角色或设定。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。"
    );
    let expected_user = format!(
        "故事前提：{}\n\n概念设定：{}\n\n创作资产：\n{}\n\n写作要求：第一章正文，1500-2500 字，只输出正文，不写标题。须紧扣故事大纲的起因（第一幕）开篇。故事前提是你的创作方向；创作资产（世界观/大纲/伏笔）是硬约束，须在硬约束内落实前提核心意图，不得自相矛盾。",
        premise, concept, assets
    );
    assert_eq!(out.user, expected_user);
}

#[test]
fn genesis_prose_fallback_matches_legacy_format() {
    let out = assemble_genesis_prose_fallback("前提", "资产正文").unwrap();
    assert_eq!(
        out.system,
        "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。"
    );
    assert_eq!(
        out.user,
        "故事前提：前提\n\n创作资产：\n资产正文\n\n写作要求：章节正文，1500-2500 字，只输出正文，不写标题。故事前提是你的创作方向；创作资产（世界观/大纲/伏笔）是硬约束，须在硬约束内落实前提核心意图，不得自相矛盾。"
    );
}

#[test]
fn continue_beat_keeps_user_opaque_and_locks_system() {
    let user = "【节拍任务】\n去码头";
    let out = assemble_continue_beat(user).unwrap();
    assert_eq!(
        out.system,
        "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。须在节拍任务硬约束内落实指令。"
    );
    assert_eq!(out.user, user);
}

#[test]
fn continue_beat_anti_repeat_suffix_is_concat() {
    let out = assemble_continue_beat("u").unwrap();
    let retry = format!(
        "{} 禁止重复同一段落或意象循环，不得首尾回环。",
        out.system
    );
    assert!(retry.starts_with(&out.system));
    assert!(retry.contains("不得首尾回环"));
}

#[test]
fn tool_loop_head_matches_legacy_format() {
    let catalog = "可用工具（JSON action 调用）：\n- board_read: 读\n  参数: {}\n";
    let task = "写一章";
    let out = assemble_tool_loop_head(catalog, task).unwrap();
    assert!(out.system.is_empty());
    let expected = format!(
        "{}\n\n你只能输出一个 JSON action，不要输出其他内容：\n\
         - 调用工具: {{\"type\":\"tool\",\"name\":\"<工具名>\",\"args\":{{...}}}}\n\
         - 完成任务: {{\"type\":\"final\",\"content\":\"<最终产出>\"}}\n\n任务：\n{}",
        catalog, task
    );
    assert_eq!(out.user, expected);
}
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cd src-tauri && cargo test --lib prompts::assembly::genesis_first_chapter_matches_legacy_format -- --nocapture
```

Expected: 编译失败（函数不存在）。

- [ ] **Step 3: 实现工厂**

追加到 `assembly.rs`（`assemble` 之后、`#[cfg(test)]` 之前）。**常量正文禁止改字。**

`assemble()` 会对每层 `trim()`。创世 user 金标里「创作资产：\n{assets}」在 assets 无首尾空白时，trim 整层会变成 `创作资产：\n{assets}`（末尾无多余换行），再与下一层 `\n\n` 拼接，结果等于旧 `format!`。

**陷阱：** 若把四段合成一层，测试仍过但失去分层。必须四层（首章）/ 三层（散文回退）。

```rust
pub const GENESIS_FIRST_CHAPTER_SYSTEM: &str = "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾、不得发明与资产冲突的角色或设定。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。";

pub const GENESIS_PROSE_FALLBACK_SYSTEM: &str = "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。";

pub const CONTINUE_BEAT_SYSTEM: &str = "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。须在节拍任务硬约束内落实指令。";

pub const TOOL_LOOP_PROTOCOL: &str = "你只能输出一个 JSON action，不要输出其他内容：\n\
- 调用工具: {\"type\":\"tool\",\"name\":\"<工具名>\",\"args\":{...}}\n\
- 完成任务: {\"type\":\"final\",\"content\":\"<最终产出>\"}";

const GENESIS_FIRST_CHAPTER_TASK: &str = "写作要求：第一章正文，1500-2500 字，只输出正文，不写标题。须紧扣故事大纲的起因（第一幕）开篇。故事前提是你的创作方向；创作资产（世界观/大纲/伏笔）是硬约束，须在硬约束内落实前提核心意图，不得自相矛盾。";

const GENESIS_PROSE_FALLBACK_TASK: &str = "写作要求：章节正文，1500-2500 字，只输出正文，不写标题。故事前提是你的创作方向；创作资产（世界观/大纲/伏笔）是硬约束，须在硬约束内落实前提核心意图，不得自相矛盾。";

fn l(
    id: &'static str,
    kind: LayerKind,
    slot: Slot,
    body: String,
    required: bool,
) -> Layer {
    Layer {
        id,
        kind,
        slot,
        body,
        required,
    }
}

pub fn assemble_genesis_first_chapter(
    premise: &str,
    concept_json: &str,
    assets_ctx: &str,
) -> Result<AssembledPrompt, AssembleError> {
    assemble(&[
        l(
            "instruction",
            LayerKind::Instruction,
            Slot::System,
            GENESIS_FIRST_CHAPTER_SYSTEM.to_string(),
            true,
        ),
        l(
            "premise",
            LayerKind::Context,
            Slot::User,
            format!("故事前提：{premise}"),
            true,
        ),
        l(
            "concept",
            LayerKind::Context,
            Slot::User,
            format!("概念设定：{concept_json}"),
            true,
        ),
        l(
            "assets",
            LayerKind::Context,
            Slot::User,
            format!("创作资产：\n{assets_ctx}"),
            true,
        ),
        l(
            "task",
            LayerKind::Context,
            Slot::User,
            GENESIS_FIRST_CHAPTER_TASK.to_string(),
            true,
        ),
    ])
}

pub fn assemble_genesis_prose_fallback(
    premise: &str,
    assets_ctx: &str,
) -> Result<AssembledPrompt, AssembleError> {
    assemble(&[
        l(
            "instruction",
            LayerKind::Instruction,
            Slot::System,
            GENESIS_PROSE_FALLBACK_SYSTEM.to_string(),
            true,
        ),
        l(
            "premise",
            LayerKind::Context,
            Slot::User,
            format!("故事前提：{premise}"),
            true,
        ),
        l(
            "assets",
            LayerKind::Context,
            Slot::User,
            format!("创作资产：\n{assets_ctx}"),
            true,
        ),
        l(
            "task",
            LayerKind::Context,
            Slot::User,
            GENESIS_PROSE_FALLBACK_TASK.to_string(),
            true,
        ),
    ])
}

pub fn assemble_continue_beat(user: &str) -> Result<AssembledPrompt, AssembleError> {
    assemble(&[
        l(
            "instruction",
            LayerKind::Instruction,
            Slot::System,
            CONTINUE_BEAT_SYSTEM.to_string(),
            true,
        ),
        l(
            "continue_user",
            LayerKind::Context,
            Slot::User,
            user.to_string(),
            true,
        ),
    ])
}

pub fn assemble_tool_loop_head(
    catalog: &str,
    task: &str,
) -> Result<AssembledPrompt, AssembleError> {
    assemble(&[
        l(
            "tools",
            LayerKind::Tools,
            Slot::User,
            catalog.to_string(),
            true,
        ),
        l(
            "protocol",
            LayerKind::Instruction,
            Slot::User,
            TOOL_LOOP_PROTOCOL.to_string(),
            true,
        ),
        l(
            "task",
            LayerKind::Context,
            Slot::User,
            format!("任务：\n{task}"),
            true,
        ),
    ])
}
```

**`TOOL_LOOP_PROTOCOL` 陷阱：** 旧 `format!` 里协议段是：

```
你只能输出一个 JSON action，不要输出其他内容：\n\
 - 调用工具: ...
```

注意「- 调用工具」前是否有空格。以 `tool_loop.rs` 源码为准逐字复制，金标测试用同一字面量。若 `assemble` 的 `trim()` 吃掉 catalog 末尾 `\n`，`join("\n\n")` 仍是 `catalog_trimmed + "\n\n" + protocol`，而旧 `format!("{}\\n\\n{}", catalog, ...)` 在 catalog 已以 `\n` 结尾时会多一个换行。

**必须先跑金标。** 若 `tool_loop_head_matches_legacy_format` 因 catalog 尾 `\n` 失败：不要改 `assemble()` 的 trim 规则（会破坏创世金标）。改为工厂对 catalog **不 trim 拼接**——即 `assemble_tool_loop_head` 不用通用 `assemble` 的 trim 吃掉 catalog 的尾换行，或 catalog 层 `required` 仍走 assemble 但 join 前对 Tools 层保留原始 body。

选定实现（写进代码，不要两种并存）：**`assemble` 保持 trim**（创世需要）。`assemble_tool_loop_head` 用与旧 `format!` 相同的三个参数拼接，内部仍构建 Layer 并 `assemble` 校验 id；若 trim 导致与旧串不等，则：

```rust
pub fn assemble_tool_loop_head(catalog: &str, task: &str) -> Result<AssembledPrompt, AssembleError> {
    let assembled = assemble(&[ /* 三层，catalog 原样 */ ])?;
    Ok(AssembledPrompt {
        system: String::new(),
        user: format!(
            "{}\n\n{}\n\n任务：\n{}",
            catalog, TOOL_LOOP_PROTOCOL, task
        ),
    })
}
```

这样 `assemble()` 仍校验重复 id / required，对外 user **强制**等于旧 `format!`。测试锁的是对外 user。若走这条，在工厂旁注释「P0 对外串等价；trim 与历史 format 对 catalog 尾 NL 不一致」。

- [ ] **Step 4: 跑测试确认通过**

```bash
cd src-tauri && cargo test --lib prompts::assembly:: -- --nocapture
```

Expected: Task 1 四测 + 本 Task 五测全绿。

- [ ] **Step 5: Commit**（仅用户说「提交」时）

```bash
git add src-tauri/src/prompts/assembly.rs
git commit -m "$(cat <<'EOF'
feat: 锁定创世/续写/ToolLoop 组装金标

EOF
)"
```

---

### Task 3: 接线创世 `writer_first_chapter` / `writer_prose_fallback`

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs`（锚点：`async fn writer_first_chapter` 内 `llm.complete(`；`async fn writer_prose_fallback` 内同样）

**GitNexus:** `impact({target: "writer_first_chapter", direction: "upstream"})`。预期 LOW。若 HIGH/CRITICAL 停下来报告。`writer_prose_fallback` 同样跑一遍。

- [ ] **Step 1: 现有回归必须先绿（表征）**

```bash
cd src-tauri && cargo test --lib agency::tests::test_legacy_writer_prose_fallback agency::tests::test_continue_writer_prose_fallback -- --nocapture
```

Expected: 已有测试通过（改前基线）。若名字略有出入，`rg "prose_fallback" src-tauri/src/agency/tests.rs` 定位。

- [ ] **Step 2: 替换 complete 入参**

在 `coordinator.rs` 文件顶部的 `use` 区追加：

```rust
use crate::prompts::assembly::{
    assemble_continue_beat, assemble_genesis_first_chapter, assemble_genesis_prose_fallback,
};
```

（`assemble_continue_beat` 供 Task 4；本 Task 一起 import 以免 Task 4 再改 use。）

`writer_first_chapter` 中把：

```rust
        let text = llm.complete(
            "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾、不得发明与资产冲突的角色或设定。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。",
            &format!("故事前提：{}\n\n概念设定：{}\n\n创作资产：\n{}\n\n写作要求：第一章正文，1500-2500 字，只输出正文，不写标题。须紧扣故事大纲的起因（第一幕）开篇。故事前提是你的创作方向；创作资产（世界观/大纲/伏笔）是硬约束，须在硬约束内落实前提核心意图，不得自相矛盾。", premise, concept_json, assets_ctx),
            TaskType::CreativeWriting,
            8192,
        ).await?;
```

换成：

```rust
        let assembled = assemble_genesis_first_chapter(premise, &concept_json, &assets_ctx)
            .map_err(|e| AppError::from(e.to_string()))?;
        let text = llm
            .complete(
                &assembled.system,
                &assembled.user,
                TaskType::CreativeWriting,
                8192,
            )
            .await?;
```

`writer_prose_fallback` 中把 `llm.complete("你是小说主创…", &format!("故事前提：{}…", premise, assets_ctx), …)` 换成：

```rust
        let assembled = assemble_genesis_prose_fallback(premise, &assets_ctx)
            .map_err(|e| AppError::from(e.to_string()))?;
        let text = llm
            .complete(
                &assembled.system,
                &assembled.user,
                TaskType::CreativeWriting,
                8192,
            )
            .await?;
```

`chapter_key` 仍只用于 `board.write` 的 key，不进 prompt。

- [ ] **Step 3: 确认内联 system 字面量已消失**

```bash
rg "你是小说主创，只输出章节正文" src-tauri/src/agency/coordinator.rs
```

Expected: **零命中**（Task 4 之前 `write_beat_once` 仍会命中一次；本 Step 只要求 `writer_first_chapter` / `writer_prose_fallback` 函数体内不再有该字面量）。若 Task 3 单独执行，允许 `write_beat_once` 仍命中。

- [ ] **Step 4: 跑测试**

```bash
cd src-tauri && cargo test --lib agency::tests::test_legacy_writer_prose_fallback agency::tests::test_continue_writer_prose_fallback prompts::assembly:: -- --nocapture
```

Expected: 全绿。

- [ ] **Step 5: Commit**（仅用户说「提交」时）

```bash
git add src-tauri/src/agency/coordinator.rs
git commit -m "$(cat <<'EOF'
refactor: 创世主创 complete 改走 prompt assembly

EOF
)"
```

---

### Task 4: 接线 `write_beat_once` system

**Files:**
- Modify: `src-tauri/src/agency/coordinator.rs`（锚点：`let system = "你是小说主创，只输出章节正文。` 与 `let retry_system = format!("{system} 禁止重复`)

**GitNexus:** `impact({target: "write_beat_once", direction: "upstream"})`。未入索引则用 `file_path: "src-tauri/src/agency/coordinator.rs"`。LOW/MEDIUM 可继续；HIGH 先报告。

- [ ] **Step 1: 写/更新表征测试**

已有 `assembled_user_prompt_omits_non_admitted_emotional_core`（`coordinator.rs` 内）与 `writer_prompt_order_is_card_then_body_then_summary_then_ending`（`beat_card.rs`）必须继续绿。不要改它们的断言。

在 `assembly.rs` tests 已有 `continue_beat_anti_repeat_suffix_is_concat`。本 Task 只接线。

- [ ] **Step 2: 替换**

找到 `write_beat_once` 里 user 已由 `render_writer_user_prompt` / `assemble_continue_user_prompt` 得到之后、`llm.complete` 之前：

把

```rust
        let system = "你是小说主创，只输出章节正文。人设、世界观与已埋伏笔以下方资产区为准，不得自相矛盾。禁止重复：同一段落/句子不得出现两次，不得复述已有正文。须在节拍任务硬约束内落实指令。";
        let text = match llm
            .complete(system, &user, TaskType::CreativeWriting, 8192)
```

换成：

```rust
        let assembled = assemble_continue_beat(&user)
            .map_err(|e| AppError::from(e.to_string()))?;
        let system = assembled.system;
        let user = assembled.user;
        let text = match llm
            .complete(&system, &user, TaskType::CreativeWriting, 8192)
```

anti-repeat 分支保持：

```rust
            let retry_system = format!("{system} 禁止重复同一段落或意象循环，不得首尾回环。");
            if let Ok(retry) = llm
                .complete(&retry_system, &user, TaskType::CreativeWriting, 8192)
```

`system` 改为 `String` 后，`format!("{system} …")` 仍合法。

- [ ] **Step 3: 确认 coordinator 内联主创 system 清零**

```bash
rg "你是小说主创，只输出章节正文" src-tauri/src/agency/coordinator.rs
```

Expected: 零命中。

- [ ] **Step 4: 跑测试**

```bash
cd src-tauri && cargo test --lib prompts::assembly:: agency::beat_card:: assembled_user_prompt_omits -- --nocapture
```

Expected: 全绿。再跑：

```bash
cd src-tauri && cargo test --lib
```

Expected: ≥1367 passed / 2 ignored，只增不减。

- [ ] **Step 5: Commit**（仅用户说「提交」时）

```bash
git add src-tauri/src/agency/coordinator.rs
git commit -m "$(cat <<'EOF'
refactor: 续写 write_beat_once 改走 prompt assembly

EOF
)"
```

---

### Task 5: 接线 `ToolLoop::run` head（MEDIUM）

**Files:**
- Modify: `src-tauri/src/agency/tool_loop.rs`（锚点：`let head = format!(`）

**GitNexus:** `impact({target: "run", file_path: "src-tauri/src/agency/tool_loop.rs", direction: "upstream"})`。撰写时 **MEDIUM / 11 直接调用**。本 Task **只替换 head 字符串构造**，禁止改 `max_turns` / deadline / parse_action / observation 拼接。风险来自调用面宽，不是控制流变更。若 impact 升到 HIGH/CRITICAL，停下来报告用户。

- [ ] **Step 1: 现有 tool_loop 测试必须先绿**

```bash
cd src-tauri && cargo test --lib agency::tool_loop:: -- --nocapture
```

Expected: 全绿。

- [ ] **Step 2: 替换 head**

文件顶部追加：

```rust
use crate::prompts::assembly::assemble_tool_loop_head;
```

把

```rust
        let head = format!(
            "{}\n\n你只能输出一个 JSON action，不要输出其他内容：\n\
             - 调用工具: {{\"type\":\"tool\",\"name\":\"<工具名>\",\"args\":{{...}}}}\n\
             - 完成任务: {{\"type\":\"final\",\"content\":\"<最终产出>\"}}\n\n任务：\n{}",
            self.registry.catalog_for_role(role),
            task
        );
```

换成：

```rust
        let head = assemble_tool_loop_head(&self.registry.catalog_for_role(role), task)
            .map_err(|e| AppError::from(e.to_string()))?
            .user;
```

不要动 `system_prompt` 参数；角色 md 仍由调用方传入 `run()`。

- [ ] **Step 3: 确认 tool_loop 不再内联协议字面量**

```bash
rg "你只能输出一个 JSON action" src-tauri/src/agency/tool_loop.rs
```

Expected: 零命中（常量已迁到 `assembly.rs`）。

- [ ] **Step 4: 跑测试**

```bash
cd src-tauri && cargo test --lib agency::tool_loop:: prompts::assembly::tool_loop_head_matches_legacy_format -- --nocapture
```

Expected: 全绿。然后 `cargo test --lib` 只增不减。

- [ ] **Step 5: Commit**（仅用户说「提交」时）

```bash
git add src-tauri/src/agency/tool_loop.rs
git commit -m "$(cat <<'EOF'
refactor: ToolLoop head 改走 prompt assembly

EOF
)"
```

---

### Task 6: 场景预览改报 Agency 热路径

**Files:**
- Modify: `src-tauri/src/prompts/registry.rs`（锚点：`pub fn preview_prompt_composition`；测试 `test_preview_prompt_composition_timesliced`）
- Modify: `src-frontend/src/pages/settings/PromptsPanel.tsx`（锚点：`COMPOSITION_SCENES` 与 `useState<string>('timesliced')`）
- Modify: `src-frontend/src/pages/settings/__tests__/PromptsPanel.test.tsx`（锚点：`scene: 'timesliced'`）

**GitNexus:** `impact({target: "preview_prompt_composition", file_path: "src-tauri/src/prompts/registry.rs", direction: "upstream"})`。预期 LOW。不要给 `PromptCompositionPreview` 加字段。

- [ ] **Step 1: 改 Rust 测试（先红）**

替换 `test_preview_prompt_composition_timesliced` / `test_preview_prompt_composition_trishot` 为：

```rust
    #[test]
    fn test_preview_prompt_composition_agency_continue() {
        let preview = preview_prompt_composition("agency_continue");
        assert_eq!(preview.scene, "agency_continue");
        assert_eq!(preview.scene_label, "Agency 续写");
        assert!(preview
            .layers
            .iter()
            .any(|l| l.prompt_id == "inline.agency_continue_system"));
        assert!(preview
            .layers
            .iter()
            .any(|l| l.prompt_id == "compiler.scene_beat_card"));
        assert!(!preview
            .layers
            .iter()
            .any(|l| l.prompt_id == "orchestrator_timesliced_writer"));
    }

    #[test]
    fn test_preview_prompt_composition_agency_genesis() {
        let preview = preview_prompt_composition("agency_genesis");
        assert_eq!(preview.scene, "agency_genesis");
        assert!(preview
            .layers
            .iter()
            .any(|l| l.prompt_id == "inline.agency_genesis_system"));
        assert!(!preview
            .layers
            .iter()
            .any(|l| l.prompt_id == "trishot_synthesizer"));
    }

    #[test]
    fn test_preview_timesliced_alias_maps_to_agency_continue() {
        let preview = preview_prompt_composition("timesliced");
        assert_eq!(preview.scene, "agency_continue");
    }

    #[test]
    fn test_preview_trishot_alias_maps_to_agency_genesis() {
        let preview = preview_prompt_composition("trishot_call3");
        assert_eq!(preview.scene, "agency_genesis");
    }
```

- [ ] **Step 2: 跑 Rust 测试确认失败**

```bash
cd src-tauri && cargo test --lib prompts::registry::test_preview_prompt_composition_agency_continue -- --nocapture
```

Expected: FAIL（仍返回 timesliced / writer_system）。

- [ ] **Step 3: 实现 match 臂**

把 `preview_prompt_composition` 的 match 改成（`pipeline_review` 臂原样保留）：

```rust
    let (scene_key, scene_label, specs): (&str, &str, &[(&str, &str, &str)]) = match scene {
        "agency_genesis" | "trishot_call3" | "trishot" | "genesis" => (
            "agency_genesis",
            "Agency 创世",
            &[
                ("instruction", "inline.agency_genesis_system", "inline"),
                ("context", "compiler.premise", "compiler"),
                ("context", "compiler.concept", "compiler"),
                ("context", "compiler.assets", "compiler"),
                ("context", "compiler.task", "compiler"),
            ],
        ),
        "pipeline_review" | "review" => (
            "pipeline_review",
            "审稿流水线",
            &[
                ("system", "pipeline_review", "review_system"),
                ("criteria", "review_contract_criteria", "contract"),
            ],
        ),
        // 默认与 timesliced 别名：Agency 续写热路径
        _ => (
            "agency_continue",
            "Agency 续写",
            &[
                ("instruction", "inline.agency_continue_system", "inline"),
                ("context", "compiler.scene_beat_card", "compiler"),
                ("context", "compiler.continue_assets", "compiler"),
            ],
        ),
    };
```

`prompt_display_name` 对未知 id 已回退为 id 本身，无需新字段。

- [ ] **Step 4: 前端下拉**

`PromptsPanel.tsx`：

```ts
const COMPOSITION_SCENES = [
  { value: 'agency_continue', label: 'Agency 续写' },
  { value: 'agency_genesis', label: 'Agency 创世' },
  { value: 'pipeline_review', label: '审稿流水线' },
] as const;
```

```ts
  const [compositionScene, setCompositionScene] = useState<string>('agency_continue');
```

层点击：`source` 为 `inline` / `compiler` 时不要 `jumpToPrompt`（注册表里没有这些 id）：

```tsx
                  onClick={() => {
                    if (layer.source === 'inline' || layer.source === 'compiler') return;
                    jumpToPrompt(layer.prompt_id);
                  }}
```

`PromptsPanel.test.tsx` 把 `scene: 'timesliced'` 和 `toHaveValue('timesliced')` 改成 `agency_continue`。

- [ ] **Step 5: 跑验证**

```bash
cd src-tauri && cargo test --lib prompts::registry::test_preview -- --nocapture
cd src-frontend && npx tsc --noEmit && npx vitest run src/pages/settings/__tests__/PromptsPanel.test.tsx && npm run format:check
python3 scripts/architecture_guard.py
```

Expected: Rust 预览四测绿；vitest 该文件绿；tsc / format / guard 绿。

- [ ] **Step 6: Commit**（仅用户说「提交」时）

```bash
git add src-tauri/src/prompts/registry.rs src-frontend/src/pages/settings/PromptsPanel.tsx src-frontend/src/pages/settings/__tests__/PromptsPanel.test.tsx
git commit -m "$(cat <<'EOF'
fix: 提示词场景预览改为 Agency 续写/创世

EOF
)"
```

---

### Task 7: architecture_guard 锁 `prompts`↛`agency`

**Files:**
- Modify: `scripts/architecture_guard.py`（锚点：`PROHIBITED = {`）

**GitNexus:** 脚本非 Rust 符号。

- [ ] **Step 1: 改规则**

```python
PROHIBITED = {
    "db": {"narrative", "agents", "memory", "creative_engine", "story_system", "pipeline"},
    "domain": MODULES,  # domain 只应依赖基础库，理论上不应依赖任何业务模块
    "prompts": {"agency"},
}
```

- [ ] **Step 2: 跑守卫**

```bash
python3 scripts/architecture_guard.py
```

Expected: PASSED。若失败，说明 `prompts/` 已有 `use crate::agency`——删掉该 use，把调用留在 `agency` 侧。

- [ ] **Step 3: P0 总回归**

```bash
cd src-tauri && cargo test --lib
cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check
python3 scripts/architecture_guard.py
```

Expected: cargo 只增不减；vitest ≥589 / 3 skipped；其余绿。对照设计 §8 P0 清单逐条勾。

- [ ] **Step 4: Commit**（仅用户说「提交」时）

```bash
git add scripts/architecture_guard.py
git commit -m "$(cat <<'EOF'
chore: architecture_guard 禁止 prompts 依赖 agency

EOF
)"
```

---

### Task 8: P1 工具 `usage_guidance`

**前置：** P0 清单已勾。本 Task 会改变 ToolLoop catalog 字节，**不再**字符串等价。

**Files:**
- Modify: `src-tauri/src/agency/tools.rs`（锚点：`trait AgentTool`；`fn catalog_for_role`；`BoardReadTool` / `BoardWriteTool` / `AssetQueryTool` 的 `impl`）

**GitNexus:** `impact({target: "catalog_for_role", direction: "upstream"})` 与 `impact({target: "AgentTool", direction: "upstream"})`。trait 加默认方法，所有 impl 不改也能编过。若 HIGH，先报告。

- [ ] **Step 1: 写失败测试**

在 `tools.rs` 已有 `#[cfg(test)]` 末尾追加；若无测试模块，在文件底新建。用 `ToolRegistry::agency_default()`：

```rust
    #[test]
    fn catalog_without_guidance_keeps_name_description_schema_lines() {
        let reg = ToolRegistry::agency_default();
        let cat = reg.catalog_for_role(crate::agency::models::AgentRole::EditorAuditor);
        assert!(cat.contains("board_read"));
        assert!(cat.contains("参数:"));
    }

    #[test]
    fn catalog_includes_usage_for_read_write_query() {
        let reg = ToolRegistry::agency_default();
        let writer = reg.catalog_for_role(crate::agency::models::AgentRole::LeadWriter);
        assert!(writer.contains("用法: 资产已注入时不要轮询 board_read 拉全文"));
        assert!(writer.contains("用法: 正文写入 draft 区，勿覆盖 user_created 资产"));
        assert!(writer.contains("用法: 按 kind 查询，不要倾倒全表"));
        let editor = reg.catalog_for_role(crate::agency::models::AgentRole::EditorAuditor);
        assert!(editor.contains("用法: 资产已注入时不要轮询 board_read 拉全文"));
        assert!(!editor.contains("用法: 正文写入 draft 区，勿覆盖 user_created 资产"));
    }
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cd src-tauri && cargo test --lib catalog_includes_usage_for_read_write_query -- --nocapture
```

Expected: FAIL（无「用法:」）。

- [ ] **Step 3: 实现**

`AgentTool` trait 追加默认方法（放在 `args_schema` 之后、`execute` 之前）：

```rust
    fn usage_guidance(&self) -> Option<&'static str> {
        None
    }
```

`catalog_for_role` 的 format 从：

```rust
                    out.push_str(&format!(
                        "- {}: {}\n  参数: {}\n",
                        tool.name(),
                        tool.description(),
                        tool.args_schema()
                    ));
```

改为：

```rust
                    out.push_str(&format!(
                        "- {}: {}\n  参数: {}\n",
                        tool.name(),
                        tool.description(),
                        tool.args_schema()
                    ));
                    if let Some(usage) = tool.usage_guidance() {
                        out.push_str(&format!("  用法: {}\n", usage));
                    }
```

三个 impl：

`BoardReadTool`：

```rust
    fn usage_guidance(&self) -> Option<&'static str> {
        Some("资产已注入时不要轮询 board_read 拉全文；只需补读遗漏 key")
    }
```

`BoardWriteTool`：

```rust
    fn usage_guidance(&self) -> Option<&'static str> {
        Some("正文写入 draft 区，勿覆盖 user_created 资产")
    }
```

`AssetQueryTool`：

```rust
    fn usage_guidance(&self) -> Option<&'static str> {
        Some("按 kind 查询，不要倾倒全表")
    }
```

其余工具不 override。

- [ ] **Step 4: 跑测试**

```bash
cd src-tauri && cargo test --lib catalog_includes_usage_for_read_write_query catalog_without_guidance_keeps_name_description_schema_lines agency::tool_loop:: -- --nocapture
```

Expected: 全绿。再 `cargo test --lib`。

- [ ] **Step 5: Commit**（仅用户说「提交」时）

```bash
git add src-tauri/src/agency/tools.rs
git commit -m "$(cat <<'EOF'
feat: board_read/write 与 asset_query 自带模型用法

EOF
)"
```

---

### Task 9: P2 内置模板 leftover `{{ident}}` CI

**前置：** P1 绿。运行时 `TemplateEngine` **继续 fail-open**（未知变量保留原文）。

**Files:**
- Modify: `src-tauri/src/prompts/engine.rs`（追加 `leftover_mustache_idents` + 单测）
- Modify: `src-tauri/src/prompts/registry.rs`（`resolve_prompt_with_vars` warn；CI 测试）

**GitNexus:** `impact({target: "resolve_prompt_with_vars", direction: "upstream"})`。只加 warn，不改返回值。若 HIGH，先报告。

- [ ] **Step 1: scanner 失败测试**

在 `engine.rs` 现有 tests 追加：

```rust
    #[test]
    fn leftover_mustache_ignores_if_blocks_and_finds_idents() {
        let text = "{{#if x}}keep{{/if}} {{ghost}} {{else}} {{world_rules}}";
        let left = leftover_mustache_idents(text);
        assert!(left.contains(&"ghost".to_string()));
        assert!(left.contains(&"world_rules".to_string()));
        assert!(!left.iter().any(|s| s.contains('#')));
    }

    #[test]
    fn leftover_mustache_empty_when_filled() {
        let mut vars = HashMap::new();
        vars.insert("name".into(), "x".into());
        let rendered = TemplateEngine::render("Hello {{name}}", &vars);
        assert!(leftover_mustache_idents(&rendered).is_empty());
    }
```

- [ ] **Step 2: 跑测试确认失败**

```bash
cd src-tauri && cargo test --lib prompts::engine::leftover_mustache_ignores_if_blocks_and_finds_idents -- --nocapture
```

Expected: 编译失败。

- [ ] **Step 3: 实现 scanner**

放在 `TemplateEngine` impl **外面**：

```rust
/// 渲染后仍残留的 `{{ident}}`。忽略 `{{#if}}` / `{{/if}}` / `{{else}}`。
pub fn leftover_mustache_idents(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            break;
        };
        let inner = after[..end].trim();
        rest = &after[end + 2..];
        if inner.is_empty()
            || inner.starts_with('#')
            || inner.starts_with('/')
            || inner.eq_ignore_ascii_case("else")
        {
            continue;
        }
        if inner
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            out.push(inner.to_string());
        }
    }
    out
}
```

`resolve_prompt_with_vars` 在 `render_with_conditions` 之后：

```rust
    let rendered =
        crate::prompts::engine::TemplateEngine::render_with_conditions(&template, vars);
    let leftover = crate::prompts::engine::leftover_mustache_idents(&rendered);
    if !leftover.is_empty() {
        log::warn!(
            "prompt `{prompt_id}` leftover placeholders after render: {:?}",
            leftover
        );
    }
    Ok(rendered)
```

**禁止**因此返回 Err。

- [ ] **Step 4: bundled CI 测试**

在 `registry.rs` tests 追加：

```rust
    fn is_fail_closed_prompt_id(id: &str) -> bool {
        id.starts_with("agency_") || id.starts_with("writer_") || id == "scene_outline"
    }

    #[test]
    fn builtin_agency_writer_scene_outline_have_no_undeclared_placeholders() {
        let prompts = get_builtin_prompts();
        let mut failures = Vec::new();
        for (id, entry) in prompts.iter() {
            if !is_fail_closed_prompt_id(id) {
                continue;
            }
            let mut vars = std::collections::HashMap::new();
            for v in &entry.variables {
                vars.insert(v.clone(), "x".into());
            }
            let rendered =
                crate::prompts::engine::TemplateEngine::render_with_conditions(
                    &entry.default_content,
                    &vars,
                );
            let leftover = crate::prompts::engine::leftover_mustache_idents(&rendered);
            if !leftover.is_empty() {
                failures.push(format!("{id}: {:?}", leftover));
            }
        }
        assert!(
            failures.is_empty(),
            "bundled prompts have leftover {{{{ident}}}} after filling declared variables:\n{}",
            failures.join("\n")
        );
    }

    #[test]
    fn leftover_scanner_catches_undeclared_fixture() {
        let leftover =
            crate::prompts::engine::leftover_mustache_idents("hello {{ghost}} world");
        assert_eq!(leftover, vec!["ghost".to_string()]);
    }
```

若 `builtin_agency_writer_scene_outline_have_no_undeclared_placeholders` 失败：对列出的 id，把 body 里的 `{{ident}}` 补进 YAML `variables:`，或改掉未声明占位。**禁止**放宽 scanner、禁止把失败 id 加入跳过名单。

- [ ] **Step 5: 跑全量**

```bash
cd src-tauri && cargo test --lib
cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check
python3 scripts/architecture_guard.py
```

Expected: cargo 只增不减；前端与 guard 绿。对照设计 §8 P1+P2。

- [ ] **Step 6: Commit**（仅用户说「提交」时）

```bash
git add src-tauri/src/prompts/engine.rs src-tauri/src/prompts/registry.rs
git commit -m "$(cat <<'EOF'
test: 内置 agency/writer/scene_outline 模板变量 CI fail-closed

EOF
)"
```

---

## Self-review（对照设计）

| 设计条目 | Task |
|---|---|
| `assemble()` 重复 id / required / 空跳过 | 1 |
| P0 创世 `writer_first_chapter` / `writer_prose_fallback` 字符串等价 | 2, 3 |
| P0 `write_beat_once` + anti-repeat 后缀 | 2, 4 |
| P0 ToolLoop head | 2, 5 |
| preview Agency + 别名 + 面板 | 6 |
| prompts↛agency | 7 |
| 不改 `to_prompt()` / IPC 字段 / 不换 `agency_lead_writer_system.md` | Constraints |
| P1 usage_guidance 三工具 | 8 |
| P2 leftover CI + 运行时 warn、engine fail-open | 9 |
| P3 producer/concept_pack | 刻意不做 |

无 TBD / 「类似 Task N」占位。`TOOL_LOOP_PROTOCOL` 与 catalog 尾 NL 的 trim 冲突已在 Task 2 给出选定实现。
