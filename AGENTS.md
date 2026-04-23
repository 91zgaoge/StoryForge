# StoryForge Agent 指南

> 本文件包含 AI 助手需要了解的项目背景、编码风格和工具配置

## 🧠 永久记忆：自动化测试助手

本项目已配置 **Playwright + Chromium** 无头浏览器自动化测试环境，专为 AI 助手设计。

### 快速启动测试

```bash
# 一键截图所有页面
npm run screenshot

# 截图幕前界面
npm run screenshot:front

# 截图幕后界面
npm run screenshot:back

# 运行完整测试
npm test
```

### 测试助手 API

文件位置：`e2e/test-helper.ts`

```typescript
import { runTest } from './e2e/test-helper';

runTest(async (helper) => {
  // 导航
  await helper.navigate('http://localhost:5173');
  
  // 截图
  await helper.screenshot('homepage');
  
  // 交互
  await helper.click('button');
  await helper.type('input[name="title"]', '测试标题');
  await helper.press('Enter');
  
  // 等待
  await helper.waitFor('.success-message');
  await helper.sleep(1000);
  
  // 执行 JS
  const title = await helper.eval<string>('document.title');
});
```

### 已配置的测试环境

| 组件 | 版本 | 路径 |
|------|------|------|
| Playwright | latest | `e2e/` |
| Chromium | 147.0.7727.15 | `C:\Users\admin\AppData\Local\ms-playwright\chromium-1217` |
| bunwv | 0.0.5 | 全局安装 (备用) |

### 测试文件位置

- 测试代码：`e2e/*.spec.ts`
- 测试截图：`e2e/screenshots/`
- 测试报告：`playwright-report/`
- 配置：`playwright.config.ts`

---

## 📋 项目背景

**StoryForge (草苔)** - AI 辅助小说创作桌面应用

- **版本**: v4.1.0
- **GitHub**: https://github.com/91zgaoge/StoryForge
- **技术栈**: Tauri 2.4 + Rust 1.94 + React 18 + TypeScript 5.8 + SQLite + Vitest

### 双界面架构

| 界面 | 用途 | URL |
|------|------|-----|
| 幕前 (Frontstage) | 沉浸式写作 | `/frontstage.html` |
| 幕后 (Backstage) | 工作室管理 | `/index.html` |

---

## 🎨 编码风格

### Rust 后端

- 使用 `snake_case` 命名
- 错误处理使用 `Result<T, E>`
- 异步函数使用 `async/await`
- 数据库使用 `rusqlite` + `r2d2` 连接池

### TypeScript 前端

- 使用 `camelCase` 命名
- 组件使用函数式组件 + Hooks
- 状态管理使用 Zustand
- API 调用使用 TanStack Query

### 提交信息格式

```
<type>: <subject>

<body>

type:
  feat: 新功能
  fix: 修复
  docs: 文档
  style: 格式
  refactor: 重构
  test: 测试
  chore: 构建
```

---

## 🔧 开发命令

```bash
# 启动前端开发服务器
cd src-frontend && npm run dev

# 启动 Tauri 应用
cd src-tauri && cargo tauri dev

# 构建生产版本
cd src-tauri && cargo tauri build

# 运行测试
npm test
```

---

## 📚 重要文档

- [ARCHITECTURE.md](./ARCHITECTURE.md) - 架构设计
- [TESTING.md](./TESTING.md) - 测试文档
- [CHANGELOG.md](./CHANGELOG.md) - 更新日志
- [ROADMAP.md](./ROADMAP.md) - 开发路线

---

### 最近完成的功能

