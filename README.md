<p align="center">
  <img src="docs/images/logo.png" alt="StoryForge 草苔" width="120" />
</p>

# StoryForge (草苔) v3.4.0 - 智能化创作系统

> 🌿 越写越懂的创作系统 - AI 辅助小说创作桌面应用
>
> v3.4.0 实现从"功能堆砌"到"系统智能"的跨越：创作方法论引擎 × StyleDNA × 自适应学习 × 工作流闭环

## 🎭 独具特色的双界面设计

StoryForge 独创**"幕前 - 幕后"**双界面架构，让创作与阅读完美融合：

### 🎬 幕前 (Frontstage) - 沉浸式阅读写作

**设计理念**：像阅读一本精美小说一样写作

- **OKLCH 暖色纸张** - 感知均匀的色彩系统，`oklch(96.5% 0.008 95)` 暖色调背景，护眼舒适
- **霞鹜文楷正文字体** - 采用 LXGW WenKai 作为正文字体，中文排版优雅，去除通用字体的"AI 感"
- **大字号阅读体验** - 18px 正文字号，1.8 倍行距，久写不累
- **顶部动态状态栏** - 字数、字号、快捷键提示、保存状态一目了然
- **底部 LLM 对话栏** - 悬停显示，集成模型状态灯，去除多余图标保持极简
- **AI 流式续写** - Ctrl+Space 开启「文思」，文字如泉水般涌现
- **禅模式** - F11 快捷键进入全屏沉浸，专注创作无干扰
- **后台设置同步** - 写作风格、字体设置在后台统一管理
- **精简侧边栏 Dock** - 修订模式、文本批注、评论线程、古典评点、幕后切换一键直达
- **右键上下文菜单** - 编辑器内右键唤起，支持剪切/复制/粘贴、修订模式、文本批注、评论线程、生成古典评点、全选
- **角色卡片弹窗** - 点击角色名自动高亮并弹出角色详情卡片
- **AI 氛围提示气泡** - 如萤火虫般在右侧浮现的创作建议，不干扰阅读
- **文本内联批注** - 选中文本添加高亮批注（note / todo / warning / idea）
- **评论线程** - 选中文本发起评论讨论，支持多轮回复与解决
- **修订模式与变更追踪** - 开启后所有增删以可视化痕迹记录，支持逐条接受/拒绝
- **古典评点生成** - AI 模拟金圣叹风格对段落进行实时文学点评

![幕前界面预览](docs/images/frontstage-preview.png)

### 🔧 幕后 (Backstage) - 全能创作工作室

**设计理念**：专业作家的数字工作台

- **故事管理** - 多故事、多场景结构化组织
- **角色管理** - 角色卡片、关系图谱、性格追踪
- **场景化叙事** - 以场景为单位的戏剧冲突驱动
- **场景编辑器** - 三标签页设计（基础信息 / 戏剧结构 / 内容编辑）
- **知识图谱可视化** - 基于 ReactFlow 的交互式力导向图谱，支持搜索、筛选、实体编辑
- **记忆健康与自动归档** - 基于艾宾浩斯遗忘曲线的实体保留分析，一键归档遗忘内容
- **版本控制** - 场景历史自动快照，行级 diff 对比，随时回溯
- **技能系统** - AI 技能插件工坊，支持导入/启用/禁用/执行
- **MCP 外部服务器** - 连接外部 MCP 服务器，扩展工具生态
- **数据导出** - 支持 PDF、EPUB、Markdown 等多种格式
- **模型映射与路由** - 为不同 Agent 独立配置 LLM 模型
- **意图引擎** - 聊天栏自动解析用户意图并调度对应 Agent 执行
- **创作方法论引擎** - 雪花法 / 场景节拍 / 英雄之旅 / 人物深度，自动注入创作约束
- **StyleDNA 系统** - 六维定量风格模型，10 种经典作家 DNA，实时风格匹配
- **自适应学习系统** - 记录用户反馈、挖掘偏好、动态调节生成参数
- **创作工作流引擎** - 7 阶段全自动工作流（构思→大纲→场景→写作→审阅→迭代→入库）

