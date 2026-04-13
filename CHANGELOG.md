# Changelog

All notable changes to StoryForge (草苔) project will be documented in this file.

## [3.1.1] - 2026-04-13 - 幕前界面重构与本地模型配置

### 🎭 幕前界面重构

- **精简侧边栏**
  - 侧边栏宽度缩减至 120px，仅保留"幕后"切换按钮
  - 去除冗余图标和文字，追求极简禅意

- **顶部动态状态栏**
  - 字数统计、字体大小、快捷键提示、保存状态集中展示
  - 去除底部固定的 AI 续写按钮，界面更加纯净

- **底部 LLM 对话栏**
  - 默认隐藏，鼠标悬停底部区域时优雅浮现
  - 集成模型状态指示灯（绿/黄/红三色 + 呼吸动画）
  - 支持 💬 对话 / 🖼️ 多模态 模式切换
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

### 🖥️ Tauri 本地构建修复

- 修复 `tauri.conf.json` 中 `beforeBuildCommand` 在 Windows 下的路径兼容性问题
- 成功构建 Release 版本并打包 Windows 安装程序
- 生成 MSI (12.3 MB) 和 NSIS (8.1 MB) 两种安装包

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