- **v4.1.0 幕前界面深度重构：化整为零，萤火随行** (2026-04-22) — P0+P1+P2 全流程体验重构
  - **设计理念**: 从 20+ 可见 UI 元素缩减至 <5 持久元素。AI 功能以萤火暗示（firefly hints）形式按需浮现，用完即隐。"创作者不应在工具中花费精力标注自己的创作"——移除所有显式注释/评论创建 UI。
  - **P0 核心重构 (4 项)**:
    - 顶栏精简: 44px 细线设计，小说标题（点击进入幕后）、字数统计、字号调节、🔥 文思三态切换（off·/passive✨/active🔥）、禅模式。移除汉堡菜单、订阅徽章、"开启文思"按钮、"AI 续写"按钮、主行动按钮。
    - 底栏删除: 彻底删除底部聊天工具栏（chat input、模型状态点、WenSiPanel 嵌入、Slash textarea 菜单）。AI 结果以幽灵文本（ghost text）内联呈现，Tab 接受/Esc 拒绝。
    - 侧边栏精简: 5 按钮→2 按钮：修（修订模式）/ 批（生成古典评点）/ 幕（幕后）。移除注释和评论显式 UI。
    - 键盘快捷键: `Ctrl+Enter` / `Cmd+Enter` 全局触发续写，`Ctrl+Space` 循环文思模式，`F11` 禅模式。
  - **P1 萤火系统 (3 项)**:
    - 幽灵文本: 编辑器末尾灰色斜体段落（`opacity: 0.35`），附带萤火操作栏（Tab 接受 / Esc 拒绝）。
    - 右边缘萤火: `smartGhostText` 从右侧淡入（0.8s）→ 停留 → 淡出（1.2s），不打扰写作流。
    - 空态引导: 编辑器无内容时居中显示诗意提示"开始写下第一句话，文思将随你而行"。
  - **P2 体验优化 (4 项)**:
    - 内联 `/` 命令菜单: 8 命令（续写/润色/古风/场景/自动续写/审校/评点/排版），光标处触发，方向键导航，回车执行，Esc 关闭，自动删除 `/` 字符。
    - WenSiPanel 浮动化: 从底栏嵌入改为 FrontstageApp 右下角浮动卡片，通过 `/` 菜单高级命令触发。
    - 修订横幅精简: 从多行可展开缩减为 32px 单行，变更列表可滚动，默认折叠。
    - 古典评点保留: AI 生成的段落评点（金圣叹式朱批）保留为内联段落，朱红色 `oklch(55% 0.18 25)`，霞鹜文楷字体，左边框红色，※ 前缀，缩进 3em。
  - **移除（设计决策）**:
    - 显式注释系统: sidebar "注"按钮、注释/评论面板、选中文本弹窗创建按钮、右键菜单项、所有相关 hooks（`useTextAnnotations`、`useCommentThreads`）。
    - 原因: AI 写作工具不需要创作者标注自己的作品；AI 反馈应以幽灵文本或古典评点形式自然呈现。
  - 编译: `cargo check` 零错误零警告，`cargo test` 160/160，`npm run build` 通过

- **v4.0.1 全面代码审计与空实现修复** (2026-04-22) — Phase A+B
  - **Phase A: 代码审计与 P0 修复 (15+ 项)**:
    - 综合审计: 扫描 40+ 模块，输出 `CODE_AUDIT_REPORT_V4.md`（5 严重/17 参数/9 空实现）
    - IPC: 统一 17 处 camelCase→snake_case 参数名，修复 Tauri v2 反序列化静默失败
    - 空实现补全: `analytics` 真实统计、`agents/commands` 真实状态、`skills/executor` 真实 MCP 调用、`export/import_from_text` 正则解析、`workflow/scheduler` 执行日志、`evolution/updater` manifest CRUD、`mcp/server` 缺失 `.await`
    - 前端修复: `settings.ts` 移除硬编码密钥、`useCollaboration.ts` WebSocket 真实发送、`useStreamingGeneration.ts` 移除 mock、`textAnalyzer.ts` 增量分析
    - UI: 聊天工具栏从 absolute 改为正常流、编辑器 padding 优化
    - 类型统一: `skills/mod.rs` 移除重复 `McpServerConfig`
  - **Phase B: 内存模块 SQLite 持久化 (3 模块)**:
    - Migration 26/27/28: `chat_sessions`/`chat_messages`、`story_runtime_states`、`collab_sessions`/`collab_participants`
    - `chat/mod.rs`: `ChatManager` 改为 `DbPool` 持久化
    - `state/manager.rs`: `StoryStateManager` 改为 `DbPool` 持久化
    - `collab/mod.rs` + `websocket.rs`: `CollabManager` 持久化 + 完整消息处理闭环（Join/Leave/Operation/Cursor/Participants）
  - 编译: `cargo check` 零错误零警告，`cargo test` 160/160，`npm run build` 通过