![幕后界面预览](docs/images/backstage-preview.png)

### 🔄 双窗口无缝协作

| 功能 | 幕前 | 幕后 |
|------|------|------|
| 阅读写作 | ✅ 沉浸式体验 | - |
| 故事管理 | - | ✅ 完整功能 |
| 场景管理 | ✅ 快速切换 / 版本历史 | ✅ 详细编辑 / 故事线拖拽 |
| AI 续写 | ✅ 流式生成 / 自动续写 | ✅ 参数调节 / 方法论约束 |
| 角色查看 | ✅ 卡片式预览 | ✅ 完整编辑 / 关系图谱 / StyleDNA |
| 知识图谱 | - | ✅ 可视化 / 编辑 / 归档 |
| 技能执行 | ✅ 快捷执行 | ✅ 技能工坊管理 |
| 文本批注 | ✅ 内联批注 / 评论线程 | ✅ 场景级批注 |
| 创作方法论 | - | ✅ 雪花法 / 节拍表 / 英雄之旅 / 人物深度 |
| StyleDNA | - | ✅ 风格选择 / 相似度分析 |
| 工作流引擎 | - | ✅ 一键全自动创作 |
| 自适应学习 | - | ✅ 反馈记录 / 偏好挖掘 |

**快捷键对照**：
- `F11` - 幕前禅模式切换
- `Ctrl+Space` - 开启/关闭 AI 文思
- `Tab` - 接受 AI 建议
- `Esc` - 拒绝 AI 建议

---

## ✨ v3.4.0 核心新特性

### 🧠 智能化创作系统（5 阶段重构）

**Phase 1 - 地基重构：真实上下文**
- `StoryContextBuilder` — 从真实数据库构建丰富的 Agent 上下文（世界观、角色、场景结构）
- `QueryPipeline` — 四阶段知识检索（CJK 分词搜索 → 知识图谱扩展 → 预算控制 → 上下文组装）
- `ContinuityEngine` + `ForeshadowingTracker` — 连续性追踪与伏笔回收系统

**Phase 2 - 方法论注入**
- 创作方法论引擎：`MethodologyEngine` 自动将方法论约束注入 Writer 系统提示词
- 四种经典方法论：雪花法（10 步）· 场景节拍表（6 节拍）· 英雄之旅（12 阶段）· 人物深度模型（6 维度）
- `AgentOrchestrator` — Writer→Inspector→Writer 质量反馈循环（可配置阈值与最大循环数）

**Phase 3 - 风格深度化**
- `StyleDNA` 六维定量模型：词汇/句法/修辞/视角/情感/对白
- 10 种内置经典作家 DNA：金庸、张爱玲、海明威、村上春树、莫言、古典散文、现代极简、黑色侦探、武侠诗意、浪漫主义
- 实时风格相似度计算与提示词注入

**Phase 4 - 自适应学习**
- `FeedbackRecorder` — 记录用户对 AI 生成内容的接受/拒绝/修改行为
- `PreferenceMiner` — 五维度启发式偏好挖掘（主题/风格/节奏/视角/结构）
- `AdaptiveGenerator` — 动态调节温度（temperature）、top-p、提示词权重
- `PromptPersonalizer` — 将用户偏好自动注入系统提示词

**Phase 5 - 工作流闭环**
- `CreationWorkflowEngine` — 7 阶段全自动工作流：构思 → 大纲 → 场景设计 → 写作 → 审阅 → 迭代 → 入库
- 3 种创作模式：一键全自动 / AI 初稿 + 人工精修 / 人工初稿 + AI 润色
- `QualityChecker` — 四维质量评估（结构/人物/风格/情节）

### 🎨 品牌焕新

- 全新 Logo：「草苔」立方体标志 —— 融合自然叶脉纹理的几何立方体造型
- `cargo tauri icon logo.png` 生成全平台图标包（Windows / macOS / iOS / Android）

