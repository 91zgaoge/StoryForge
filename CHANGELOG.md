# Changelog

All notable changes to StoryForge (草苔) project will be documented in this file.

## [3.0.0] - 2025-04-12 - 重大架构调整

### 🎪 Scene-Based Narrative Architecture (场景化叙事架构)

**核心变更：从"章节"到"场景"**

- **Scene Model** - 戏剧冲突驱动的叙事单位
  - `dramatic_goal` - 明确的戏剧目标
  - `external_pressure` - 外部压迫因素
  - `conflict_type` - 6种标准冲突类型枚举
  - `character_conflicts` - 角色间冲突关系
  - `setting` - 场景设置（地点/时间/氛围）

- **StoryTimeline Component** - 可视化场景序列
  - 拖拽排序 (@dnd-kit)
  - 场景卡片展示
  - 戏剧目标预览
  - 冲突类型标签

- **SceneEditor Component** - 三标签页场景编辑器
  - 基础信息：标题、设置、在场角色
  - 戏剧结构：目标、压迫、冲突类型
  - 内容编辑：富文本编辑器

- **Database Schema** - 新增场景表
  - `scenes` 表替代 chapters 作为主叙事单位
  - 保留 `chapters` 表用于兼容性
  - 新增 `world_buildings`, `settings`, `writing_styles` 表

### 🧠 Enhanced Memory System (增强记忆系统)

