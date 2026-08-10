# 技能书提炼优化 + 去 Pro 设计文档

日期：2026-08-09
状态：已批准
灵感来源：[book-to-skill](https://github.com/virgiliojr94/book-to-skill)（提取结构而非摘要；分层资产；practitioner voice；保留作者原始命名）

## 背景与问题

技能书提炼（guidebook distillation）v0.33 落地后存在三类问题：

1. **Pro 门控**：`upload_guidebook` 要求 Pro 订阅（`guidebook_distillation/commands.rs:38`），用户要求免费放开。
2. **提炼质量浅**：分块提炼只产"一句话要点"（`distill_chunk` → `points[]`），合并去重后得 10-20 条原则，再压成 3-8 个步骤。书中大量可操作细节——具名技巧、决策规则、反模式——在"要点→原则→步骤"的两次压缩中丢失。这正是"方法论资产融合运用不够"痛点的一部分。
3. **失败不可恢复**：hash 去重只看记录存在不看状态（`service.rs:60-66`），提炼失败后重新上传同一文件会直接返回旧的 failed 记录 id；任务层 max_retries=3 实际无效（executor 返回 `Ok(success:false)` 直接 `ctx.fail()`）。

## 借鉴 book-to-skill 的核心思想

- **提取结构而非摘要**：具名框架（保留作者原始命名）、决策规则（"当X时做Y，因为Z"）、反模式（避免什么+为什么）
- **分层资产**：核心方法论 + 技巧模式库（patterns）+ 决策速查表（cheatsheet），按需注入
- **Practitioner voice**："Use X when Y"，不写"The book explains X"
- **Density over completeness**：不堆砌，每条都要可执行

## 设计

### Part A：去 Pro 限制

- **后端**：`subscription/mod.rs:78-97` free 白名单加入 `guidebook_distillation`。`commands.rs:38` 的既有门控保留但对免费用户放行（白名单命中即通过）。
- **前端**：`GuidebookDistillationPanel.tsx` 移除上传拦截（`:342-344`）、Pro 徽章（`:386-390`）、升级横幅（`:411-426`）、UpgradeModal（`:455-463`）；更新 `__tests__/GuidebookDistillationPanel.test.tsx`。
- **不碰**：拆书（`book_deconstruction`）、Pipeline 三命令、StyleBlend、内置方法论（`agents/service.rs:1960`）、personalizer（`:2205`）的既有 Pro 守卫。

### Part B：提炼质量升级

#### B1 分块提炼 prompt 升级（`resources/prompts/distillation/distill_chunk.md`）

每块输出结构化 JSON：

```json
{
  "key_points": ["一句话可执行要点"],
  "techniques": [{"name": "作者原始命名", "when_to_use": "何时用", "how": "怎么做"}],
  "decision_rules": ["当X时做Y，因为Z"],
  "anti_patterns": [{"what": "避免什么", "why": "为什么会失败"}]
}
```

要求：practitioner voice；保留作者对技巧/框架的原始命名；允许空数组；忽略序言/致谢。代码侧 `distiller.rs` 的 chunk 结果结构同步扩展，单块失败仍 warn 跳过。

#### B2 合并阶段升级（`distill_merge.md`）

输入全部块的四类产出（代码侧截断：200字/条、总量 12000字），分类聚合去重：

```json
{
  "principles": ["10-20条核心原则"],
  "techniques": [{"name": "...", "when_to_use": "...", "how": "..."}],
  "decision_rules": ["..."],
  "anti_patterns": [{"what": "...", "why": "..."}]
}
```

#### B3 产物扩展（V125 迁移）

`custom_methodologies` 表加两列：

- `patterns_json TEXT NOT NULL DEFAULT '[]'`——技巧模式库 `Vec<Technique{name, when_to_use, how}>`
- `cheatsheet_json TEXT NOT NULL DEFAULT '{}'`——决策速查 `{decision_rules: Vec<String>, anti_patterns: Vec<AntiPattern{what, why}>}`

`steps_json` 步骤化方法论（`distill_methodology`）保持不变，仍从 principles 生成。`models.rs` 加 `Technique`/`AntiPattern`/`Cheatsheet` 结构与 `parse_steps` 同款容错解析。

#### B4 续写注入增强（`render_custom_methodology_extension`，`service.rs:348`）

现有注入：当前 `methodology_step` 对应步骤的 instruction + checklist。
增强后追加：

- 按当前步骤序号从 patterns 中轮转选 2-3 条技巧（`step_index % patterns.len()` 起连续取），渲染为"【技巧参考】名称：何时用→怎么做"
- 从 cheatsheet 取最多 3 条决策规则 + 1 条反模式，渲染为"【决策速查】"
- 总量截断保护（与现有一致的字符预算内）

轮转而非 LLM 选取：零额外调用、确定性、可测试。

#### B5 失败可重试

- `upload_and_distill` hash 去重（`service.rs:60-66`）改为：仅当旧记录状态为 `completed` 或进行中（`pending`/`distilling`）时直接返回旧 id；`failed`/`cancelled` 时复用已存文件重新提炼（走 B5 的重试路径），不重复落文件。
- 新增 Tauri 命令 `retry_guidebook_distillation(guidebook_id)`：仅 failed/cancelled 状态可用，清 error、置回 pending、重跑 `run_distillation`（复用已存 file_path，走 TaskService，与 upload 同路径）。
- 前端失败/已取消卡片加"重试"按钮（`GuidebookCard`）。

### 前端编辑器

`MethodologyEditor` 增加只读/可编辑区展示 patterns 与 cheatsheet（跟随现有 steps 编辑模式），`update_custom_methodology` 命令支持更新新字段。

## 错误处理

- 分块失败：warn 跳过（现状保留）
- merge 产出为空：报错（现状保留）
- methodology 结构校验失败：重试一次（现状保留）
- 新字段解析失败：降级为空集合，不阻断提炼（与 `parse_steps` 容错一致）

## 测试

- `distiller.rs`：chunk/merge 新 JSON 结构解析（含坏 JSON 容错、空数组）
- `models.rs`：`Technique`/`AntiPattern`/`Cheatsheet` 序列化与容错解析
- `service.rs`：注入渲染含技巧/速查段；轮转选取确定性；重试路径（failed 可重试、completed 不可、hash 去重状态分支）
- 迁移 V125：`create_test_pool()` 全量迁移通过
- 前端：面板无 Pro 拦截/徽章/横幅；失败卡片重试按钮

## 明确不做（YAGNI）

- 增量合并（fold-in：新书并入已有方法论）
- 提炼结果校验器（validate_skill 类）
- SKILL.md 导出
- 章节级独立资产文件（book-to-skill 的 chapters/ 模式——StoryForge 的注入场景是续写 prompt，章节文件无加载入口）

## 涉及文件

| 文件 | 改动 |
|---|---|
| `src-tauri/src/subscription/mod.rs` | free 白名单 +`guidebook_distillation` |
| `src-tauri/src/db/migrations/V125__custom_methodology_assets.rs`（新） | patterns_json / cheatsheet_json 列 |
| `src-tauri/src/db/migrations/mod.rs` | 注册 V125 |
| `src-tauri/src/guidebook_distillation/models.rs` | Technique/AntiPattern/Cheatsheet + 容错解析 |
| `src-tauri/src/guidebook_distillation/distiller.rs` | chunk/merge 结构化产出解析 |
| `resources/prompts/distillation/distill_chunk.md` | 结构化提炼 prompt |
| `resources/prompts/distillation/distill_merge.md` | 分类聚合 prompt |
| `src-tauri/src/guidebook_distillation/service.rs` | 落库新字段、注入增强、hash 去重状态分支、retry |
| `src-tauri/src/guidebook_distillation/commands.rs` | `retry_guidebook_distillation` + update 命令新字段 |
| `src-tauri/src/guidebook_distillation/repository.rs` | 新字段读写、状态查询 |
| `src-tauri/src/lib.rs` | 注册 retry 命令 |
| `src-frontend/src/components/guidebook-distillation/*` | 去 Pro UI、重试按钮、编辑器新字段 |
| `src-frontend/src/hooks/useGuidebookDistillation.ts` | retry hook |
| `src-frontend/src/services/api/*` | retry + update 新参数 |
