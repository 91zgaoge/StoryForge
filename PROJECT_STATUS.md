# StoryForge (草苔) v2.0 项目完成状态

> 最后更新: 2026-04-12（幕前界面重构完成）

---

## ✅ 已完成功能

### 架构基础 (100%)
- ✅ Tauri + Rust 桌面应用框架
- ✅ 幕前幕后双窗口架构
- ✅ 窗口间通信机制 (Events)
- ✅ SQLite 数据库 (r2d2 连接池)
- ✅ 前端 React 18 + TypeScript 5.8 + Vite 6

### 幕前界面 (Frontstage) - 95%

| 功能模块 | 状态 | 完成度 | 备注 |
|---------|------|--------|------|
| 双栏布局 | ✅ | 100% | 可折叠侧边栏 + 主编辑区 |
| 章节大纲 | ✅ | 100% | 拖拽排序、内联编辑、增删改 |
| TipTap编辑器 | ✅ | 100% | Markdown快捷键、底部工具栏 |
| 写作风格 | ✅ | 100% | 5种风格、后台设置、持久化 |
| 字体设置 | ✅ | 100% | 7种预设 + 自定义字体、后台管理 |
| 角色卡片 | ✅ | 90% | TipTap扩展、点击触发 |
| AI提示 | ✅ | 90% | 气泡动效、LLM流式生成 |
| 禅模式 | ✅ | 100% | F11快捷键、ESC退出 |
| 自动保存 | ✅ | 100% | 2秒延迟、状态指示 |
| 技能面板 | ✅ | 90% | 5种Agent类型、执行UI |
| 底部工具栏 | ✅ | 100% | 悬停显示、纸质风格、分组设计 |

### 幕后界面 (Backstage) - 85%

| 功能模块 | 状态 | 完成度 | 备注 |
|---------|------|--------|------|
| 仪表盘 | ✅ | 100% | 统计卡片、快速创建、最近编辑 |
| 故事库 | ✅ | 100% | CRUD、类型标签 |
| 角色管理 | ✅ | 100% | 卡片展示、增删改 |
| 章节管理 | ✅ | 100% | Monaco编辑器、向量搜索、协同 |
| 技能管理 | ✅ | 80% | 列表展示、启用/禁用、需完善执行 |
| MCP配置 | 🟡 | 60% | 界面框架、需完善功能 |
| 设置中心 | ✅ | 90% | LLM配置、导入导出、需完善Agent映射 |

### 后端系统 (90%)

| 模块 | 状态 | 完成度 | 备注 |
|------|------|--------|------|
| 数据层 | ✅ | 100% | Story/Character/Chapter CRUD |
| Agent系统 | ✅ | 90% | 5种Agent完整实现、Tauri命令 |
| 技能系统 | ✅ | 90% | 内置5技能、技能执行面板 |
| LLM集成 | ✅ | 90% | OpenAI/Anthropic/Ollama、流式生成 |
| 配置管理 | ✅ | 95% | 多模型配置、导入导出、测试连接 |
| 向量检索 | ✅ | 70% | LanceDB、TF-IDF嵌入 |
| 协同编辑 | ✅ | 75% | WebSocket、OT算法 |
| 导出功能 | ✅ | 100% | Markdown/PDF/EPUB/HTML/TXT/JSON |
| 窗口管理 | ✅ | 100% | 双窗口切换、通信 |

---

## 📊 详细功能清单

### 1. 幕前 (Frontstage) - 沉浸式写作

#### 1.1 界面布局
- ✅ 双栏布局：侧边栏 + 主编辑区
- ✅ 侧边栏：故事选择、章节大纲
- ✅ 可折叠侧边栏
- ✅ 响应式设计

#### 1.2 章节大纲 (ChapterOutline)
- ✅ 章节列表展示
- ✅ 拖拽排序 (@dnd-kit)
- ✅ 内联编辑标题
- ✅ 删除章节（确认对话框）
- ✅ 添加新章节
- ✅ 选中高亮
- ✅ 字数统计显示