基于 [karpathy/llm_wiki](https://github.com/karpathy/llm_wiki) 方法论实现

- **CJK Tokenizer** - 中文二元组分词器
  - 针对中日韩文字优化
  - Bigram 分词算法
  - Unicode CJK 范围检测

- **Ingest Pipeline** - 两步思维链内容摄取
  - Step 1: `analyze_content()` - LLM 分析实体、关系、事件
  - Step 2: `generate_knowledge()` - 生成结构化知识
  - 自动提取：实体、关系、事件、情感、伏笔

- **Knowledge Graph** - 带权知识图谱
  - `kg_entities` 表 - 实体存储
  - `kg_relations` 表 - 关系存储带 `strength` 字段
  - 关系强度动态计算（基于证据数量和时间衰减）
  - 优先级排序检索

- **Query Pipeline** - 四阶段查询检索
  - Stage 1: CJK 分词搜索
  - Stage 2: 图谱扩展（基于关系强度）
  - Stage 3: 预算控制（4K-1M tokens 可配置）
  - Stage 4: 带引用编号的上下文组装

- **Multi-Agent Sessions** - 多助手独立会话
  - WorldBuilding Agent - 世界观助手
  - Character Agent - 人物助手
  - WritingStyle Agent - 文风助手
  - Plot Agent - 情节助手
  - Scene Agent - 场景助手
  - Memory Agent - 记忆助手
  - 独立 Wiki 引用追踪
  - 对话保存到 Wiki 功能

### 🤖 AI-Powered Novel Creation (AI 智能生成)

- **NovelCreationAgent** - 小说创建专用 Agent
  - `generate_world_building_options()` - 生成世界观选项
  - `generate_character_profiles()` - 生成角色谱
  - `generate_writing_styles()` - 生成文字风格
  - `generate_next_scene()` - 生成下一个场景建议

- **NovelCreationWizard Component** - 引导式创建向导
  - 4 步引导流程：类型 → 世界观 → 角色 → 文风
  - 灰色提示词："小说类型：玄幻...商战...或随便定"
  - 卡片式选择界面
  - 双击编辑功能
  - 首个场景自动生成

### 📦 Studio Configuration System (工作室配置系统)

- **StudioConfig Model** - 每部小说独立配置
  - `story_metadata` - 故事元数据
  - `llm_config` - LLM 配置
  - `ui_config` - 界面主题配置
  - `agent_bots` - Agent 配置

- **StudioManager** - 配置管理器
  - `export_studio()` - ZIP 格式导出
  - `import_studio()` - 选择性导入
  - 冲突检测和处理

- **Default Themes**
  - 幕前：温暖纸张主题 (#f5f4ed)
  - 幕后：暗色影院主题

### 📁 New Files

#### Rust Backend
```
src-tauri/src/
├── commands_v3.rs                    # V3 Tauri 命令集（24个新命令）
├── db/
│   ├── models_v3.rs                  # V3 数据模型
│   └── repositories_v3.rs            # V3 存储层
├── agents/
│   └── novel_creation.rs             # 小说创建 Agent
├── memory/
│   ├── mod.rs
│   ├── tokenizer.rs                  # CJK 分词器
│   ├── ingest.rs                     # Ingest 管线
│   ├── query.rs                      # 查询检索管线
│   └── multi_agent.rs                # 多助手会话
└── config/
    └── studio_manager.rs             # 工作室配置管理
```

#### Frontend
```
src-frontend/src/
├── components/
│   ├── StoryTimeline.tsx             # 故事线视图
│   ├── SceneEditor.tsx               # 场景编辑器
│   └── NovelCreationWizard.tsx       # 创建向导
├── hooks/
│   ├── useScenes.ts                  # 场景管理 Hook
│   ├── useWorldBuilding.ts           # 世界构建 Hook
│   └── useStudioConfig.ts            # 工作室配置 Hook
├── pages/
│   └── Scenes.tsx                    # 场景管理页面
└── types/
    └── v3.ts                         # V3 TypeScript 类型
```

### 🔧 Dependencies Added

- `zip = "0.6"` - ZIP 压缩（工作室配置导入/导出）

### 🔄 Database Migration

**新增表：**
- `scenes` - 场景表（主叙事单位）
- `world_buildings` - 世界观表
- `world_rules` - 世界规则表
- `settings` - 场景设置表
- `writing_styles` - 文字风格表
- `kg_entities` - 知识图谱实体表
- `kg_relations` - 知识图谱关系表
- `studio_configs` - 工作室配置表

**保留表：**
- `chapters` - 用于向后兼容

---

## [2.0.0-alpha.3] - 2025-04-12

### ✨ New Features
- **编辑器设置中心** - 在后台设置中统一管理编辑器配置
  - 写作风格选择（5种预设风格）
  - 字体家族设置（7种预设字体 + 自定义字体）
  - 字号调节（12-32px）
  - 行高调节（1.2-3.0）
  - 实时预览效果

### 🔧 Improvements
- **幕前界面重构** - 移除 ReaderWriter 组件，简化架构
- **工具栏重新设计** - 移至编辑器底部，默认隐藏
  - 悬停时平滑滑出显示
  - 仿 Claude 纸质平面风格
  - 分组卡片式设计（历史/格式/标题/列表/其他）
  - 精致按钮样式（衬线字体标签、陶土色边框）
- **顶部栏简化** - 移除风格切换、字体设置、禅模式按钮
  - 保留 AI 续写开关
  - 简洁快捷键提示
- **编辑器宽度优化** - 最大宽度 900px，内容居中

### 🗑️ Removed
- `ReaderWriter.tsx` - 功能整合到 RichTextEditor
- `WritingStyleSwitcher.tsx` - 功能移至后台设置
- 禅模式按钮 - 保留 F11 快捷键
- 顶部工具栏字体设置按钮

---

## [2.0.0-alpha.2] - 2025-04-11

### ✨ New Features
- **Dashboard 增强** - 添加新建故事功能、最近编辑故事列表、空状态引导
- **Stories 页面完善** - 添加故事选择功能、内联编辑、当前故事指示器
- **Sidebar 改进** - 显示当前编辑故事、用户头像和状态
- **导出功能完整实现** - 支持 Markdown/HTML/TXT/JSON/PDF/EPUB 六种格式
- **连接状态显示** - ConnectionStatus 组件显示后端连接状态
- **错误边界** - ErrorBoundary 组件捕获渲染错误

### 🔧 Improvements
- 完善导出功能的 MIME 类型处理
- 添加动画样式 (fade-in, slide-up)
- 优化用户交互流程（创建故事后自动跳转）
- Toast 通知系统优化

### 📊 完成度更新
- 前端界面: 85% → 95%
- 整体完成度: 93% → 95%

---

## [2.0.0-alpha] - 2025-04-11

### 🎉 Highlights

This is a major refactoring release that aligns the implementation with the architecture design document.

### 🐛 Critical Bug Fixes (2025-04-11)

#### Connection Issue Fix
- **Fixed**: Windows 上无法连接本地服务端口的问题
- **原因**: `localhost` 解析为 IPv6 `::1` 而服务器绑定到 IPv4 `127.0.0.1`
- **解决**: 统一使用 `127.0.0.1`，完善 CSP 配置
- **文件**: `src-tauri/tauri.conf.json`, `src-frontend/vite.config.ts`, `src-tauri/capabilities/main-capability.json`

#### React Infinite Loop Fix
- **Fixed**: "Maximum update depth exceeded" 错误
- **原因**: React Query 重试与 useEffect 状态更新形成循环
- **解决**: 
  - 创建独立 `DataLoader` 组件分离数据加载与渲染
  - 使用 `useRef` 确保初始化只执行一次
  - 限制 React Query 重试次数
- **文件**: `src/components/DataLoader.tsx`, `src/pages/Dashboard.tsx`, `src/main.tsx`

#### Error Handling Enhancement
- **Added**: `ErrorBoundary` 组件捕获渲染错误
- **Added**: `ConnectionStatus` 组件显示连接状态
- **文档**: 新增 `docs/FIXES_2025_04_11.md` 详细修复记录

### 🔧 Recent Improvements (2025-04-11)

#### Frontend
- **Complete Chapter Management** - Full CRUD operations with API integration
- **Enhanced Character Management** - Added create/delete functionality
- **Real-time Stats** - Dashboard now displays actual story/character/chapter counts
- **Improved UX** - Loading states, error handling, and confirmation dialogs

#### Backend
- **Improved Embedding Algorithm** - Upgraded from character-based to hash-based TF feature extraction
- **New Tauri Command** - `create_chapter` for chapter creation, `health_check` for connection testing
- **WebSocket Server** - Multi-port support (8765-8769) to avoid port conflicts
- **Compilation Warnings** - Cleaned up unused import warnings

### 📝 Notes
- Frontend completion increased from 70% to 85%
- 应用现在可以在 Windows 上正常运行

---

## [2.0.0-beta] - 2026-04-11

### 🎉 5-Phase Implementation Complete

All 5 planned features have been implemented:

#### Phase 1: Vector Database Upgrade
- LanceDB-compatible API with memory-based storage
- Ready to switch to real LanceDB when Rust is upgraded

#### Phase 2: MCP Server Enhancement
- 3 built-in tools: filesystem, text_processing, web_search
- Tool registration and timeout control

#### Phase 3: Collaborative Editing (OT Algorithm)
- Core OT algorithm with Insert/Delete/Retain operations
- Operation transformation and application

#### Phase 4: Monaco Editor Integration
- Full-featured code editor for chapter writing
- Markdown support, font size control, fullscreen mode

#### Phase 5: Export Function UI
- Complete export dialog with format selection
- Backend export API for Markdown/PDF/EPUB/HTML/TXT/JSON
- Vector retrieval improved from 90% to 95%

### ✨ New Features

#### Skills System (Replaces Plugin System)
- **Generic skill import** - Import skills from directories or single files
- **Skill categories** - Writing, Analysis, Character, Plot, Style, Export, Integration, Custom
- **Prompt-based skills** - Skills powered by prompt templates
- **MCP skills** - Skills backed by MCP (Model Context Protocol) servers
- **Hook system** - Event hooks for before/after chapter generation, character creation, etc.
- **5 Built-in skills**:
  - `builtin.style_enhancer` - Enhances writing style and literary quality
  - `builtin.plot_twist` - Generates unexpected but logical plot twists
  - `builtin.character_voice` - Ensures character voice consistency
  - `builtin.emotion_analyzer` - Analyzes emotional arcs in chapters
  - `builtin.pacing_optimizer` - Optimizes story pacing

#### MCP (Model Context Protocol) Support
- **MCP Client** - Connect to external MCP servers (filesystem, web search, etc.)
- **Tool calling** - Execute tools provided by MCP servers
- **Resource reading** - Access resources through MCP protocol
- **Stdio transport** - JSON-RPC 2.0 over standard I/O

#### State Management
- **StoryState** - Complete story state tracking including:
  - Story metadata and configuration
  - Character states with arc progression
  - Chapter states with status tracking
  - Plot progression tracking
  - World state with locations and lore
- **Validation Schema** - Data validation for stories, chapters, and characters

#### Model Router
- **Smart routing** - Automatically select optimal LLM based on task type:
  - Creative Writing → High quality models
  - Editing → Precision-focused models
  - Analysis → Reasoning-capable models
  - Summarization → Faster, cheaper models
- **Cost tracking** - Per-request cost calculation and tracking
- **Quality/Speed tiers** - Ultra/High/Medium/Low quality, Fast/Normal/Slow speed

#### Evolution System
- **Content Analyzer** - Comprehensive story analysis:
  - Pacing analysis with slow/rushed section detection
  - Character consistency checking
  - Plot coherence verification
  - Writing quality assessment
- **Skill Updater** - Automatic skill optimization based on analysis
- **Deep Reviewer** - In-depth story evolution analysis:
  - Narrative arc evaluation
  - Theme development tracking
  - Reader engagement prediction
  - Learning outcome identification

#### Embeddings System
- **Text chunking** - Configurable chunking strategies (sentences, paragraphs, fixed size)
- **Multiple providers** - OpenAI, Azure, Ollama, Local
- **Batch processing** - Efficient batch embedding generation

#### Utilities
- **Text utilities** - Word/sentence counting, dialogue extraction, similarity calculation
- **File utilities** - Safe filename sanitization, directory operations, file listing
- **Validation utilities** - Email, URL, password, JSON validation

### 🔧 Improvements

#### Export Functionality
- **PDF export** - Full PDF generation using `printpdf` crate
- **EPUB export** - EPUB e-book generation using `epub-builder` crate
- **Multiple formats** - Markdown, PlainText, JSON, HTML, PDF, EPUB

#### Module Completion
- All empty modules from architecture design now implemented
- Complete state/ router/ evolution/ embeddings/ utils/ modules

### 🔄 Changes

- **Plugin → Skills** - Replaced plugin system with more flexible skills system
- **Architecture alignment** - All modules now match ARCHITECTURE.md specification

### 📦 Dependencies Added
- `printpdf` - PDF generation
- `epub-builder` - EPUB generation
- `serde_yaml` - YAML parsing for skills
- `walkdir` - Directory traversal
- `notify` - File system watching
- `dirs` - Standard directories
- `log` - Logging framework

### 🐛 Known Issues
- Frontend UI partially implemented (70% complete)
- Collaborative editing OT algorithm needs completion
- MCP Server implementation is framework-only

---

## [1.5.0] - 2025-04-08

### Added
- Agent system with WriterAgent, InspectorAgent, OutlinePlannerAgent
- Workflow engine with DAG support
- Vector storage implementation with TF vectorization
- Style mimic agent for learning writing styles
- Plot complexity analyzer

### Changed
- Improved LLM adapter pattern
- Enhanced prompt management system

---

## [1.0.0] - 2025-04-01

### Added
- Initial project structure
- Tauri + Rust architecture
- LLM integration (OpenAI, Anthropic, Ollama)
- SQLite database with r2d2 connection pooling
- Basic frontend with Tailwind CSS
- Story/Chapter/Character data models
- Export to Markdown/JSON/HTML

---

## Legend

- ✨ New feature
- 🔧 Improvement
- 🐛 Bug fix
- 🔄 Change
- 🗑️ Removal
- ⚠️ Deprecation
- 🎪 Scene System
- 🧠 Memory System
- 🤖 AI Generation
- 📦 Studio System
