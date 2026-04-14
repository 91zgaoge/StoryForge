# Changelog

All notable changes to StoryForge (草苔) project will be documented in this file.

## [Unreleased] - 意图引擎与 Agent 调度 + 知识图谱可视化 + 自动归档

### 🕸️ 知识图谱可视化

- **后端图数据 API**
  - `get_relations_by_story`：按故事 ID 批量查询关系
  - `get_story_graph`：一次性返回完整知识图谱（实体 + 关系）

- **交互式图谱视图** (`src-frontend/src/components/KnowledgeGraph/`)
  - 基于 **ReactFlow** 实现可缩放、可拖拽的力导向图谱
  - 节点按实体类型着色（角色/地点/物品/组织/概念/事件）
  - 关系边按强度显示不同粗细和透明度，高强度边带动画效果
  - 左上角图例面板显示统计信息
  - 点击节点展开右侧详情面板，展示属性和关联关系

- **页面集成**
  - 新增 backstage 「知识图谱」页面和 Sidebar 导航入口
  - 自动绑定当前选中的故事，空状态引导用户先选择故事

### 🧠 记忆健康与自动归档系统

- **后端保留报告 API**
  - `get_retention_report`：基于 Ebbinghaus 遗忘曲线计算实体保留状态
  - 复用已有的 `RetentionManager`，按实体类型应用不同衰减配置

- **自动归档工作流**
  - `kg_entities` 表新增 `is_archived` 和 `archived_at` 字段
  - `archive_forgotten_entities`：一键归档所有遗忘状态实体
  - `restore_archived_entity`：从归档状态恢复指定实体
  - `get_archived_entities`：查询故事的已归档实体列表
  - 数据库迁移脚本自动补全旧表缺失的保留/归档字段

- **记忆健康面板**（集成在知识图谱页面）
  - 汇总卡片：总实体数、平均优先级、系统健康状态
  - 自动归档建议：根据遗忘比例生成动态推荐文案，支持一键执行
  - 优先级分布可视化：关键/高/中/低/已遗忘五级进度条
  - 关键实体列表和待归档实体列表

- **已归档页签**
  - 知识图谱页面新增「已归档」标签页
  - 展示所有已归档实体，支持逐条恢复

### 🤖 Agent 模型映射与路由

- **后端配置持久化**
  - `AppConfig` 新增 `agent_mappings` 字段，支持 JSON 持久化
  - 默认映射：writer/inspector/outline_planner/style_mimic/plot_analyzer → Qwen 3.5
  - `get_settings` / `save_settings` 完整读写 agent_mappings
  - `get_agent_mappings` / `update_agent_mapping` 从硬编码改为读取/写入真实配置

- **模型路由逻辑**
  - `LlmService` 新增 `generate_with_profile`，支持按模型 ID 调用指定配置
  - `AgentService` 新增 `generate_for_agent`，自动根据 Agent 类型查找映射模型
  - 5 种 Agent（写作/质检/大纲/文风/情节）均已接入模型路由
  - 未配置映射时自动回退到活跃 LLM Profile

### 🧠 意图解析引擎 (Intent Engine)

- **后端意图解析器** (`src-tauri/src/intent.rs`)
  - 基于 LLM 的 JSON 意图提取，支持 11 种意图类型
  - 包含 `IntentParser`（解析）和 `IntentExecutor`（执行）两个核心组件
  - 新增 `parse_intent` 和 `execute_intent` Tauri 命令

- **Agent 调度执行**
  - 将意图的 `required_agents` 映射到现有的 `AgentService`
  - 支持串行 (`serial`) 和并行 (`parallel`) 两种执行模式
  - 执行结果包含每个 Agent 的步骤输出、评分和建议

- **前端意图感知对话**
  - `useIntent` Hook 新增 `executeIntent` 方法
  - `RichTextEditor` 聊天栏根据意图类型自动选择执行路径
  - `text_generate` / `text_rewrite` 继续走流式输出路径
  - `plot_suggest` / `character_check` / `world_consistency` 等走 Agent 调度路径
  - 聊天消息显示意图标签（如 "情节建议 · 建议卡片"）

### 🛠️ 技能工坊 (Skills) 后端连通