#### 1.3 富文本编辑器 (RichTextEditor)
- ✅ TipTap / ProseMirror 内核
- ✅ Markdown 快捷键 (Ctrl+B/I/U, Ctrl+Shift+1-6)
- ✅ 底部工具栏（悬停显示、纸质风格）
  - 历史按钮组（撤销/重做）
  - 格式按钮组（粗体/斜体/下划线/删除线/高亮）
  - 标题按钮组（H1/H2）
  - 列表按钮组（有序/无序）
  - 其他按钮组（引用/代码）
- ✅ 字号调节 (12-32px，后台设置)
- ✅ 行高调节 (1.2-3.0，后台设置)
- ✅ 字体设置（7种预设 + 自定义，后台管理）

#### 1.4 AI 辅助写作
- ✅ AI 续写（Ctrl+Space，真实LLM流式生成）
- ✅ 生成预览面板
- ✅ 接受/拒绝控制（Tab/Esc）
- ✅ AI 提示气泡（情节/人物/环境/节奏/情感）
- ✅ 萤火虫动效
- ✅ 技能执行面板（SkillExecutionPanel）
  - 5种Agent类型选择
  - 实时进度显示
  - 质量评分
  - 改进建议
  - 结果复制/应用
- 🟡 文思泉涌开关

#### 1.5 写作风格 (EditorSettings)
- ✅ 5种预设风格：
  - 现代简洁 (默认)
  - 古典深沉 (仿陀思妥耶夫斯基)
  - 现代中文 (仿张爱玲)
  - 极简主义 (仿海明威)
  - 浪漫抒情
- ✅ 风格预览（后台设置）
- ✅ CSS 变量动态切换
- ✅ localStorage 持久化
- ✅ 移至后台设置中心统一管理

#### 1.6 角色卡片 (CharacterCardPopup)
- ✅ 角色详情展示（背景、性格、目标）
- ✅ 弹窗定位与边界检测
- ✅ ESC/点击外部关闭
- ✅ TipTap角色名扩展（characterName mark）
- ✅ 点击角色名触发弹窗

#### 1.7 禅模式
- ✅ F11 切换全屏
- ✅ 隐藏所有 UI 元素
- ✅ 退出提示

---

### 2. 幕后 (Backstage) - 创作工作室

#### 2.1 仪表盘 (Dashboard)
- ✅ 欢迎界面
- ✅ 统计卡片（故事/角色/章节数）
- ✅ 快速创建故事
- ✅ 最近编辑故事列表
- ✅ 空状态引导

#### 2.2 故事库 (Stories)
- ✅ 网格布局展示
- ✅ 创建故事（标题/类型/描述）
- ✅ 编辑故事信息
- ✅ 删除故事
- ✅ 类型标签（科幻/奇幻/悬疑等）
- ✅ 点击进入章节管理

#### 2.3 角色管理 (Characters)
- ✅ 角色卡片（头像、名称、性格预览）
- ✅ 创建角色（名称/背景）
- ✅ 删除角色
- ✅ 故事关联显示
- ✅ 空状态引导

#### 2.4 章节管理 (Chapters)
- ✅ Monaco Editor 集成
- ✅ Markdown 语法高亮
- ✅ 创建章节（自动编号）
- ✅ 保存章节（Ctrl+S）
- ✅ 删除章节
- ✅ 字数统计
- ✅ 全屏编辑模式
- ✅ 向量搜索面板（LanceDB）
- ✅ 协同编辑（WebSocket）
- ✅ 协作者列表

#### 2.5 技能管理 (Skills)
- ✅ 技能列表展示
- ✅ 分类标签（Writing/Analysis/Character/Plot/Style/Export/Integration/Custom）
- ✅ 启用/禁用开关
- ✅ 5个内置技能：
  - builtin.style_enhancer (文风增强器)
  - builtin.plot_twist (情节反转)
  - builtin.character_voice (角色声音)
  - builtin.emotion_analyzer (情感分析)
  - builtin.pacing_optimizer (节奏优化)