- **v4.0.0 借鉴 AI-Novel-Writing-Assistant 全面优化** (2026-04-22) — Phase 1+2+3 共 9 项新功能
  - **Phase 1: P0 核心能力 (3 项)**:
    - Canonical State: 新增规范状态系统，统一聚合 StoryContextBuilder/character_states/foreshadowing/KG 等分散状态，AI 续写时准确知道"当前处于故事哪个阶段"
    - Payoff Ledger: 升级 ForeshadowingTracker 为伏笔账本，新增时间窗口追踪(target_start/target_end)、逾期检测、风险信号、回收时机智能推荐
    - Execution Panel: 新增章节执行面板，智能推荐下一步行动（"处理逾期伏笔"/"续写"/"运行审校"），集成到 Scenes.tsx 和 FrontstageApp
  - **Phase 2: P1 质量与控制 (3 项)**:
    - Narrative Phase Detection: 增强叙事阶段检测（逾期伏笔→ConflictActive、高置信度长内容→Climax、主要伏笔回收→Resolution），注入 Writer prompt
    - Structured Outline: Scene 模型新增 execution_stage/outline_content/draft_content，SceneEditor 重写为 6 标签页（规划/大纲/起草/审校/定稿/批注）
    - Audit System: 新增统一审计模块，整合 ContinuityEngine/StyleChecker/QualityChecker/PayoffLedger，五维评分（连续性/人物/风格/节奏/伏笔），支持 light/full 审计
  - **Phase 3: P2 体验优化 (3 项)**:
    - Novel Creation Wizard: 新增 5 步小说创建向导（创意→世界观→角色→文风→首个场景），每步提供 AI 生成选项
    - Enhanced Streaming: StreamOutput 组件增强（Markdown 渲染、实时字数、停止按钮、打字机效果），接入 FrontstageApp/WenSiPanel/CreationWizard
    - Strategy Configuration: Settings 新增写作策略配置（运行模式/冲突强度/叙事节奏/AI 自由度），动态注入 Writer prompt
  - 编译: `cargo check` 零错误，`cargo test` 160/160，`npm run build` 通过

- **v3.7.1 智能化创作系统 5 阶段重构深度修复** (2026-04-22) — Phase A+B+C 共 15 项修复
  - **Phase A: P0 核心断裂修复 (5 项)**:
    - QueryPipeline: `graph_expansion` 内容分词后逐 token 匹配实体，修复图谱扩展永不命中的 bug
    - QueryPipeline: `budget_control` 修复内层 break 只跳出内层循环的预算泄漏 bug
    - ContinuityEngine: `check_world_rules` 修复检查方向——从"检测规则描述片段"改为"提取禁止条款后检测"
    - ContinuityEngine: `get_character_states` 效率优化（O(N×M)→O(N+M)），`check_character_locations` 增强跨场景位置检测
    - PreferenceMiner: `record_feedback` 成功后异步触发 `mine_preferences`，自适应学习闭环激活
    - StyleChecker: 接入 `AgentOrchestrator` 闭环，Writer→Inspector→StyleChecker→Writer 风格校验生效
    - Ingestion: 实现真正的内容保存（Chapter 创建/更新）+ 简化知识图谱实体提取，工作流闭环完成
  - **Phase B: P1 功能补全 (6 项)**:
    - 方法论: Migration 22 添加 `methodology_id`/`methodology_step`，Settings 页面新增创作方法论配置
    - 创作模式: `CreationWorkflowEngine` 按 `CreationMode` 分支（AI全自动/AI初稿+精修/人工初稿+润色）
    - 进度反馈: 前端 `useWorkflowProgress` Hook + Stories.tsx 进度弹窗（阶段名称+百分比+指示器）
    - Orchestrator 事件: 前端监听 `orchestrator-step` 实时状态（生成→质检→改写），Settings 暴露阈值/循环数配置
    - AdaptiveGenerator: `calculate_temperature` 累加而非覆盖，pacing/style 偏好微调生效
    - 反馈记录: AiSuggestionNode + WenSiPanel 接入 `record_feedback`，覆盖内联建议/自动续写/自动修改
  - **Phase C: P2 优化 (4 项)**:
    - StyleAnalyzer: 新增 `analyze_with_llm` + `analyze_style_sample` IPC，Stories.tsx 新增"从文本生成风格"
    - QualityChecker: 新增 `check_with_llm`，Review 阶段优先 LLM 评估、回退规则评估
    - PhaseWorkflow: 硬编码阶段逻辑迁移到配置驱动，`PhaseWorkflow` 配置系统激活
    - 增量 Context: 每阶段完成后关键产出回注 `AgentContext`（Conception→world_rules, Outlining→scene_structure）
  - 编译: `cargo check` 零错误，`cargo test` 145/145，`npm run build` 通过