### 🏗️ 架构与质量

- **160 项测试全部通过**（Rust 139 + 前端 21）
- `cargo check` 零警告
- 版本号统一：Cargo.toml / package.json / tauri.conf.json → 3.4.0

---

## 📊 项目状态概览

**当前版本**: v3.4.0  
**最后更新**: 2026-04-19  
**GitHub**: https://github.com/91zgaoge/StoryForge  
**整体完成度**: 100%

> 🍃 品牌图标：「草苔」立方体标志 —— 融合自然叶脉纹理的几何立方体造型，象征创作的结构化生长与文学的立体纵深

| 模块 | 状态 | 完成度 |
|------|------|--------|
| 核心架构 | ✅ 稳定 | 100% |
| 场景化系统 | ✅ 完成 | 100% |
| 记忆系统 | ✅ 完成 | 100% |
| AI 生成 | ✅ 完成 | 100% |
| 知识图谱可视化 | ✅ 完成 | 100% |
| 工作室配置 | ✅ 完成 | 100% |
| 双界面设计 | ✅ 完成 | 100% |
| LLM 集成 / 流式输出 | ✅ 完成 | 100% |
| 本地模型配置 | ✅ 完成 | 100% |
| Agent 系统 / 意图引擎 | ✅ 完成 | 100% |
| 技能系统 / MCP | ✅ 完成 | 100% |
| 版本控制 / 修订模式 | ✅ 完成 | 100% |
| 文本批注 / 评论线程 | ✅ 完成 | 100% |
| 前端界面 | ✅ 完成 | 100% |
| 桌面构建打包 | ✅ 完成 | 100% |
| 创作方法论引擎 | ✅ 完成 | 100% |
| StyleDNA 系统 | ✅ 完成 | 100% |
| 自适应学习系统 | ✅ 完成 | 100% |
| 创作工作流引擎 | ✅ 完成 | 100% |
| 拆书功能 | ✅ 完成 | 100% |
| 任务系统 | ✅ 完成 | 100% |
| 测试覆盖 | ✅ 完成 | 160 tests |

---

## 🗂️ 项目结构

