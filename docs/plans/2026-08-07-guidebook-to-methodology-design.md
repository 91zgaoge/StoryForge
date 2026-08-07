# 指导书 → 创作方法论资产 设计文档

日期：2026-08-07
状态：已批准（产物形态=方法论带步骤；格式=txt/pdf/epub；存储=自定义方法论表）

## 背景与目标

用户上传一本故事创作指导书（写作方法论书籍），应用用 LLM 自动提炼其核心内容，生成一条**带步骤的自定义创作方法论**资产，落库后与 5 种内置方法论（雪花/场景结构/英雄之旅/人物纵深/高密度世界观）**同等待遇**：可在故事设置与创建向导中选用、续写时自动注入约束、章节完成自动推进步骤、策略选择器可自动挑选。

## 关键现状（探索结论）

- **可整套复用的拆书流水线** `src-tauri/src/book_deconstruction/`：上传校验（txt/pdf/epub，100MB）→ SHA256 去重 → `parser.rs`（Txt/Pdf/EpubParser，识别中文章节标题）→ `chunker.rs` 分块策略 → `analyzer.rs` LLM 状态机（进度回调、check_cancel、500ms 进度事件）→ TaskService 任务（max_retries=3、心跳监控）→ 结果落库。前端 `BookDeconstruction.tsx` + `BookUploadPanel.tsx` + `useBookDeconstruction.ts`。
- **方法论无 DB 表**，三层结构：`domain/methodology.rs` 的 `MethodologyType` 枚举（5 个硬编码）+ `resources/prompts/methodology/methodology_{id}[_stepN].md` + `stories.methodology_id/methodology_step` 关联。
- **续写注入点**：`creative_engine/write_time_bundle.rs` 的 `resolve_methodology_extension`（约 960 行），按 `methodology_{id}_step{N}` → `methodology_{id}` 顺序解析，未知 id 仅 warn 跳过；用 `resolve_prompt_default`（只读内置 md，不读 DB）。
- **策略选择器可见性**：`strategy/asset_catalog.rs:21 methodology_assets()` 把枚举转成 SelectableAsset。
- **前端清单硬编码**：`MethodologySettings.tsx:7`、`NovelCreationWizard.tsx` 各有一份。
- **结论**：新增自定义方法论要触达全链路，必须新增注册机制并改 4 类接入点，不能只丢 md 文件。

## 总体设计

### 1. 新增模块 `src-tauri/src/guidebook_distillation/`

照搬 book_deconstruction 的骨架，语义独立：

- `models.rs`：`Guidebook` 记录、`DistillationStatus { Pending, Extracting, Distilling, Merging, Completed, Failed, Cancelled }`、`CustomMethodology`（含 steps）。
- `commands.rs`：`upload_guidebook`、`get_distillation_status`、`get_distillation_result`、`list_guidebooks`、`delete_guidebook`、`cancel_guidebook_distillation`、`list_custom_methodologies`、`update_custom_methodology`（改名/描述/步骤/启停）、`delete_custom_methodology`。
- `service.rs`：`upload_and_distill`：校验（复用同款扩展名/大小校验）→ SHA256 去重（`guidebooks.file_hash` UNIQUE）→ 复制到 `app_data_dir/guidebooks/{id}.{ext}` → `parser::parse_book`（**直接复用 book_deconstruction::parser**，不复制代码）→ 建记录 → 创建 `task_type="guidebook_distillation"` 任务（max_retries=3、心跳 600s），任务系统不可用时回退 spawn 直跑。
- `distiller.rs`：LLM 状态机：
  1. 元信息提炼（书名/作者/主题，→10%）
  2. 分块提炼（复用 chunker；每块提炼"核心原则/技巧/步骤要点"，→70%）
  3. 合并去重（LLM 汇总所有分块要点，→90%）
  4. 结构化方法论（生成 name/description/steps[{title, instruction, checklist[]}]/通用 prompt 扩展文本，输出 JSON，→100%）
  - 每步 `check_cancel()` + 心跳；500ms emit `guidebook-distillation-progress`；prompt 走 `resolve_prompt`（DB 覆盖优先），新增 `resources/prompts/distillation/distill_chunk.md`、`distill_merge.md`、`distill_methodology.md`、`distill_metadata.md`（**禁止硬编码 prompt 文本**）。
  - LLM JSON 解析失败：重试一次；仍失败 → Failed 并记录错误。