- **v3.6.1 全面功能审计与深度修复** (2026-04-22) — P0+P1+P2 共 30 项修复
  - **P0 紧急修复 (10 项)**:
    - DB: Migration 21 补全 scenes/kg_relations `confidence_score` 缺失列，消除运行时崩溃
    - IPC: 统一 25 处 camelCase→snake_case 参数名，修复 Tauri v2 反序列化失败
    - 场景: `create_scene` 后端扩展参数，前端传参不再静默丢弃
    - Orchestrator: 修复 Rewrite 事件错误携带初稿分数的 bug (`writer_result.score` → `rewrite_result.score`)
    - 技能: `execute_skill` 注入真实 `StoryContext`，`SkillExecutor` 实现真正 LLM 调用
    - 自适应学习: FrontstageApp accept/reject 接入 `record_feedback`，FeedbackRecorder 数据源激活
    - 审计: `LlmService::generate` 完成后调用 `log_ai_usage`，AI 调用日志写入数据库
    - 配额: auto_write/auto_revise 错误处理识别配额关键字，触发 Toast 提示
  - **P1 功能补全 (8 项)**:
    - ContinuityEngine: 补全 timeline + character_emotion + relationship 检查，5/5 全部实现
    - 一键创作: `CreationWorkflowEngine` 每阶段发射 `workflow-progress` 事件 + QualityReport 填充
    - SceneRepository: 新增 5 个单元测试（create/get/update/delete/reorder），Rust 测试 139→144
    - hooks/index.ts: 补全 `useCommentThreads` 等 6 个 Hook 导出
    - 类型: `ChangeTrack.scene_id` 改为 `string | undefined`，与后端 `Option<String>` 对齐
    - 评论: RichTextEditor 已解决评论支持「重新打开」
    - 变更追踪: 修订模式增加单条 change 独立接受/拒绝按钮
    - 清理: 移除弃用 `check_ai_quota` IPC 注册
  - **P2 优化 (6 项)**:
    - 概念统一: Sidebar `chapter_count` 显示从"场景"改为"章"
    - 滑块: SceneEditor 置信度 `step` 从 0.05 改为 0.1
    - 拆书转故事: 人物 background 合并 personality + appearance，场景 summary 保存为 content
    - 伏笔看板: 幕后新增 Foreshadowing 页面，支持 setup/payoff/abandoned 状态管理
    - 技能 Hook: 6 个关键业务点（create_chapter/character/scene、AI write、world_building update）激活 Hook 调用
    - 孤儿表: 评估 `world_rules`/`settings`/`character_states`，保留兼容
  - 编译: `cargo check` 零错误，`cargo test` 144/144，`npm run build` 通过

- **v3.5.2 全功能落地：剩余 7 项修复完成** (2026-04-22)
  - #17 auto_revise 取消/进度事件：后台任务模式 + 4 阶段进度 + 取消支持
  - #20 confidence_score：Scene 类型补全 + SceneEditor 置信度滑块
  - #16 MCP 持久连接：全局连接池 + disconnect/get_connections + DuckDuckGo 真实搜索
  - #19 一键创作按钮：Stories 页面入口 + run_creation_workflow 调用
  - #18 StyleDNA UI：stories 表 style_dna_id + 前端选择模态框 + 创作注入
  - #15 技能系统补全：execute_skill 异步 LLM 调用 + 2 个缺失技能（角色声音/情感节奏）
  - #14 意图引擎接入：RichTextEditor 聊天栏 parseIntent → 路由 → executeIntent
  - 139 Rust tests + 前端构建全部通过，版本号统一 3.5.2

- **v3.5.1 全面功能审计与修复** (2026-04-22) — 13 项关键修复
  - 自动修改: 结果应用到编辑器 + 保存到数据库
  - 拆书: 书名/作者持久化、convert_to_story story_id 修复、store_embeddings、进度 100%、心跳闪烁修复
  - 场景模型: scene_versions 表生产环境补建、conflict_type 列索引修复、版本快照全字段检测
  - AI 核心: AgentOrchestrator 闭环集成、ContinuityEngine/ForeshadowingTracker 写作流集成、AdaptiveGenerator 动态参数应用、auto_write Ingest 触发
  - Inspector: JSON 结构化输出 + 三层解析增强
  - LLM: 取消机制实现、useLlmStream 真实流式
  - StyleDNA: 内置风格自动种子化、CreationWorkflowEngine 暴露命令
  - 测试: Rust 139 全部通过，前端构建通过，已推送 GitHub

