# 智能创作资产融合深度重构设计

日期：2026-08-04
状态：已批准（合并一次性交付；空转 Genesis 提示词族删除）
范围：续写链路 + 计划结构 + 创世链路全量重构

## 背景与问题

用户反馈：智能创作流程对已有创作资产（创作方法论、写作策略、提示词）融合运用不足，导致续写产出人物角色少、场景单一、世界构建简单、故事冲突不足、故事线推进慢。

三路只读审计（主流程 / 资产盘点 / writer prompt 组装）交叉验证后，根因归为四类断裂：

### A. 收敛导向的 prompt 体系（→ 人物少、场景单一）

- writer prompt 明文「禁止自创新角色」（`write_time_bundle.rs:600`）、「禁止另起开篇」（`agents/orchestrator.rs:3645`），大纲规划层同样禁止新角色（`resources/prompts/planner/scene_outline.md:20`）；全链路没有任何「引入新角色/切换场景/升级冲突」的扩张性指令。
- 每次续写自我限定「写一段正文（800-1500字）」（`resources/prompts/writer/orchestrator_timesliced_writer.md:13`），max_tokens 4096 的预算被文案锁死。

### B. 冲突引擎资产死注入（→ 冲突不足、推进慢）

- 追读力债务、钩子类型、活跃冲突、微兑现等参数每轮注入 `enriched_params`（`planner/executor.rs:1252-1343`），但只有 Full 路径消费，默认 TimeSliced 路径完全忽略。
- 冲突强度在 TimeSliced 退化为「冲突强度：0.5」裸数字（`write_time_bundle.rs:933-940`）；Full 路径的分档语义文案（`agents/service.rs:1888-1899`）到不了默认路径。

### C. 推荐型资产传递断裂（→ 方法论/策略融合不足）

- 体裁画像推荐的方法论/风格 DNA/技能 ID 只写进内存 strategy，不进 writer、不落库（`commands/orchestrator.rs:1284` vs `planner/executor.rs:1221-1250`）。
- 向导中 LLM 选出的桥段卡/剧情引擎/高压关系无持久化字段，落库即丢（`applyWizardToStory.ts:55-57`）。
- 方法论仅 5 个硬编码 ID（`domain/methodology.rs:12-18`、`write_time_bundle.rs:211-238`），未知 ID 静默丢弃；`methodology_step` 恒为 1 永不推进；风格混合 blend 永不到达 writer。

### D. 计划结构丢弃资产规划（→ 资产融合流于表面）

- 续写几乎全部塌缩为单 writer 步（`planner/mod.rs:274-288`），planner prompt 中的世界观/角色/伏笔上下文与 LLM 的规划产出被整体丢弃，方法论/策略不影响计划结构。

另发现：7 个 Genesis 生成族提示词（`narrative_*_generate` 等）整体空转无生产调用方；11 个死注册提示词前端可编辑但无人读取。

## 总体决策

- 实施尺度：深度重构，合并一次性交付（不分期）。
- 空转 Genesis 生成族 + 11 个死注册提示词：删除（向导继续使用 `novel_creation_*` 提示词族，避免双套平行体系）。
- 所有 prompt 改动走 PromptRegistry（`resources/prompts/**/*.md`），前端「提示词」面板可编辑可覆盖。

## 详细设计

### 一、续写链路资产贯通（TimeSliced 补齐死注入）

`WriteTimeBundle`（`creative_engine/write_time_bundle.rs`）新增字段与 `to_prompt()` 段落，组装逻辑全部从 Full 路径（`agents/service.rs`）下沉为共享函数复用，不新造轮子：

| 新注入 | 来源/复用 | 预算 |
|---|---|---|
| 活跃冲突清单 | CanonicalState 快照（`service.rs:2345-2357` 下沉共享） | ~600 字 |
| 角色目标/弧光/秘密 | 角色卡扩展（`service.rs:2386-2411` 同款） | 每角色 ~200 字 |
| 追读力债务/钩子/微兑现 | 打通 `executor.rs:1252-1343` 死注入，渲染复用 `writer_chase_debt` 模板 | — |
| 体裁元素参考表+典型结构 | `reference_tables_json`/`typical_structure_json`（`service.rs:1966-1971` 下沉） | ~800 字 |
| 风格混合 blend | `commands/orchestrator.rs:612-651` 已拼好的 blend 文本经 writer 参数透传；优先 blend、回退单 DNA | — |

配套：

- 冲突约束语义化：`format_writing_strategy_constraints`（`write_time_bundle.rs:933-940`）改用 Full 分档文案（`service.rs:1888-1899` 提取共享）。
- 推进锚点去重：`build_progression_anchor`（`agents/orchestrator.rs:3784-3831`）与 bundle 重复注入的大纲/世界观合并，省出预算给新段落。

### 二、推荐资产贯通 + 方法论动态化

- `planner/executor.rs:1221-1250` 扩展：体裁画像推荐的 methodology_id / style_dna_ids / skill_ids 作为 writer 参数透传；bundle 优先推荐值、story 字段回退；story 无显式方法论时推荐值写回落库（有显式值不覆盖）。
- 方法论解析动态化：`write_time_bundle.rs:206-250` 的硬编码 match 改为 PromptRegistry 动态解析——先试 `methodology_{id}_step{N}`，再试 `methodology_{id}`；未知 ID 记 warn（不再静默 None）。新增方法论 = 丢一个 md 文件进 `resources/prompts/methodology/`，前端面板即可编辑。
- `methodology_step` 自动推进：每完成一章 step+1，到该方法论最大步数停留；`MethodologySettings` 手动步进保留。