```
v2-rust/
├── src-frontend/                 # 前端代码 (React + TypeScript)
│   ├── src/
│   │   ├── main.tsx             # 幕后入口
│   │   ├── App.tsx              # 幕后主应用
│   │   ├── frontstage/          # 幕前界面
│   │   │   ├── FrontstageApp.tsx
│   │   │   ├── components/
│   │   │   │   ├── RichTextEditor.tsx    # TipTap 富文本编辑器
│   │   │   │   ├── EditorContextMenu.tsx # 右键上下文菜单
│   │   │   │   ├── AiSuggestionBubble.tsx # AI 氛围提示
│   │   │   │   ├── CharacterCardPopup.tsx # 角色卡片弹窗
│   │   │   │   └── ChapterOutline.tsx
│   │   │   └── styles/frontstage.css
│   │   ├── pages/               # 幕后页面
│   │   │   ├── Dashboard.tsx    # 仪表盘
│   │   │   ├── Stories.tsx      # 故事库
│   │   │   ├── Characters.tsx   # 角色管理
│   │   │   ├── Scenes.tsx       # 场景管理
│   │   │   ├── Chapters.tsx     # 章回管理
│   │   │   ├── KnowledgeGraph.tsx # 知识图谱
│   │   │   ├── Skills.tsx       # 技能工坊
│   │   │   ├── Mcp.tsx          # MCP 服务器配置
│   │   │   └── Settings.tsx     # 设置中心
│   │   ├── components/          # 共享组件
│   │   │   ├── StoryTimeline.tsx    # 故事线视图
│   │   │   ├── SceneEditor.tsx      # 场景编辑器
│   │   │   ├── VersionTimeline.tsx  # 版本历史
│   │   │   ├── DiffViewer.tsx       # 版本对比
│   │   │   ├── VectorSearch.tsx     # 向量搜索
│   │   │   ├── NovelCreationWizard.tsx # 创建向导
│   │   │   └── ExportDialog.tsx
│   │   └── hooks/               # 自定义 Hooks
│   │       ├── useScenes.ts
│   │       ├── useIntent.ts         # 意图解析
│   │       ├── useChangeTracking.ts # 修订追踪
│   │       ├── useTextAnnotations.ts # 文本批注
│   │       ├── useCommentThreads.ts  # 评论线程
│   │       ├── useSceneVersions.ts   # 版本管理
│   │       ├── useMcpTools.ts        # MCP 工具
│   │       ├── useVectorSearch.ts    # 向量搜索
│   │       └── useStudioConfig.ts
│   ├── index.html               # 幕后 HTML
│   ├── frontstage.html          # 幕前 HTML
│   └── package.json
│
├── src-tauri/                   # Tauri 后端 (Rust)
│   ├── src/
│   │   ├── main.rs              # 应用入口
│   │   ├── lib.rs               # 库入口
│   │   ├── commands.rs          # Tauri 命令
│   │   ├── commands_v3.rs       # V3 命令集
│   │   ├── intent.rs            # 意图解析引擎
│   │   ├── db/                  # 数据库层
│   │   │   ├── models_v3.rs
│   │   │   └── repositories_v3.rs
│   │   ├── agents/              # Agent 系统
│   │   │   ├── service.rs
 │   │   │   ├── orchestrator.rs  # Writer→Inspector 质量闭环
│   │   │   ├── commentator.rs   # 古典评点家
│   │   │   ├── memory_compressor.rs
│   │   │   └── novel_creation.rs
 │   │   ├── creative_engine/     # 智能化创作引擎 (v3.4.0)
 │   │   │   ├── mod.rs
 │   │   │   ├── context_builder.rs    # 真实 DB 上下文构建
 │   │   │   ├── continuity.rs         # 连续性追踪
 │   │   │   ├── foreshadowing.rs      # 伏笔回收系统
 │   │   │   ├── methodology/          # 创作方法论引擎
 │   │   │   │   ├── mod.rs
 │   │   │   │   ├── snowflake.rs
 │   │   │   │   ├── scene_structure.rs
 │   │   │   │   ├── hero_journey.rs
 │   │   │   │   └── character_depth.rs
 │   │   │   ├── style/                # StyleDNA 系统
 │   │   │   │   ├── mod.rs
 │   │   │   │   ├── dna.rs
 │   │   │   │   └── classic_styles.rs
 │   │   │   ├── adaptive/             # 自适应学习系统
 │   │   │   │   ├── mod.rs
 │   │   │   │   ├── feedback.rs
 │   │   │   │   ├── miner.rs
 │   │   │   │   ├── generator.rs
 │   │   │   │   └── personalizer.rs
 │   │   │   └── workflow/             # 工作流引擎
 │   │   │       ├── mod.rs
 │   │   │       ├── engine.rs
 │   │   │       └── quality.rs
│   │   ├── memory/              # 记忆系统
│   │   │   ├── tokenizer.rs
│   │   │   ├── ingest.rs
│   │   │   ├── query.rs
│   │   │   ├── hybrid_search.rs
│   │   │   └── multi_agent.rs
│   │   ├── llm/                 # LLM 适配器
│   │   │   ├── adapter.rs
│   │   │   ├── openai.rs
│   │   │   ├── anthropic.rs
│   │   │   └── ollama.rs
│   │   ├── collab/              # 协作编辑
│   │   │   └── websocket.rs
│   │   └── config/              # 配置管理
│   │       └── studio_manager.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── docs/                        # 文档
├── README.md
├── CHANGELOG.md
├── ARCHITECTURE.md
└── AGENTS.md
```

---

## 🎨 前端双界面架构

