# 技能书提炼全面升级档设计文档

日期：2026-08-09
状态：已批准
前置：`2026-08-09-guidebook-distillation-upgrade-design.md`（T1-T6 已实施，四类结构化资产 + 重试 + 免费）
灵感来源：[book-to-skill](https://github.com/virgiliojr94/book-to-skill) Mode 4（Update/Fold-in）、`tools/validate_skill.py`、SKILL.md 输出格式

## 背景

上一档实施时将三项列为"明确不做（YAGNI）"，现用户要求全面实施：增量合并（fold-in）、提炼结果校验器、SKILL.md 导出。

## F1 增量合并（fold-in）

**用户故事**：用户已上传《故事》提炼出方法论 A；再上传《冲突工程学》时选择"合并进 A"，两本书的创作智慧融合为一个更强的方法论，而不是两个割裂的方法论。

### 数据流

```
upload_guidebook(file_path, merge_into=Some(cm_id))
  → 同一提炼流水线（metadata → chunk 四类资产 → merge_assets）
  → 【新】distill_foldin prompt：新资产 + 现有 CM 资产 → 融合去重
     输入：现有 principles/techniques/decision_rules/anti_patterns + 新提炼同类资产
     输出：融合后四类资产（techniques 保留双方原始命名，语义重复合并）
  → 融合后 principles 重跑 distill_methodology 生成新 steps（方法论保持单一连贯）
  → 更新现有 CM（name 保留、description 保留、steps/patterns/cheatsheet 替换为融合结果）
  → guidebooks 记录 methodology_id = cm_id、merge_into_methodology_id = cm_id
```

### 决策与理由

- **steps 重新生成而非拼接**：两套步骤拼接产生顺序混乱的缝合体；从融合 principles 重跑 `distill_methodology` 成本仅多一次 LLM 调用，产物连贯。CM 的 `name`/`description` 保留（用户可能已改名），`enabled` 不动。
- **引用该 CM 的故事**：`methodology_step` 可能被融合后的新 steps 数量顶格——落库后 clamp：`min(原 step, 新 max_steps)`。
- **V126 迁移**：`ALTER TABLE guidebooks ADD COLUMN merge_into_methodology_id TEXT`（无 FK，CM 被删不影响书记录）。记录合并意图，retry 路径读取并沿用。
- **hash 去重交互**：fold-in 上传命中 completed 记录 → 返回旧 id（不重复提炼）；命中 failed/cancelled → retry，retry 从该记录读 `merge_into_methodology_id` 继续 fold-in。
- **CM 侧来源**：`custom_methodologies.guidebook_id` 保持指向首本书（不改）；第二本书的 `guidebooks.methodology_id` 指向同一 CM。`list_all_methodologies` 的 source_book 逻辑不变。

### 错误处理

- `merge_into` 指向不存在的 CM / 内置方法论 id（非 `custom_` 前缀）→ 上传即报错
- fold-in LLM 失败 → 该次提炼 failed，可重试（重试保留 merge 意图）
- 现有 CM 被禁用 → 仍允许合并（合并后保持禁用，用户自行启用）

## F2 提炼结果校验器

对标 `validate_skill.py`：确定性校验，落库前自动清洗，纯 Rust 零 LLM。

### `validate_and_clean_output(output) -> (CleanedOutput, Vec<CleanReport>)`

- 硬校验（既有，保留）：方法论 name 非空、steps 非空、instruction 非空（失败 → 触发既有重试一次）
- 自动清洗（新增）：
  - techniques：剔除 `name` 空白项；按 `name` 去重（保留首个）；`when_to_use`/`how` 超 200 字截断
  - decision_rules：剔除空白项；去重；超 200 字截断
  - anti_patterns：剔除 `what` 空白项；按 `what` 去重
  - steps：`title` 超 20 字截断、`instruction` 超 500 字截断（防爆 token）
- 质量指标：清洗后 log::info 输出（技巧数/规则数/反模式数/剔除条数），供调 prompt 参考
- 前端零改动（清洗静默）

## F3 SKILL.md 导出

把自定义方法论导出为 book-to-skill 同款 SKILL.md，供 Claude Code / Copilot CLI / Amp 等支持 Agent Skills 标准的 agent 加载。

### `render_skill_md(cm, source_book) -> String`

```markdown
---
name: <cm.name 的 slug 化>
description: "创作方法论：提炼自《书名》。写小说/续写/设计情节冲突时使用。"
---

# <cm.name>
**来源**：《书名》（指导书提炼）| **生成**：YYYY-MM-DD

## 创作方法论（按步骤执行）
1. **步骤名**：instruction
   - 检查：checklist 项

## 技巧模式库
**技巧名**
- 何时用：...
- 怎么做：...

## 决策速查
- 当X时做Y，因为Z

## 反模式（务必避免）
- **what**：why
```

- 空段省略（无 patterns 不出【技巧模式库】段）
- name slug 化：小写、空白转 `-`、去非字母数字与 CJK 外字符（CJK 保留）
- 命令 `export_methodology_skill(id) -> Result<String, AppError>` 返回 Markdown 文本
- 前端 `MethodologyEditor` 加"导出 SKILL.md"按钮：调命令取文本 → tauri save 对话框（默认文件名 `<slug>.md`）→ `plugin-fs` 写文件

## 测试

- F1：fold-in 合并逻辑（dedup 状态分支带 merge_into、retry 沿用意图、CM 更新+step clamp）；`distill_foldin` 响应类型解析
- F2：清洗各规则（剔除/去重/截断/空输入）；硬校验失败路径
- F3：render_skill_md 各段渲染与空段省略、slug 化；命令端到端（内存库造 CM → 导出文本断言）

## 明确不做

- fold-in 合并时不保留双版本（无方法论版本历史）
- 校验器不加前端质量报告 UI
- SKILL.md 导出不含 chapters/ 分章文件（StoryForge 无章节级资产）
- 不导出到 agent skills 目录（用户自行保存放置）

## 涉及文件

| 文件 | 改动 | 功能 |
|---|---|---|
| `src-tauri/src/db/migrations/V126__guidebooks_merge_into.sql`（新） | merge_into_methodology_id 列 | F1 |
| `src-tauri/src/guidebook_distillation/models.rs` | Guidebook +字段、LlmDistillFoldinResponse | F1 |
| `src-tauri/src/guidebook_distillation/repository.rs` | 新列读写 | F1 |
| `resources/prompts/distillation/distill_foldin.md`（新） | 融合 prompt | F1 |
| `src-tauri/src/guidebook_distillation/distiller.rs` | distill 支持 foldin 模式 | F1 |
| `src-tauri/src/guidebook_distillation/service.rs` | upload_and_distill merge_into 参数、run_distillation fold-in 分支、step clamp、校验清洗接入 | F1/F2 |
| `src-tauri/src/guidebook_distillation/commands.rs` | upload 加参数、export 命令 | F1/F3 |
| `src-tauri/src/guidebook_distillation/validator.rs`（新） | 校验清洗 | F2 |
| `src-tauri/src/guidebook_distillation/skill_export.rs`（新） | SKILL.md 渲染 | F3 |
| `src-tauri/src/guidebook_distillation/mod.rs` | 注册新模块 | F2/F3 |
| `src-tauri/src/handlers.rs` | 注册 export 命令 | F3 |
| `src-frontend/src/components/guidebook-distillation/*` | 上传合并选择、卡片标注、导出按钮 | F1/F3 |
| `src-frontend/src/hooks/useGuidebookDistillation.ts` | upload 参数、export hook | F1/F3 |
| `src-frontend/src/types/guidebook-distillation.ts` | GuidebookListItem +merge 字段 | F1 |