- 订阅门槛：沿用 Pro feature 检查，新增 feature key `guidebook_distillation`，默认与 `book_deconstruction` 同档开放。

### 2. 数据库（新 Migration）

- `guidebooks`：id / title / author / subject / file_hash(UNIQUE) / file_path / status / progress / error / created_at / updated_at。
- `custom_methodologies`：id / guidebook_id(FK) / name / description / steps_json(`[{title, instruction, checklist: []}]`) / enabled(INTEGER 默认 1) / created_at / updated_at。
- `stories.methodology_id` 继续复用：自定义方法论 id 用 `custom_{uuid}` 前缀，与内置 id 区分。

### 3. 创作链接入（自定义方法论 = 内置同等待遇）

- `write_time_bundle.rs resolve_methodology_extension`：内置 PromptRegistry 解析失败后，若 id 带 `custom_` 前缀 → 查 `custom_methodologies`，按 `story.methodology_step`（越界取最后一步）渲染该步骤的 `instruction + checklist` 为注入文本；渲染格式与内置 md 注入一致（`【创作方法论约束】` section 不变）。
- `domain/methodology.rs`：`methodology_max_steps` 对 `custom_` 前缀从 `steps_json.len()` 取值（新查询函数，需 DB 访问的调用点改为传入或查询）；`next_methodology_step` 逻辑不变（依赖 max_steps）。
- `strategy/asset_catalog.rs methodology_assets()`：追加 enabled 的自定义方法论（id/name/description→when_to_use）。
- `creative_engine/methodology/mod.rs list_available()`：合并内置 + 自定义，供前端与引擎统一使用；新增 `list_all_methodologies` Tauri 命令。
- **前端清单去硬编码**：`MethodologySettings.tsx`、`NovelCreationWizard.tsx` 改为 invoke `list_all_methodologies`，自定义项显示来源徽标（来自《书名》）。

### 4. 前端

- 拆书页面 `BookDeconstruction.tsx` 加 Tab："书籍拆解" / "指导书提炼"（新组件 `GuidebookDistillationPanel.tsx`），复用上传面板模式（同款 dialog 过滤 txt/pdf/epub）。
- 提炼结果视图：方法论名称/描述/步骤列表（可编辑）、启用开关、删除；监听 `guidebook-distillation-progress` + 任务事件 + 轮询兜底（照搬拆书三通道）。
- Hook：`useGuidebookDistillation.ts`。

### 5. 错误处理与边界

- 上传格式/大小非法：前后端双重校验，复用现有错误文案模式。
- file_hash 重复：返回已有记录（与拆书一致）。
- LLM 失败：任务系统重试 3 次；分块级失败记录到 error，整体 Failed 可重新触发。
- 取消：cancel flag + `check_cancel`，状态置 Cancelled。
- 提炼产物为空或步骤数为 0：置 Failed（不产生空方法论）。
- 删除 guidebook：级联提示关联方法论；删除方法论时若被 story 引用，提示并将 story 的 methodology_id 置空（或禁止删除——实现时取"置空+提示"）。

### 6. 测试

- Rust 单测：distiller JSON 解析（含坏输出重试）、custom_methodologies CRUD、resolve_methodology_extension 的 custom 分支、max_steps/steps 推进、asset_catalog 合并。
- 前端 vitest：上传面板校验、结果视图渲染、`list_all_methodologies` 合并清单。
- 回归基线：`cargo test --lib` 1179 项、前端 `tsc --noEmit` + `vitest run` 367 项，全绿方可提交。

## 不做的事（v1 边界）

- 不做"同时生成 prompt 模板"（用户已选仅方法论）。
- 不做增量/断点续提炼（分块结果不持久化，重试为整体重跑）。
- 不动内置 5 种方法论的现有行为。
- 不做跨表 file_hash 去重（guidebooks 与 reference_books 各自独立）。