### 技术栈
- **React 18** - UI 框架
- **Vite 6** - 构建工具，支持多入口
- **TypeScript 5.8** - 类型安全
- **Tailwind CSS 3** - 原子化样式
- **TipTap** - ProseMirror 富文本编辑器
- **Zustand** - 轻量状态管理
- **TanStack Query** - 服务端状态管理
- **Tauri 2.4** - 桌面应用框架
- **@dnd-kit** - 拖拽排序
- **ReactFlow** - 知识图谱可视化

### 核心组件

#### RichTextEditor - 幕前富文本编辑器
集成 TipTap 编辑器与 LLM 对话栏：
- 文本内联批注高亮（note / todo / warning / idea）
- 评论线程锚点高亮
- 修订模式 trackInsert / trackDelete 可视化标记
- 角色卡片 `@` 提及弹窗
- 古典评点段落自动插入
- 底部对话栏悬停显示，模型状态灯与流式对话
- 右键上下文菜单

#### StoryTimeline - 故事线视图
可视化场景序列，支持拖拽重新排序：
- 场景卡片展示戏剧目标、冲突类型
- 拖拽手柄调整场景顺序
- 点击选择场景进行编辑

#### SceneEditor - 场景编辑器
三标签页场景编辑：
- **基础信息** - 标题、场景设置、在场角色、记忆压缩
- **戏剧结构** - 戏剧目标、外部压迫、冲突类型（11 种）
- **内容编辑** - 富文本编辑器、场景批注、版本历史

#### KnowledgeGraphView - 知识图谱可视化
基于 ReactFlow 的交互式力导向图谱：
- 节点按实体类型着色，关系边按强度显示粗细
- 实时搜索与类型筛选，双击节点聚焦居中
- 右侧详情面板支持实体就地编辑
- 记忆健康面板与自动归档建议

#### NovelCreationWizard - 创建向导
引导式小说创建流程：
- 类型输入（灰色提示词）
- 世界观 / 角色谱 / 文风卡片式选择
- 完成自动 Ingest 到知识图谱

---

## ✅ 功能实现详情

### 1. 智能化创作系统 v3.4.0 (100% ✅)

| 功能 | 状态 | 说明 |
|------|------|------|
| `StoryContextBuilder` | ✅ | 从真实数据库构建丰富的 Agent 上下文 |
| `QueryPipeline` | ✅ | 四阶段知识检索（CJK→图谱→预算→组装） |
| `ContinuityEngine` | ✅ | 章节连续性追踪与伏笔回收 |
| `ForeshadowingTracker` | ✅ | 伏笔埋设与回收追踪 |
| `MethodologyEngine` | ✅ | 雪花法/节拍表/英雄之旅/人物深度 |
| `AgentOrchestrator` | ✅ | Writer→Inspector→Writer 质量闭环 |
| `StyleDNA` | ✅ | 六维定量风格模型，10 种经典作家 DNA |
| `StyleAnalyzer` | ✅ | 从文本提取风格指纹 |
| `StyleChecker` | ✅ | 对比文本与目标 DNA 相似度 |
| `FeedbackRecorder` | ✅ | 记录接受/拒绝/修改行为 |
| `PreferenceMiner` | ✅ | 五维度启发式偏好挖掘 |
| `AdaptiveGenerator` | ✅ | 动态调节 temperature/top-p/权重 |
| `PromptPersonalizer` | ✅ | 将用户偏好注入系统提示词 |
| `CreationWorkflowEngine` | ✅ | 7 阶段全自动工作流 |
| `QualityChecker` | ✅ | 四维质量评估（结构/人物/风格/情节） |

### 2. 场景化叙事系统 (100% ✅)

| 功能 | 状态 | 说明 |
|------|------|------|
| Scene 模型 | ✅ | 戏剧目标、外部压迫、冲突类型 |
| SceneRepository | ✅ | CRUD + 重新排序 |
| 故事线视图 | ✅ | 拖拽排序、场景卡片 |
| 场景编辑器 | ✅ | 三标签页 + 批注 + 版本历史 |
| 冲突类型枚举 | ✅ | 11 种标准冲突类型 |
| 记忆压缩 | ✅ | MemoryCompressorAgent 集成 |