- **v3.5.0 拆书体验升级** (2026-04-21) — 进度提示 + 取消支持
  - 后端: `BookAnalyzer` 5 步 Pipeline 每个子步骤发送详细进度，人物/章节逐块汇报
  - 前端: `AnalysisProgress` 8 步骤指示器 + 百分比 + 块处理信息，告别"只见转圈"
  - 取消: `TaskExecutionContext.is_cancelled()` + analyzer 循环检查 + `cancel_book_analysis` IPC
  - 数据库: `reference_books` 新增 `task_id` 字段 + Migration 18
  - 测试: Rust 139 全部通过，前端构建通过

- **v3.4.0 智能化创作系统** (2026-04-18) — 5 阶段重构
  - Phase 1 地基重构: `StoryContextBuilder` 真实 DB 上下文, `QueryPipeline` 四阶段检索, `ContinuityEngine`, `ForeshadowingTracker` — 27 tests ✅
  - Phase 2 方法论注入: 雪花法/场景节拍/英雄之旅/人物深度 + `MethodologyEngine` + `AgentOrchestrator`(Writer→Inspector 闭环) — 34 tests ✅
  - Phase 3 风格深度化: `StyleDNA` 六维模型, `StyleAnalyzer`, `StyleChecker`, 10 经典作家 DNA, `StyleDnaRepository` — 45 tests ✅
  - Phase 4 自适应学习: `FeedbackRecorder`, `PreferenceMiner`(5 维启发式), `AdaptiveGenerator`(动态 temperature/top-p), `PromptPersonalizer` — 54 tests ✅
  - Phase 5 工作流闭环: `CreationWorkflowEngine`(7 阶段), `QualityChecker`(4 维评估) — 63 tests ✅
  - 版本号统一 3.3.0→3.4.0，Logo 生成全平台图标包

- **Freemium 付费系统** (2026-04-18)
  - 后端: `subscriptions`/`ai_usage_quota`/`ai_usage_logs` 表 + `SubscriptionService` + Tauri IPC 命令
  - 前端: `useSubscription` Hook + `SubscriptionStatus` 指示器 + `UpgradePanel` 付费引导 + 配额用尽提示
  - 策略: "分析免费，修改收费" — 免费用户看提示，Pro 用户享内联改写 + 风格 DNA + 方法论
  - Agent 分层: 免费版 max_tokens 1000 + 简化 prompt；专业版完整能力
  - 优化: 原子扣减 / 成功后扣费 / session 冷却 / 离线缓存 / 防抖修复 — 9 项

- **幕前排版与 AI 续写优化** (2026-04-17)
  - 段落间距收紧 + 首行缩进 2em，底部栏 padding-bottom 增至 10rem
  - 自动续写：接受 AI 生成后自动触发下一轮续写
  - Zen 模式绝对纯净：隐藏所有 AI UI 元素

- **TaskService 全局共享修复 + 集成测试建设** (2026-04-19)
  - 关键 Bug: `TaskService` 未全局共享 → 每个 command 新建实例 → `BookDeconstructionExecutor` 丢失 → 拆书功能不可用
  - 修复: `TaskService<R: Runtime>` 泛型化 + 手动 `Clone` + `app.manage(task_service)` + `State<'_, TaskService>`
  - 缓存修复: `useSetActiveModel` `invalidateQueries({ queryKey: ['settings'] })`
  - 单元测试: Rust 71 新增（settings 16 + task_system 13 + repositories 14 + validation 20）+ 前端 21 新增
  - 集成测试: Rust 5 新增（executor registry 共享、任务生命周期、调度器、无执行器失败、拆书去重）
  - 测试总计: Rust 139 + 前端 21 = 160 tests 全部通过