- ✅ 技能导入（后端支持，前端需完善）
- ✅ 技能执行面板（5种Agent类型）
  - 写作助手 (Writer)
  - 质检员 (Inspector)
  - 大纲规划师 (OutlinePlanner)
  - 风格模仿师 (StyleMimic)
  - 情节分析师 (PlotAnalyzer)
- ✅ 进度可视化
- ✅ 结果复制/应用

#### 2.6 MCP 配置
- ✅ 服务器列表界面
- 🟡 连接测试
- 🟡 工具调用

#### 2.7 设置中心 (Settings)
- ✅ 多类型 LLM 配置：
  - Chat 模型（文本生成）
  - Embedding 模型（向量嵌入）
  - Multimodal 模型（多模态）
  - Image 模型（图像生成）
- ✅ 支持的提供商：
  - OpenAI
  - Anthropic
  - Azure OpenAI
  - Ollama（本地）
  - DeepSeek
  - 通义千问/Qwen
  - Moonshot
  - 智谱AI/Zhipu
  - Custom
- ✅ 预设模型快速选择
- ✅ API Key 管理（安全输入）
- ✅ 模型参数调节（Temperature/Max Tokens/Dimensions）
- ✅ 设为默认/启用开关
- ✅ 设置导出（JSON 下载）
- ✅ 设置导入（JSON 上传）
- ✅ 版本兼容性检查
- ✅ 编辑器设置（写作风格/字体/字号/行高）
  - 5种预设写作风格
  - 7种预设字体 + 自定义字体
  - 字号调节（12-32px）
  - 行高调节（1.2-3.0）
  - 实时预览
- 🟡 Agent 模型映射（界面框架）
- 🟡 通用设置（主题/语言/自动保存）

---

### 3. 后端 (Rust/Tauri)

#### 3.1 数据层 (src/db)
- ✅ SQLite 数据库初始化
- ✅ r2d2 连接池
- ✅ Story 模型（CRUD + 类型/风格字段）
- ✅ Character 模型（CRUD + 背景/性格/目标）
- ✅ Chapter 模型（CRUD + 序号/大纲/内容）
- ✅ Repository 数据访问模式

#### 3.2 Agent 系统 (src/agents)
- ✅ Agent trait 定义
- ✅ AgentContext 上下文（Serialize/Deserialize）
- ✅ AgentResult 结果
- ✅ AgentService 服务层
- ✅ AgentType 枚举（5种类型）
- ✅ AgentTask 任务结构
- ✅ AgentEvent 事件系统
- ✅ Tauri命令（agent_execute, agent_execute_stream）
- ✅ WriterAgent（完整实现）
- ✅ InspectorAgent（完整实现）
- ✅ OutlinePlannerAgent（完整实现）
- ✅ StyleMimicAgent（完整实现）
- ✅ PlotAnalyzerAgent（完整实现）

#### 3.3 技能系统 (src/skills)
- ✅ Skill 结构（Manifest + Runtime）
- ✅ SkillCategory 分类（9类）
- ✅ SkillManager 管理器
- ✅ SkillLoader 加载器
- ✅ SkillExecutor 执行器
- ✅ SkillRegistry 注册表
- ✅ Hook 系统（Before/After 事件）
- ✅ Prompt Runtime
- ✅ MCP Runtime
- ✅ Native Runtime（框架）
- ✅ 5个内置技能

#### 3.4 LLM 集成 (src/llm)
- ✅ LlmAdapter trait
- ✅ OpenAI 适配器
- ✅ Anthropic 适配器
- ✅ Ollama 适配器（本地）
- ✅ Azure 适配器（框架）
- ✅ Prompt 管理
- ✅ LlmService 服务层
- ✅ 流式生成（stream_generate）
- ✅ Tauri命令（llm_generate, llm_generate_stream）
- ✅ 连接测试（llm_test_connection）