### 3. 记忆系统 (100% ✅)

| 功能 | 状态 | 说明 |
|------|------|------|
| CJK 分词器 | ✅ | 二元组分词，中日韩支持 |
| Ingest 管线 | ✅ | 两步思维链：分析→生成 |
| 知识图谱 | ✅ | 实体/关系带强度评分，ReactFlow 可视化 |
| 查询检索 | ✅ | 四阶段检索管线 |
| 多助手会话 | ✅ | 6 种助手类型独立会话 |
| 混合搜索 | ✅ | BM25 + 向量融合 (RRF) |
| FTS5 全文索引 | ✅ | SQLite 原生全文加速 |
| 场景版本 | ✅ | 版本历史、比较、恢复、版本链 |
| 记忆保留 | ✅ | 遗忘曲线、优先级管理、自动归档 |
| 向量持久化 | ✅ | SQLite + LanceDB 混合存储 |

### 4. AI 智能生成 (100% ✅)

| 功能 | 状态 | 说明 |
|------|------|------|
| NovelCreationAgent | ✅ | 世界观/角色/文风/首个场景生成 |
| 创建向导 | ✅ | 4 步引导流程，自动 Ingest |
| 卡片式 UI | ✅ | 单击选择，双击编辑 |
| 古典评点家 | ✅ | 金圣叹风格段落点评 |
| 意图引擎 | ✅ | 11 种意图解析 + Agent 调度 |
| 真实 SSE 流式 | ✅ | OpenAI / Anthropic / Ollama 全适配 |

### 5. 协作与批注系统 (100% ✅)

| 功能 | 状态 | 说明 |
|------|------|------|
| 修订模式 | ✅ | trackInsert / trackDelete，逐条接受/拒绝 |
| 文本内联批注 | ✅ | note/todo/warning/idea，高亮锚定 |
| 评论线程 | ✅ | 多轮回复、解决/重开、删除 |
| 场景批注 | ✅ | 场景级批注/待办/警告 |
| WebSocket 协作 | ✅ | 协作编辑服务端 |

### 6. 工作室配置与扩展 (100% ✅)

| 功能 | 状态 | 说明 |
|------|------|------|
| StudioConfig 模型 | ✅ | 每部小说独立配置 |
| ZIP 导出/导入 | ✅ | `.storyforge` 格式，选择性导入 |
| 技能系统 | ✅ | 内置 5+ 技能，支持导入/禁用/执行 |
| MCP 服务器 | ✅ | 外部服务器配置与工具调用 |
| 模型映射 | ✅ | Agent → LLM 独立路由 |
| 默认主题 | ✅ | 幕前暖色 / 幕后暗色 |

### 7. 本地模型与构建

| 模块 | 完成度 | 说明 |
|------|--------|------|
| 本地模型配置 | 100% | Gemma / Qwen / bge-m3 |
| LLM 集成 | 100% | OpenAI/Anthropic/Ollama/本地 API |
| Agent 系统 | 100% | 6 种 Agent + 模型路由 |
| 技能系统 | 100% | 内置技能 + 扩展支持 |
| 向量检索 | 100% | TF-IDF + BM25 + 语义 + 混合 |
| 导出功能 | 100% | PDF/EPUB/Markdown |
| Tauri 打包 | 100% | MSI + NSIS 安装包 |

---

## 📅 更新历史

### v3.4.0 (2026-04-18) - 智能化创作系统（5 阶段重构）

- **Phase 1 地基重构** - `StoryContextBuilder` 真实 DB 上下文, `QueryPipeline` 四阶段检索, `ContinuityEngine`, `ForeshadowingTracker` (27 tests)
- **Phase 2 方法论注入** - 雪花法/场景节拍/英雄之旅/人物深度 + `MethodologyEngine` + `AgentOrchestrator` Writer→Inspector 闭环 (34 tests)
- **Phase 3 风格深度化** - `StyleDNA` 六维模型, 10 经典作家 DNA, `StyleAnalyzer`, `StyleChecker` (45 tests)
- **Phase 4 自适应学习** - `FeedbackRecorder`, `PreferenceMiner`, `AdaptiveGenerator`, `PromptPersonalizer` (54 tests)
- **Phase 5 工作流闭环** - `CreationWorkflowEngine` 7 阶段工作流, `QualityChecker` 四维评估, 3 种创作模式 (63 tests)
- **品牌焕新** - `logo.png` 立方体标志生成全平台图标包
- **版本统一** - Cargo.toml / package.json / tauri.conf.json → 3.4.0