### 三、扩张性写作合约（prompt 文案）

- `writer_system.md` / `orchestrator_timesliced_writer.md` 增加阶段感知的扩张-收敛平衡准则，按 story_progress 分档：
  - 开篇/发展期：鼓励引入有叙事功能的新角色、允许场景切换、推动冲突升级；
  - 高潮期：聚焦既有冲突爆发；
  - 收尾期：收敛收束。
  - 底线保留：新角色必须有叙事功能、不与 MASTER_SETTING 冲突。
- `write_time_bundle.rs:600`「禁止自创新角色」改为同款阶段感知文案。
- 字数上限改为配置项 `continuation_target_words`（默认 2000 → 1400-2600），模板动态渲染。

### 四、计划结构重构（beat 驱动多步计划）

- 续写默认计划改为：`beat_planner` → `writer`（depends_on beat_planner）→ 可选 `mini_review` 轻质检。`sanitize_plan_for_prose_request`（`planner/mod.rs:274-288`）适配为保留该链而非塌缩单步。
- 新增 `beat_planner` capability + 新提示词 `writer_beat_plan.md`：输入 PlanContext 资产摘要 + 当前方法论 step + 策略节拍卡，输出 ≤300 字 JSON（戏剧目标/冲突升级点/引入新元素/伏笔操作/目标字数），注入 writer step 参数；planner 的 understanding 不再丢弃。
- beat_planner 单次 LLM（max_tokens ~600、60s 超时），失败/超时自动降级回单 writer 路径。
- AppConfig 新增 `plan_mode: beat（默认）| single_writer` 回退开关。

### 五、创世链路修复 + 数据迁移

- `novel_creation_*` 向导提示词（`agents/novel_creation.rs`）注入体裁画像内容（core_tone/反模式/典型结构）、方法论扩展、四元组推荐（现状只拼自由文本 genre）。
- **V119 迁移**：`stories` 表新增 `strategy_json TEXT`；`applyWizardToStory` 落库向导选中的 beat_card_ids / story_engine_ids / pressure_relationship_id / emotional_payoff / conflict_arena；`build_selected_strategy` 优先读持久化值，缺失字段再走启发式推断。旧数据 NULL 时行为与现状一致。
- 删除 7 个 Genesis 生成族提示词（`narrative_story_concept_generate`、`narrative_world_building_generate`、`narrative_character_generate`、`narrative_scene_generate`、`narrative_foreshadowing_generate`、`narrative_story_arc_generate`、`narrative_first_chapter_generate`）与 11 个死注册提示词（`commentator_paragraph`、`deconstruction_story_arc`、`memory_knowledge_generation`、`methodology_character_analysis`、`methodology_scene_self_check`、`narrative_first_scene_generate`、`narrative_genre_profile_generate`、`narrative_opening_skeleton`、`narrative_outline_extract`、`strategy_reference_book_context`、`writer_reference_scene_fewshots`）及其 md 文件与 Registry 引用；`narrative/prompts.rs` 的 Generate 模式代码同步清理。

### 六、错误处理

- 新注入段落全部「有则注入、无则跳过记 debug」，资产缺失不导致失败。
- beat_planner 失败/超时降级单 writer 路径。
- PromptRegistry 解析失败回退内置 md（现有机制）。
- AppConfig 新字段缺省值保持现状兼容。

### 七、测试策略

- `write_time_bundle` 新段落：注入断言单测（有资产进 prompt / 无资产不崩）。
- 方法论动态解析：注册临时 prompt id 解析测试 + 未知 ID warn 测试。
- 步进推进：章节完成触发 step+1 单测。
- beat 计划链：sanitize 后保留 beat→writer 链单测；beat_planner 失败降级路径测试。
- `strategy_json`：落库/读取回环测试；旧 NULL 数据回退启发式测试。
- 前端：`applyWizardToStory` 持久化测试、字数配置传递。
- 回归门槛：`cargo test` 全绿 + `npx tsc --noEmit` 通过。

## 非目标（YAGNI）

- 不改创世 2.0 agency 多代理流程（is_new_novel 路径）。
- 不接回向导使用 Genesis 族提示词（已决策删除）。
- 不恢复 beat_cards/story_engines/pressure_relationships 数据库表（V094 已 DROP，内容保持 Rust 内置常量 + strategy_json 存 ID）。
- 不动模型网关与 LLM 适配层。

## 实施期偏差记录（v0.31.0 收尾时回写）

- **Task 9：删除 beat_planner 内联硬编码 prompt 兜底（用户裁决）**：计划原允许解析失败时内联兜底 prompt，实施中经用户裁决删除该兜底，解析失败统一走降级输出（单 writer 路径）。
- **Task 11：serde legacy 解析回归修复（计划外）**：V119 `strategy_json` 的 serde(default) 导致 legacy 数据解析回归，特征键识别恢复至 v0.26.28 口径，评审确认正确。
- **Task 7/9：新测试并入既有 `mod tests`**：因 Rust 同名片强制约束，新测试未独立成文件，并入既有测试模块，可接受。
- **Task 12：CONTEXT.md 活文档顺手修正**：随提示词清理同步修正项目 CONTEXT.md 中的过时描述，评审认定合理。
- **final-review 收尾修正**：§三/CHANGELOG 默认字数文案原写「1200-2500」，实际实现为 `continuation_target_words=2000 × 0.7-1.3 = 1400-2600`，已统一修正为 1400-2600。