- **前端类型对齐**
  - `Skill` 接口扩展为完整 `SkillInfo` 结构，包含 `parameters`、`hooks`、`runtime_type` 等字段

- **真实数据接入**
  - `Skills.tsx` 从 mock 数据改为调用 `getSkills()` 拉取后端技能列表
  - 支持按分类筛选（全部 / 写作 / 分析 / 角色 / 情节 / 风格等）

- **技能操作**
  - 启用/禁用开关调用 `enable_skill` / `disable_skill`
  - 执行按钮支持 Prompt 技能运行，自动弹出必填参数输入框
  - 非内置技能显示卸载按钮，调用 `uninstall_skill`

## [3.1.2] - 2026-04-13 - 设置页增强、浏览器开发环境修复与全新应用图标

### 🎨 全新应用图标

- 从 [iconbuddy.com](https://iconbuddy.com) 引入 **Lucide `feather`** 作为 StoryForge 品牌图标
- 设计理念：羽毛笔象征创作与文学，金色羽毛配合深色背景，优雅且富有辨识度
- 使用 `cargo tauri icon` 重新生成全平台图标包（Windows .ico / macOS .icns / iOS / Android / UWP）
- 前端 favicon 同步替换为 `feather.svg`

### 🔧 幕后设置页修复

- **编辑模型模态框修复**
  - 修复 `custom` 提供商在编辑时缺少 API Key 输入框的问题
  - 现在 `custom` 类型模型始终显示 API Key 字段，兼容本地无密钥与有密钥模型

- **模型连接状态指示灯**
  - 模型卡片右上角新增实时连接状态检测
  - **检测中**：灰色加载动画
  - **已连接 (xxms)**：绿色圆点 + 延迟显示
  - **连接失败**：红色圆点（hover 查看错误详情）
  - 浏览器开发环境下通过 `fetch` 探测 `api_base` 可用性（5 秒超时）

### 🌐 浏览器开发环境兼容

- **Vite dev server 模型回退**
  - `getModels()` / `getSettings()` / `testModelConnection()` 在浏览器环境下自动回退到本地硬编码模型
  -  backstage 设置页在 `npm run dev` 浏览器模式下不再显示「暂无模型配置」
  - 同步更新 `docs/images/backstage-preview.png`

---

## [3.1.1] - 2026-04-13 - 幕前界面重构、Waza 设计与 CI 修复

### 🎭 幕前界面重构（Waza 设计原则落地）

- **精简侧边栏**
  - 侧边栏宽度缩减至 120px，仅保留"幕后"切换按钮
  - 去除冗余图标和文字，追求极简禅意
  - 修复按钮溢出侧边栏宽度的布局问题

- **颜色系统重构（OKLCH）**
  - 所有 Hex/HSL 颜色替换为 OKLCH，建立感知均匀的 60-30-10 视觉权重
  - 主背景：`oklch(96.5% 0.008 95)`（暖纸张色）
  - 强调色：`oklch(58% 0.13 45)`（赤陶色）
  - 去除装饰性纸张噪点纹理，背景更纯净

- **字体系统升级**
  - 移除 Waza 反感的 Crimson Pro / Cormorant Garamond / Inter
  - 正文字体统一为「霞鹜文楷 (LXGW WenKai) + 思源宋体」
  - 无衬线回退：`SF Pro Display / Segoe UI / PingFang SC`

- **微交互与排版**
  - 所有按钮增加 `active:scale-95` 触感反馈
  - 全面清除 `transition: all` 反模式，改为精确属性过渡
  - Blockquote 从左边框模板改为「背景色块 + 大引号装饰」

- **顶部动态状态栏**
  - 字数统计、字体大小、快捷键提示、保存状态集中展示
  - 去除底部固定的 AI 续写按钮，界面更加纯净

- **底部 LLM 对话栏**
  - 默认隐藏，鼠标悬停底部区域时优雅浮现
  - 集成模型状态指示灯（绿/黄/红三色 + 呼吸动画）
  - 去除对话/多模态模式切换图标，保持输入框极简
  - 占位文案："在此驾驭智能文思"
  - Enter 发送，Shift+Enter 换行，支持流式对话输出

### 🤖 本地三模型配置

- **Gemma-4-31B-it-Q6_K** (`http://10.62.239.13:17099/v1`)
  - 用途：多模态对话
  - 状态：已配置，无 API Key

- **Qwen3.5-27B-Uncensored-Q4_K_M** (`http://10.62.239.13:17098/v1`)
  - 用途：语言模型对话（默认"文思助手"）
  - 状态：已配置，无 API Key

- **bge-m3** (`http://10.62.239.13:8089`)
  - 用途：Embedding 向量嵌入
  - 状态：已配置，带 API Key

### 🖥️ Tauri 本地构建与 CI 修复

- 修复 `tauri.conf.json` 中 `beforeBuildCommand` 在 Windows 下的路径兼容性问题
- 成功构建 Release 版本并打包 Windows 安装程序
- 生成 MSI (12.3 MB) 和 NSIS (8.1 MB) 两种安装包
- 修复 GitHub Actions 跨平台构建缺少 `icons/icon.icns` 的问题
- `rust-check` 三平台（Ubuntu / Windows / macOS）全部通过
- **自动发布 Nightly Release**：每次推送到 master 自动构建并发布三平台安装包到 GitHub Releases

---

## [3.1.0] - 2025-04-13 - 智能记忆与版本管理

### 🔍 Hybrid Search (混合搜索)

**Phase 1.3 Implementation**

- **BM25 Search** (`memory/hybrid_search.rs`)
  - CJK Bigram tokenizer for Chinese text
  - Inverted index with TF-IDF scoring
  - Configurable k1 and b parameters

- **Hybrid Search Engine**
  - BM25 + Vector similarity fusion
  - RRF (Reciprocal Rank Fusion) ranking
  - Configurable weights (default: BM25 40%, Vector 60%)

- **Entity Hybrid Search**
  - Name matching + vector similarity
  - Cosine similarity calculation
  - Priority scoring for entity retrieval

### 📜 Scene Version Management (场景版本管理)

**Phase 3.x Implementation**

- **SceneVersionRepository** (`db/repositories_v3.rs`)
  - `create_version()` - Snapshot current scene state
  - `get_versions()` - List version history
  - `get_version()` - Get specific version
  - `delete_version()` - Remove version

- **SceneVersionService** (`versions/service.rs`)
  - `compare_versions()` - Line-level diff with word count delta
  - `restore_version()` - Restore to any historical version
  - `get_version_chain()` - Version chain with branch structure
  - `get_version_stats()` - Edit distribution, avg confidence

- **Frontend Components**
  - `VersionTimeline.tsx` - Vertical timeline with selection
  - `ConfidenceIndicator.tsx` - Circular/bar progress indicator
  - `DiffViewer.tsx` - Side-by-side diff view
  - `useSceneVersions.ts` - React Query hooks

### 🧠 Memory Retention Management (记忆保留管理)

**Phase 1.4 Implementation**

- **RetentionManager** (`memory/retention.rs`)
  - Ebbinghaus forgetting curve: R(t) = R₀ × e^(-λt)
  - 5 priority levels: Critical/High/Medium/Low/Forgotten
  - Retention report generation
  - Context window optimization

---

## [3.0.0] - 2025-04-12 - 重大架构调整

### 🎪 场景化叙事架构
- Scene 取代 Chapter，戏剧冲突驱动
- 戏剧目标、外部压迫、冲突类型、角色冲突
- StoryTimeline 拖拽排序、SceneEditor 三标签页

### 🧠 增强记忆系统
- CJK Bigram Tokenizer
- 两步 Ingest Pipeline
- 带权知识图谱
- 四阶段 Query Pipeline
- 多助手独立会话

### 🤖 AI 智能生成
- NovelCreationAgent
- 4 步引导式创建向导
- 卡片式 UI

### 📦 工作室配置
- 每部小说独立配置
- ZIP 导入/导出

---

## [2.0.0] - 2025-04-12

- 幕前-幕后双界面架构
- 双窗口通信

## [1.5.0] - 2025-04-08

- Agent 系统
- 工作流引擎
- 向量存储

## [1.0.0] - 2025-04-01

- 基础架构
- LLM 集成
- 数据库设计