### v3.3.0 (2026-04-15) - 功能断层修复与架构清理

- **幕前右键菜单修复** - Tailwind utilities 补充、事件捕获修复、WebView2 默认菜单禁用、暖色 UI 重构
- **MCP 外部服务器连接** - 配置卡片 + 工具调用
- **技能导入** - 本地文件选择器导入
- **Agent 流式执行与取消** - 实时进度 + 可中断
- **知识图谱实体就地编辑** - 节点属性增删改
- **版本系统增强** - 版本链视图 + diff 元信息
- **LLM 调用路径决策** - 明确 HTTP 直连为官方路径
- **Rust Warnings 降噪** - `cargo check` 0 警告

### v3.2.0 (2026-04-14) - 意图引擎与 Agent 调度 + 知识图谱可视化 + 修订模式

- **知识图谱可视化** - ReactFlow 力导向图谱，搜索/筛选/实体编辑
- **记忆健康与自动归档** - 艾宾浩斯遗忘曲线 + 一键归档
- **意图解析引擎** - 11 种意图类型，聊天栏自动调度 Agent
- **Agent 模型映射** - 按 Agent 类型路由到不同 LLM
- **真实 SSE 流式输出** - OpenAI/Anthropic/Ollama 全适配
- **文本内联批注 + 评论线程** - 选中文本批注与讨论
- **修订模式与变更追踪** - trackInsert/trackDelete + 接受/拒绝
- **古典评点家 Agent** - 金圣叹风格文学点评
- **小说创建向导后端连通** - 4 步引导 + 自动 Ingest
- **记忆压缩师集成** - 场景内容智能压缩
- **SQLite FTS5 全文索引** - BM25 + 向量混合搜索

### v3.1.2 (2026-04-13) - 设置页增强、浏览器开发环境修复与全新应用图标

- **全新羽毛笔品牌图标**
- **模型连接状态指示灯** - 实时检测延迟
- **设置页编辑模型模态框修复** - `custom` 提供商兼容
- **浏览器开发环境兼容** - Vite dev server 模型回退

### v3.1.1 (2026-04-13) - 幕前 Waza 设计重构与 CI 修复

- **幕前界面重构** - OKLCH 颜色系统、LXGW WenKai 字体、精简侧边栏
- **底部 LLM 对话栏** - 悬停显示、流式对话、模型状态灯
- **本地三模型配置** - Gemma-4 / Qwen3.5 / bge-m3
- **Tauri 构建与 CI 修复** - MSI/NSIS 安装包、GitHub Actions Nightly Release

### v3.1.0 (2026-04-13) - 智能记忆与版本管理

- **混合搜索** - BM25 + Vector RRF 融合
- **场景版本历史** - 快照、diff、恢复、统计
- **记忆保留曲线** - 优先级分级、自动归档建议

### v3.0.0 (2026-04-12) - 重大架构调整

- **场景化叙事架构** - Scene 取代 Chapter
- **增强记忆系统** - CJK 分词、Ingest 管线、知识图谱
- **AI 智能生成** - 引导式小说创建
- **工作室配置** - 导入/导出功能

---

## 🚀 快速开始

### 环境要求
- Rust 1.70+
- Node.js 18+ (前端开发)
- SQLite 3

### 开发模式

**快速启动（Windows PowerShell）**:
```powershell
# 一键启动前端和后端
.\start-dev.ps1
```