- **拆书功能 + 任务系统 + 向量化存储** (2026-04-19)
  - 后端: `book_deconstruction` 模块 — parser/chunker/analyzer/repository/service/commands
  - 前端: `BookDeconstruction` 页面 + 6 个子组件 + `useBookDeconstruction` Hooks
  - 任务系统: `task_system` 模块 — models/repository/scheduler/heartbeat/executor/service/commands (8 IPC 命令)
  - 拆书改为 `BookDeconstructionExecutor` 任务执行，心跳保活 + 进度推送
  - 向量化: 场景/人物 embedding 自动生成并入库 LanceVectorStore
  - 数据库: 5 张新表 (tasks + task_logs + reference_books + reference_characters + reference_scenes) + 9 个索引 + Migration 16/17

- **拆书功能** (2026-04-19)
  - 后端: `book_deconstruction` 模块 — parser/chunker/analyzer/repository/service/commands
  - 前端: `BookDeconstruction` 页面 + 6 个子组件 + `useBookDeconstruction` Hooks
  - 支持 txt/pdf/epub 解析，三层 LLM 分块分析策略，生成小说类型/世界观/人物/章节/故事线
  - 一键转为故事项目，参考素材库独立存储，向量化接口预留
  - 新增 3 张数据库表 + 4 个索引 + Migration 16，6 个单元测试

- **任务系统 + 拆书改任务 + 向量化存储** (2026-04-19)
  - 后端: `task_system` 模块 — models/repository/scheduler/heartbeat/executor/service/commands (8 IPC 命令)
  - 前端: `Tasks` 页面 + `useTasks` Hooks，状态分组/心跳指示器/进度条/执行日志
  - tokio::time 调度器支持 once/daily/weekly/cron，每任务互斥锁防重叠，心跳检测60秒扫描
  - 拆书分析改为 `BookDeconstructionExecutor` 任务执行，每步分析后心跳保活
  - 向量化存储接入 LanceVectorStore：场景/人物 embedding 自动生成并入库
  - 新增 2 张数据库表 (tasks + task_logs) + 5 个索引 + Migration 17

### 编译状态

- `cargo check` ✅ | 警告: 0
- `cargo check --release` ✅ | 警告: 0
- `npm run build` ✅
- `cargo test` ✅ 160/160

---

### 🏗️ 永久构建规则（用户强制要求）

> **每次修改代码后，先推送到 GitHub，触发 GitHub Actions 全平台构建。**
> **推送完成后，在本地执行构建并打包生成 Windows `.exe` / `.msi` 安装包。**
> **每次推送到 GitHub，都必须逐条更新 GitHub 项目的 `README.md` 文件内容。包括但不限于：功能列表、版本号、截图、应用图标、安装说明、使用指南等所有相关信息。**
> **Git tag、Cargo.toml、tauri.conf.json、package.json 中的版本号必须保持统一。**

> **⚠️ README.md 更新检查清单（推送前必做）：**
> - [ ] 版本号是否与当前 tag 一致
> - [ ] 功能列表是否包含本次新增/修改的功能
> - [ ] 截图是否更新为最新 UI（幕前 + 幕后）
> - [ ] 应用图标/Logo 是否为最新版本
> - [ ] 安装说明是否需要调整
> - [ ] 使用指南是否反映最新交互方式
> - [ ] CHANGELOG 链接是否有效

> **⚠️ 代码更新后必做（永久记住）：**
> - [ ] **重新构建 Windows `.exe`** — 任何前端代码（JS/CSS/TSX）或后端代码（Rust）修改后，必须执行 `cargo tauri build` 重新生成安装包，并复制产物到项目根目录
> - [ ] 验证 `StoryForge.exe`、`StoryForge_latest.exe`、`.msi`、`-setup.exe` 修改时间是否最新

> **🧠 AI 创作工具交互设计原则（永久记住）：**
> - **智能判断用户意图，主动调整状态** — 不要像传统软件一样弹出对话框让用户手工操作。例如：用户输入"写一篇小说"但无章节时，应**自动创建第一章**而非提示"请先选择章节"；文思模式非 active 时应**自动切换**而非提示用户按键。
> - **减少用户操作步骤** — AI 工具的核心价值是智能代理，用户给出意图后，工具应自动完成所有必要的配置和准备工作。
> - **避免非智能的传统软件式交互** — 不要用 toast/dialog/alert 来要求用户做本应由 AI 自动完成的事情。错误提示只用于真正无法自动处理的情况（如网络断开、API 密钥缺失）。

