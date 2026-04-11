# StoryForge (草苔) v2.0 架构文档

## 架构理念：幕前幕后双界面

StoryForge 采用创新的**剧院式双界面架构**：

- **幕前 (Frontstage)**: 沉浸式写作界面，如同登台演出
- **幕后 (Backstage)**: 专业工作室，如同后台准备

这种设计分离了"创作体验"与"管理操作"，让作者在不同场景下获得最佳体验。

---

## 系统架构图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        StoryForge (草苔) v2.0                             │
│                     Tauri + React + Rust + SQLite                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────┐        ┌─────────────────────────┐         │
│  │     🎭 幕前 Frontstage   │        │     🎬 幕后 Backstage    │         │
│  │    (沉浸式写作界面)      │        │    (专业工作室)          │         │
│  ├─────────────────────────┤        ├─────────────────────────┤         │
│  │                         │        │                         │         │
│  │  • 极简阅读写作界面      │◄──────►│  • 故事/角色/章节管理     │         │
│  │  • TipTap 富文本编辑器   │        │  • LLM 模型配置中心       │         │
│  │  • 章节大纲侧边栏        │        │  • Skills 技能系统        │         │
│  │  • AI 续写辅助          │        │  • MCP 扩展配置          │         │
│  │  • 写作风格切换          │        │  • 协同编辑管理          │         │
│  │  • 禅模式全屏           │        │  • 数据导出/分析          │         │
│  │  • 角色卡片弹窗          │        │                         │         │
│  │                         │        │                         │         │
│  │  暖色调 (#f5f4ed)        │        │  深色主题 (Cinema)       │         │
│  │  Claude 阅读体验设计     │        │  电影感专业界面          │         │
│  │                         │        │                         │         │
│  └──────────┬──────────────┘        └──────────┬──────────────┘         │
│             │                                   │                        │
│             └───────────────┬───────────────────┘                        │
│                             ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    Tauri Bridge (IPC)                            │   │
│  │           Commands + Events + Window Management                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                             │                                          │
│  ┌──────────────────────────┴──────────────────────────────────────┐   │
│  │                      Backend (Rust)                               │   │
│  ├─────────────────────────────────────────────────────────────────┤   │
│  │                                                                  │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │   │
│  │  │   Agents    │  │   Skills    │  │      LLM Adapter        │ │   │
│  │  │  ├─ Writer  │  │  ├─ Loader  │  │  ├─ OpenAI             │ │   │
│  │  │  ├─ Inspector│ │  │  ├─ Executor│ │  ├─ Anthropic         │ │   │
│  │  │  ├─ Planner │  │  ├─ Registry│  │  ├─ Ollama (本地)      │ │   │
│  │  │  ├─ Style   │  │  └─ Builtin │  │  └─ Azure/DeepSeek...  │ │   │
│  │  │  └─ Plot    │  │             │  │                         │ │   │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘ │   │
│  │                                                                  │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │   │
│  │  │   Memory    │  │   Config    │  │      Collaboration      │ │   │
│  │  │  ├─ Short   │  │  ├─ Settings│  │  ├─ WebSocket Server   │ │   │
│  │  │  ├─ Vector  │  │  ├─ Models  │  │  ├─ OT Algorithm       │ │   │
│  │  │  └─ Embed   │  │  └─ Export  │  │  └─ Cursor Sync        │ │   │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘ │   │
│  │                                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                             │                                          │
│  ┌──────────────────────────┴──────────────────────────────────────┐   │
│  │                      Data Layer                                   │   │
│  ├─────────────────────────────────────────────────────────────────┤   │
│  │                                                                  │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │   │
│  │  │   SQLite    │  │  LanceDB    │  │    File System          │ │   │
│  │  │  (r2d2池)   │  │  (向量检索)  │  │  • 技能库               │ │   │
│  │  │  • Stories  │  │  • 章节嵌入  │  │  • 导出文件             │ │   │
│  │  │  • Characters│ │  • 相似搜索  │  │  • 配置文件             │ │   │
│  │  │  • Chapters │  │             │  │                         │ │   │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘ │   │
│  │                                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 目录结构

```
v2-rust/
├── src-frontend/                 # 前端代码 (React + TypeScript)
│   ├── src/
│   │   ├── main.tsx             # 幕后入口
│   │   ├── App.tsx              # 幕后主应用
│   │   ├── frontstage/          # 幕前界面
│   │   │   ├── main.tsx         # 幕前入口
│   │   │   ├── FrontstageApp.tsx    # 幕前主组件
│   │   │   ├── components/      # 幕前组件
│   │   │   │   ├── ReaderWriter.tsx     # 阅读写作器
│   │   │   │   ├── RichTextEditor.tsx   # TipTap编辑器
│   │   │   │   ├── ChapterOutline.tsx   # 章节大纲
│   │   │   │   ├── CharacterCardPopup.tsx # 角色卡片
│   │   │   │   ├── WritingStyleSwitcher.tsx # 风格切换
│   │   │   │   ├── AiSuggestionBubble.tsx # AI提示气泡
│   │   │   │   └── ...
│   │   │   ├── hooks/           # 幕前专用hooks
│   │   │   ├── config/          # 配置文件
│   │   │   └── styles/          # 幕前样式
│   │   │
│   │   ├── pages/               # 幕后页面
│   │   │   ├── Dashboard.tsx    # 仪表盘
│   │   │   ├── Stories.tsx      # 故事库
│   │   │   ├── Characters.tsx   # 角色管理
│   │   │   ├── Chapters.tsx     # 章节管理
│   │   │   ├── Skills.tsx       # 技能管理
│   │   │   ├── Mcp.tsx          # MCP配置
│   │   │   └── Settings.tsx     # 设置中心
│   │   │
│   │   ├── components/          # 共享组件
│   │   │   ├── ui/              # UI组件库
│   │   │   ├── Sidebar.tsx      # 侧边栏
│   │   │   ├── Editor.tsx       # 编辑器
│   │   │   └── ...
│   │   │
│   │   ├── hooks/               # 共享hooks
│   │   ├── services/            # API服务
│   │   ├── stores/              # 状态管理
│   │   ├── types/               # 类型定义
│   │   └── utils/               # 工具函数
│   │
│   ├── index.html               # 幕后HTML
│   ├── frontstage.html          # 幕前HTML
│   └── package.json
│
├── src-tauri/                   # Tauri后端 (Rust)
│   ├── src/
│   │   ├── main.rs              # 入口
│   │   ├── lib.rs               # 库入口
│   │   ├── commands.rs          # 命令定义
│   │   │
│   │   ├── agents/              # Agent系统
│   │   │   ├── mod.rs           # Agent trait
│   │   │   ├── writer.rs        # 写作Agent
│   │   │   ├── inspector.rs     # 质检Agent
│   │   │   ├── outline_planner.rs # 大纲规划
│   │   │   ├── style_mimic.rs   # 风格模仿
│   │   │   └── plot_analyzer.rs # 情节分析
│   │   │
│   │   ├── skills/              # 技能系统
│   │   │   ├── mod.rs           # Skill管理
│   │   │   ├── loader.rs        # 技能加载
│   │   │   ├── executor.rs      # 技能执行
│   │   │   ├── registry.rs      # 技能注册
│   │   │   └── builtin.rs       # 内置技能
│   │   │
│   │   ├── llm/                 # LLM集成
│   │   │   ├── mod.rs           # 适配器接口
│   │   │   ├── openai.rs        # OpenAI实现
│   │   │   └── prompt.rs        # 提示词管理
│   │   │
│   │   ├── config/              # 配置管理
│   │   │   ├── settings.rs      # 设置结构
│   │   │   └── commands.rs      # 设置命令
│   │   │
│   │   ├── db/                  # 数据库
│   │   ├── vector/              # 向量检索
│   │   ├── collab/              # 协同编辑
│   │   ├── window/              # 窗口管理
│   │   ├── export/              # 导出功能
│   │   └── ...
│   │
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src-core/                    # 核心库 (可选)
├── docs/                        # 文档
└── README.md
```

---

## 幕前幕后通信机制

### 窗口管理
```rust
// src/window/mod.rs
pub struct WindowManager;

impl WindowManager {
    // 发送事件到幕前
    pub fn send_to_frontstage(app: &AppHandle, event: FrontstageEvent);
    
    // 发送事件到幕后
    pub fn send_to_backstage(app: &AppHandle, event: BackstageEvent);
    
    // 窗口切换
    pub fn show_frontstage(app: &AppHandle);
    pub fn show_backstage(app: &AppHandle);
}
```

### 事件类型
```rust
// 幕前事件
pub enum FrontstageEvent {
    ContentUpdate { text: String, chapter_id: String },
    AiHint { hint: String, position: Position },
    ChapterSwitch { chapter_id: String },
}

// 幕后事件
pub enum BackstageEvent {
    ContentChanged { text: String, chapter_id: String },
    GenerationRequested { chapter_id: String, context: String },
}
```

---

## 数据流

### 写作流程
```
用户输入 → ReaderWriter → TipTap Editor → invoke('update_chapter') 
→ Rust后端 → SQLite → 自动保存指示器更新
```

### AI 生成流程
```
Ctrl+Space → handleRequestGeneration → (模拟/LLM API) 
→ StreamingText → 用户接受(Tab)/拒绝(Esc) → 插入内容
```

### 设置同步流程
```
Settings页面修改 → useSettings Hook → invoke('save_settings') 
→ Rust config → config.json → 其他窗口读取
```

---

## 状态管理策略

### 前端状态 (Zustand)
- `appStore`: 全局应用状态 (当前视图、用户、加载状态)
- `currentStory`: 当前选中的故事上下文
- React Query: 服务端状态缓存

### 后端状态 (Rust)
- `DB_POOL`: SQLite 连接池 (全局单例)
- `APP_CONFIG`: 应用配置 (延迟加载)
- `SKILL_MANAGER`: 技能管理器 (初始化加载)
- `VECTOR_STORE`: 向量数据库 (异步初始化)

---

## 扩展机制

### 1. Skills 技能系统
```rust
pub trait SkillHandler: Send + Sync {
    fn execute(&self, context: &AgentContext, params: HashMap<...>) 
        -> Result<SkillResult, Error>;
}
```

支持三种运行时：
- **Prompt**: 系统提示词 + 用户模板
- **MCP**: 连接外部 MCP Server
- **Native**: 原生 Rust 实现

### 2. MCP 扩展
```rust
pub struct McpServerConfig {
    pub command: String,      // 可执行文件
    pub args: Vec<String>,    // 参数
    pub env: HashMap<...>,    // 环境变量
}
```

### 3. LLM 适配器
```rust
#[async_trait]
pub trait LlmAdapter: Send + Sync {
    async fn generate(&self, prompt: &str, config: &Config) -> Result<String>;
    async fn stream_generate(&self, prompt: &str, callback: Fn) -> Result<()>;
}
```

---

## 性能优化

### 前端
- **懒加载**: 幕前/幕后代码分割
- **虚拟列表**: 章节大纲长列表优化
- **防抖**: 自动保存 2 秒延迟
- **增量更新**: TipTap onUpdate 精确触发

### 后端
- **连接池**: r2d2 SQLite 连接复用
- **异步**: Tokio 运行时处理 I/O
- **缓存**: 向量索引内存缓存
- **并行**: Agent 并行执行

---

## 安全考虑

1. **API Key**: 本地存储，界面显示为 `***`
2. **文件访问**: Tauri 能力限制 (capabilities)
3. **SQL 注入**: 参数化查询
4. **XSS**: TipTap 内容转义
5. **CORS**: 仅允许本地请求

---

## 开发指南

### 启动开发服务器
```bash
# 前端开发
npm run dev          # Vite dev server

# 后端开发
cargo tauri dev      # Tauri dev mode

# 完整开发
npm run tauri dev    # 同时启动前后端
```

### 构建生产版本
```bash
npm run build        # 前端生产构建
cargo build --release  # Rust 生产构建
npm run tauri build  # 完整应用打包
```

---

## 未来演进

### 短期 (v2.1)
- Agent 核心逻辑实现
- AI 生成对接真实 LLM
- 角色名智能识别
- 移动端适配

### 中期 (v2.2)
- 云端同步
- 版本历史
- 插件市场
- 团队协作增强

### 长期 (v3.0)
- WASM 前端重写 (Leptos)
- 自研小模型部署
- 多人实时协作
- 跨平台移动端