**手动启动**:
```bash
# 1. 克隆项目
cd v2-rust

# 2. 安装依赖
cd src-frontend && npm install && cd ..

# 3. 终端 1 - 启动前端开发服务器
cd src-frontend && npm run dev

# 4. 终端 2 - 启动 Tauri 应用
cd src-tauri && cargo tauri dev

# 5. 构建发布版本（Windows）
cd src-tauri && cargo tauri build

# 构建产物
# target/release/storyforge.exe          - 独立可执行文件
# target/release/bundle/msi/*.msi        - MSI 安装包
# target/release/bundle/nsis/*-setup.exe - NSIS 安装包
```

**双界面入口**:
- 幕前界面: http://localhost:5173/frontstage.html
- 幕后界面: http://localhost:5173/index.html
- Tauri 应用会自动打开两个窗口，幕前在前，幕后在后

**故障排除**: 参考 [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

### 配置说明

配置文件位置：`~/.config/storyforge/config.json`

```json
{
  "llm": {
    "provider": "openai",
    "api_key": "your-api-key",
    "model": "gpt-4",
    "max_tokens": 4096,
    "temperature": 0.7
  }
}
```

---

## 🛣️ 路线图 (Roadmap)

### 已完成 (v3.4.0) ✅
- [x] **场景化叙事** - 场景取代章节，戏剧冲突驱动
- [x] **记忆系统** - 基于 llm_wiki 的知识图谱
- [x] **AI 智能生成** - 引导式小说创建
- [x] **工作室配置** - 导入/导出功能
- [x] **混合搜索** - BM25 + Vector RRF
- [x] **场景版本历史** - 快照、diff、版本链
- [x] **记忆保留曲线** - 自动归档
- [x] **幕前界面重构** - Waza / OKLCH / LXGW WenKai
- [x] **本地模型配置** - Gemma / Qwen / bge-m3
- [x] **Tauri 构建打包** - MSI / NSIS
- [x] **GitHub Actions CI** - Nightly Release
- [x] **知识图谱可视化** - ReactFlow 交互图谱
- [x] **意图引擎** - Agent 调度
- [x] **修订模式** - 变更追踪
- [x] **文本批注 / 评论线程** - 内联协作
- [x] **MCP 外部服务器** - 工具扩展
- [x] **技能工坊** - 导入/执行/管理
- [x] **创作方法论引擎** - 雪花法/节拍表/英雄之旅/人物深度
- [x] **StyleDNA 系统** - 六维风格模型 + 10 经典作家 DNA
- [x] **自适应学习系统** - 反馈→偏好→生成→个性化
- [x] **创作工作流引擎** - 7 阶段全自动闭环
- [x] **拆书功能** - txt/pdf/epub 解析，LLM 分析，一键转故事
- [x] **任务系统** - once/daily/weekly/cron 调度，心跳检测，防重叠执行
- [x] **向量化存储** - 场景/人物 embedding 自动生成并入库

### 短期计划 (v3.5.x)
- [ ] 前端 UI 接入新方法引擎（方法论选择、StyleDNA 面板、工作流启动）
- [ ] 性能优化（大数据量场景）
- [ ] 导出模板自定义

### 中期计划 (v3.6.0)
- [ ] 云端同步
- [ ] 协作写作增强（多人实时编辑）
- [ ] 插件市场

### 长期计划 (v4.0.0)
- [ ] WebAssembly 前端
- [ ] 自研小模型
- [ ] 移动端适配
- [ ] 发布平台集成

---

## 📚 相关文档

- [架构设计](ARCHITECTURE.md) - 详细架构说明
- [功能清单](docs/FEATURES.md) - 完整功能列表
- [更新日志](CHANGELOG.md) - 版本变更记录
- [项目状态](PROJECT_STATUS.md) - 开发进度
- [Agent 指南](AGENTS.md) - AI 助手开发指南
- [V3 架构计划](docs/plans/ARCHITECTURE_V3_PLAN.md) - V3 详细设计

---

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE)

---

**StoryForge (草苔)** - 让创作更智能 🌿
