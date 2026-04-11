# StoryForge (草苔) v2.0 架构文档

## 技术栈转型说明

### 从 TS/Python 混合架构 → Rust + Tauri 统一架构

| 原架构 | 新架构 | 优势 |
|--------|--------|------|
| TypeScript (open-multi-agent) | Rust (原生性能) | 10-100倍性能提升，内存安全 |
| Python (hermes-agent) | Rust (KalosM/ollama-rs) | 统一语言，零成本跨语言调用 |
| Node.js 运行时 | Tauri (Rust核心) | 15MB 安装包 vs 200MB+ Electron |
| React/Vue 可选 | Leptos/Sycamore + Tailwind | 编译时优化，WASM 前端 |
| 多进程 IPC | Rust 异步单进程 | 简化架构，消除序列化开销 |

## 新架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                    StoryForge (草苔) Desktop App                       │
│                      (Tauri + Tailwind)                        │
├─────────────────────────────────────────────────────────────────┤
│  Frontend (WebView)           │  Backend (Rust)                │
│  ┌─────────────────────────┐  │  ┌─────────────────────────┐   │
│  │  UI Layer               │  │  │  Core Logic Layer       │   │
│  │  - Leptos/WASM          │  │  │  - State Manager        │   │
│  │  - Tailwind CSS         │  │  │  - Model Router         │   │
│  │  - Monaco Editor        │  │  │  - DAG Executor         │   │
│  │  - ReactFlow (可视化)    │  │  │  - Agent System         │   │
│  └─────────────────────────┘  │  └─────────────────────────┘   │
│              ↑                │              ↑                  │
│         Tauri Bridge          │         Tokio Runtime           │
│    (JSON RPC / Events)        │    (Async + Parallel)           │
├─────────────────────────────────────────────────────────────────┤
│                      Data Layer                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │  SQLite     │  │  LanceDB    │  │  File System            │ │
│  │  (状态/元数据) │  │  (向量检索)  │  │  (章节内容/技能库)       │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    ▼                   ▼
              ┌──────────┐       ┌──────────┐
              │  Local   │       │  Remote  │
              │  Ollama  │       │  APIs    │
              │  Llama   │       │ OpenAI   │
              └──────────┘       │ Claude   │
                                 └──────────┘
```

## 核心模块职责

### 1. Frontend (src-ui/)
- **框架**: Leptos (Rust WASM) 或纯 TS + Tailwind
- **编辑器**: Monaco Editor (VSCode 内核)
- **可视化**: ReactFlow / Tauri native 画布
- **状态**: Zustand/TanStack Query → Tauri Commands

### 2. Backend Core (src-tauri/src/)

#### state/ - 全局状态管理器
- `story_state.rs` - StoryState 结构体 + 持久化
- `manager.rs` - 单例状态管理器
- `schema.rs` - Zod-like 验证 (使用 validator/garde)

#### router/ - 动态模型路由
- `model.rs` - 模型配置结构体
- `router.rs` - 路由决策算法
- `cost.rs` - 成本计算器

#### agents/ - Agent 系统 (原 hermes-agent)
- `base.rs` - Agent trait
- `writer.rs` - 写作 Agent
- `inspector.rs` - 质检 Agent
- `evolver.rs` - 进化 Agent

#### memory/ - 记忆系统
- `short_term.rs` - 上下文窗口管理
- `vector_store.rs` - LanceDB 封装
- `embeddings.rs` - 嵌入生成

#### skills/ - 技能系统
- `character.rs` - 角色技能
- `world.rs` - 世界观技能
- `style.rs` - 文风技能
- `loader.rs` - 动态加载

#### evolution/ - 进化算法
- `analyzer.rs` - 内容分析
- `updater.rs` - Skill 更新
- `reviewer.rs` - 深度复盘

### 3. Core Library (src-core/)
独立可复用的 Rust 库，供 Tauri 调用