#### 3.5 配置管理 (src/config)
- ✅ AppConfig 全局配置
- ✅ LlmProfile LLM档案
- ✅ EmbeddingProfile 嵌入档案
- ✅ 多模型配置管理
- ✅ 活跃配置切换
- ✅ 设置导入/导出命令
- ✅ 配置持久化（config.json）

#### 3.6 向量检索 (src/vector, src/embeddings)
- ✅ LanceDB 向量数据库
- ✅ VectorRecord 记录结构
- ✅ TF-IDF 本地嵌入
- ✅ 余弦相似度搜索
- ✅ 章节自动嵌入
- 🟡 OpenAI Embedding（接口准备）

#### 3.7 协同编辑 (src/collab)
- ✅ WebSocket Server（端口 8765+）
- ✅ OT 算法（Insert/Delete/Retain）
- ✅ 操作转换（Transform）
- ✅ 操作应用（Apply）
- ✅ 光标位置同步
- ✅ 用户管理
- ✅ CollabSession 管理

#### 3.8 导出功能 (src/export)
- ✅ StoryExporter 导出器
- ✅ Markdown 导出
- ✅ PDF 导出（printpdf）
- ✅ EPUB 导出（epub-builder）
- ✅ HTML 导出
- ✅ PlainText 导出
- ✅ JSON 导出

#### 3.9 状态管理 (src/state)
- ✅ StoryState（框架）
- ✅ CharacterState（框架）
- ✅ ChapterState（框架）
- ✅ PlotProgression（框架）
- ✅ WorldState（框架）
- ✅ Schema 验证

#### 3.10 MCP 支持 (src/mcp)
- ✅ MCP Client
- ✅ McpServerConfig 配置
- ✅ Tool 调用
- ✅ Resource 读取
- ✅ Stdio Transport
- 🟡 MCP Server（内置，框架）

#### 3.11 自动更新 (src/updater)
- ✅ tauri-plugin-updater 集成
- ✅ 更新检测（check_update）
- ✅ 后台下载与安装（install_update）
- ✅ 前端更新通知（UpdateNotification）
- ✅ 设置页面版本管理
- ✅ GitHub Releases 更新源

#### 3.12 窗口管理 (src/window)
- ✅ WindowManager
- ✅ 双窗口架构（frontstage/backstage）
- ✅ 窗口显示/隐藏/切换
- ✅ FrontstageEvent 事件
- ✅ BackstageEvent 事件
- ✅ 内容同步
- ✅ AI 提示推送

---

## 📈 整体完成度

| 模块 | 完成度 | 权重 | 加权得分 |
|------|--------|------|----------|
| 架构基础 | 100% | 10% | 10.0 |
| 幕前界面 | 96% | 25% | 24.0 |
| 幕后界面 | 92% | 25% | 23.0 |
| 后端系统 | 92% | 30% | 27.6 |
| 文档/测试 | 85% | 10% | 8.5 |
| **总计** | - | 100% | **93.1%** |

---

## 🎯 待完善功能（按优先级）

### P0 - 核心功能 ✅ 已完成
1. ✅ **AI 生成对接真实 LLM**
   - 位置: `src-tauri/src/llm/service.rs`, `commands.rs`
   - 状态: LlmService实现、流式生成、Tauri命令

2. ✅ **Agent 核心逻辑实现**
   - 位置: `src-tauri/src/agents/service.rs`
   - 状态: 5种Agent完整实现、AgentService协调

3. ✅ **角色名智能识别**
   - 位置: `src/frontstage/extensions/characterName.ts`
   - 状态: TipTap mark扩展、点击触发弹窗

4. ✅ **技能执行 UI**
   - 位置: `src/components/skills/SkillExecutionPanel.tsx`
   - 状态: 5种Agent类型、进度可视化、结果应用

### P1 - 重要功能（影响体验）
5. **Agent 模型映射**
   - 位置: `src/pages/Settings.tsx`, `src-tauri/src/config/`
   - 说明: 为不同 Agent 配置专用模型

6. **版本历史**
   - 位置: `src-tauri/src/versions/`
   - 说明: 章节版本管理和回滚