**本地构建脚本**: `scripts/build-local.ps1`
```powershell
# 推送前必执行：生成本地 Windows 安装包
.\scripts\build-local.ps1 -Windows
```
```powershell
# Windows 本地构建
.\scripts\build-local.ps1

# 或指定平台
.\scripts\build-local.ps1 -Windows
.\scripts\build-local.ps1 -All
```

**构建产物位置**（执行 `cargo tauri build` 后）：
```
target/x86_64-pc-windows-msvc/release/
├── storyforge.exe                          ← 30MB+，可直接运行
└── bundle/
    ├── msi/StoryForge_3.4.0_x64_en-US.msi  ← MSI安装包
    └── nsis/StoryForge_3.4.0_x64-setup.exe ← NSIS安装程序
```
> 为方便取用，每次构建后应将产物复制到项目根目录：`StoryForge.exe` 和 `StoryForge_3.4.0_x64-setup.exe`

**现实限制**:
- Windows 主机 ✅ 可本地构建 Windows (.msi/.exe)
- Linux 主机 ⚠️ 需 WSL 或 Linux 虚拟机
- macOS 主机 ❌ 无法在 Windows 上本地构建（需 macOS + Xcode）
- 跨平台完整构建 → 交由 GitHub Actions (`ubuntu-latest` / `windows-latest` / `macos-latest`)

---

## 🏛️ Spec-Kit 集成 (Spec-Driven Development)

本项目已集成 **GitHub Spec-Kit**，使用 Spec-Driven Development (SDD) 方法论管理功能开发。

### Spec-Kit 技能命令

在 Kimi Code 中使用以下 `/skill:` 命令：

| 命令 | 用途 | 阶段 |
|------|------|------|
| `/skill:speckit-constitution` | 查看/更新项目宪法 |  anytime |
| `/skill:speckit-specify` | 创建功能规格说明 | Phase 1 |
| `/skill:speckit-plan` | 生成技术实现计划 | Phase 2 |
| `/skill:speckit-tasks` | 分解为可执行任务 | Phase 3 |
| `/skill:speckit-implement` | 执行实现 | Phase 4 |
| `/skill:speckit-clarify` | 澄清需求模糊点 | Optional |
| `/skill:speckit-analyze` | 跨工件一致性检查 | Optional |
| `/skill:speckit-checklist` | 生成质量检查清单 | Optional |

### 文件结构

```
.specify/
├── memory/
│   └── constitution.md      # 项目宪法
├── templates/
│   ├── constitution-template.md
│   ├── spec-template.md
│   ├── plan-template.md
│   ├── tasks-template.md
│   └── checklist-template.md
├── scripts/
│   └── powershell/          # PowerShell 工作流脚本
│       ├── check-prerequisites.ps1
│       ├── create-new-feature.ps1
│       └── setup-plan.ps1
├── workflows/
│   └── speckit/
│       └── workflow.yml     # 完整 SDD 工作流定义
├── init-options.json
└── integration.json

.kimi/
└── skills/                  # Kimi Code 技能文件
    ├── speckit-constitution/SKILL.md
    ├── speckit-specify/SKILL.md
    ├── speckit-plan/SKILL.md
    ├── speckit-tasks/SKILL.md
    ├── speckit-implement/SKILL.md
    └── ...

specs/                       # 功能规格目录（按功能分支组织）
└── NNN-feature-name/
    ├── spec.md              # 功能规格
    ├── plan.md              # 实现计划
    ├── tasks.md             # 任务列表
    ├── checklists/
    │   └── requirements.md  # 质量检查清单
    ├── research.md          # 技术研究 (可选)
    ├── data-model.md        # 数据模型 (可选)
    └── contracts/           # 接口契约 (可选)
```

### 快速开始一个新功能

```powershell
# 1. 创建新功能分支和规格目录
.specify/scripts/powershell/create-new-feature.ps1 '功能描述'

# 2. 在 Kimi Code 中执行
/skill:speckit-specify 功能描述...
/skill:speckit-plan
/skill:speckit-tasks
/skill:speckit-implement
```

### 配置

- **AI 助手**: kimi (Kimi Code CLI)
- **脚本类型**: PowerShell (ps)
- **分支编号**: sequential (001, 002, ...)
- **项目宪法**: `.specify/memory/constitution.md`

---

*最后更新: 2026-04-17 - Spec-Kit 集成完成，项目宪法已建立，版本号统一为 3.3.0*