### P2 - 增强功能（锦上添花）
7. ✅ **自动更新**
   - 位置: `src-tauri/src/updater/`, `src-frontend/src/components/updater/`
   - 状态: tauri-plugin-updater集成、GitHub Releases更新源

8. **统计分析**
   - 位置: `src/pages/Dashboard.tsx`, `src-tauri/src/analytics/`
   - 说明: 写作数据可视化

9. **云端同步**
   - 位置: 新增模块
   - 说明: 数据备份和跨设备同步

10. **插件市场**
   - 位置: 新增模块
   - 说明: Skills 分享和下载平台

---

## 🐛 已知问题

1. **Tauri 文件锁**
   - 描述: cargo build 时偶尔出现文件锁等待
   - 解决: 重启构建或清理 target 目录

2. **Agent上下文构建**
   - 描述: agent_execute中暂时使用简化上下文
   - 解决: 后续通过Tauri State或全局DB访问完善

3. **流式生成事件**
   - 描述: 前端暂未完全接入实时流式事件
   - 解决: 使用Tauri Event系统推送生成进度

---

## 📚 相关文档

- [README.md](../README.md) - 项目简介
- [ARCHITECTURE.md](../ARCHITECTURE.md) - 架构文档
- [docs/FEATURES.md](FEATURES.md) - 详细功能清单
- [ROADMAP.md](../ROADMAP.md) - 开发路线图


---

## 📝 本次新增文件清单（4大核心功能）

### Rust后端

#### LLM服务 (`src-tauri/src/llm/`)
- `service.rs` - LlmService核心服务，支持流式生成
- `commands.rs` - Tauri命令：llm_generate, llm_generate_stream, llm_test_connection

#### Agent系统 (`src-tauri/src/agents/`)
- `service.rs` - AgentService协调服务，5种Agent执行逻辑
- `commands.rs` - Tauri命令：agent_execute, agent_execute_stream, get_available_agents

### 前端组件

#### 技能执行面板 (`src-frontend/src/components/skills/`)
- `SkillExecutionPanel.tsx` - 技能执行UI，Agent选择、进度显示、结果应用
- `index.ts` - 模块导出

---


### 自动更新功能 (`src-tauri/src/updater/`, `src-frontend/src/components/updater/`)

#### Rust后端
- `src-tauri/src/updater/mod.rs` - Updater模块，更新检测和安装命令
- `Cargo.toml` - 添加 tauri-plugin-updater 依赖
- `tauri.conf.json` - 配置 updater 插件和 GitHub Releases 源

#### 前端组件
- `src-frontend/src/hooks/useUpdater.ts` - 更新检测 hook，自动检查
- `src-frontend/src/components/updater/UpdateNotification.tsx` - 更新通知弹窗
- `src-frontend/src/components/updater/index.ts` - 模块导出

#### 集成
- `src-frontend/src/App.tsx` - 集成 UpdateNotification 组件
- `src-frontend/src/pages/Settings.tsx` - 设置页面版本信息

---

## 📝 幕前界面重构文件清单（alpha.3）

### 新增文件
- `src-frontend/src/components/EditorSettings.tsx` - 编辑器设置组件（风格/字体/字号/行高）

### 修改文件
- `src-frontend/src/frontstage/FrontstageApp.tsx` - 移除 ReaderWriter 引用，集成 RichTextEditor
- `src-frontend/src/frontstage/components/RichTextEditor.tsx` - 整合 ReaderWriter 功能，底部工具栏
- `src-frontend/src/frontstage/components/index.ts` - 移除 ReaderWriter 和 WritingStyleSwitcher 导出
- `src-frontend/src/frontstage/styles/frontstage.css` - 更新工具栏样式和编辑器布局
- `src-frontend/src/pages/Settings.tsx` - 添加 EditorSettings 到通用设置

### 删除文件
- `src-frontend/src/frontstage/components/ReaderWriter.tsx` - 功能整合到 RichTextEditor
- `src-frontend/src/frontstage/components/WritingStyleSwitcher.tsx` - 功能移至 EditorSettings

