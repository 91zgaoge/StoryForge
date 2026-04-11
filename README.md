# StoryForge (草苔) v2.0 - Rust Implementation

> 🌿 越写越懂的创作系统 - AI 辅助小说创作桌面应用

## 📊 项目状态概览

**当前版本**: v2.0.0-alpha  
**最后更新**: 2025-04-11  
**整体完成度**: ~85%

| 模块 | 状态 | 完成度 |
|------|------|--------|
| 核心架构 | ✅ 稳定 | 100% |
| 数据层 | ✅ 完成 | 100% |
| LLM 集成 | ✅ 完成 | 100% |
| Agent 系统 | ✅ 完成 | 95% |
| 向量检索 | ✅ 完成 | 90% |
| 工作流引擎 | ✅ 完成 | 90% |
| 技能系统 (Skills) | ✅ 重构完成 | 100% |
| MCP 支持 | ✅ 新增 | 90% |
| 状态管理 | ✅ 新增 | 100% |
| 模型路由 | ✅ 新增 | 100% |
| 进化算法 | ✅ 新增 | 100% |
| 导出功能 | ✅ 完善 | 100% |
| 前端界面 | 🚧 进行中 | 70% |

---

## 🗂️ 项目结构

```
v2-rust/
├── src-core/                    # 核心 Rust 库 (可复用)
│   ├── src/
│   │   ├── lib.rs              # 库入口
│   │   ├── error.rs            # 统一错误处理
│   │   ├── llm/                # LLM 适配器
│   │   │   ├── adapter.rs      # 通用适配器接口
│   │   │   ├── openai.rs       # OpenAI GPT 支持
│   │   │   ├── anthropic.rs    # Claude 支持
│   │   │   ├── ollama.rs       # 本地模型支持
│   │   │   └── types.rs        # 共享类型
│   │   ├── agents/             # Agent 系统
│   │   │   ├── base.rs         # Agent Trait
│   │   │   ├── writer.rs       # 写作 Agent
│   │   │   ├── inspector.rs    # 质检 Agent
│   │   │   ├── runner.rs       # Agent 执行器
│   │   │   └── loop_detector.rs # 循环检测
│   │   ├── db/                 # 数据持久化
│   │   │   ├── connection.rs   # 数据库连接
│   │   │   └── repositories.rs # 数据仓库
│   │   ├── vector/             # 向量数据库
│   │   │   ├── lancedb.rs      # LanceDB 封装
│   │   │   └── embeddings.rs   # 嵌入生成
│   │   ├── skills/             # 核心技能库
│   │   ├── state/              # 状态管理
│   │   └── evolution/          # 进化算法
│   └── Cargo.toml
│
├── src-tauri/                   # Tauri 桌面应用
│   ├── src/
│   │   ├── main.rs             # 应用入口
│   │   ├── commands.rs         # Tauri 命令
│   │   ├── db/                 # SQLite 数据库
│   │   │   ├── connection.rs   # 连接池管理
│   │   │   ├── models.rs       # 数据模型
│   │   │   └── repositories.rs # 数据访问层
│   │   ├── llm/                # LLM 服务层
│   │   │   ├── adapter.rs      # 适配器模式
│   │   │   ├── openai.rs       # OpenAI 实现
│   │   │   └── prompt.rs       # Prompt 管理
│   │   ├── agents/             # Agent 实现
│   │   │   ├── writer.rs       # 写作 Agent
│   │   │   ├── inspector.rs    # 检查 Agent
│   │   │   ├── outline_planner.rs # 大纲规划
│   │   │   ├── style_mimic.rs  # 风格模仿
│   │   │   └── plot_analyzer.rs # 情节分析
│   │   ├── skills/             # 🆕 技能系统 (重构)
│   │   │   ├── mod.rs          # 核心类型定义
│   │   │   ├── loader.rs       # 技能加载器
│   │   │   ├── registry.rs     # 技能注册表
│   │   │   ├── executor.rs     # 技能执行器
│   │   │   └── builtin.rs      # 内置技能
│   │   ├── mcp/                # 🆕 MCP (Model Context Protocol)
│   │   │   ├── mod.rs          # MCP 核心
│   │   │   ├── client.rs       # MCP 客户端
│   │   │   ├── server.rs       # MCP 服务端
│   │   │   ├── types.rs        # MCP 类型
│   │   │   └── transport.rs    # 传输层
│   │   ├── state/              # 🆕 全局状态管理
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs      # StoryState 管理器
│   │   │   └── schema.rs       # 验证 Schema
│   │   ├── router/             # 🆕 模型路由
│   │   │   ├── mod.rs
│   │   │   ├── model.rs        # 模型配置
│   │   │   ├── router.rs       # 路由决策
│   │   │   └── cost.rs         # 成本计算
│   │   ├── evolution/          # 🆕 进化算法
│   │   │   ├── mod.rs
│   │   │   ├── analyzer.rs     # 内容分析
│   │   │   ├── updater.rs      # Skill 更新
│   │   │   └── reviewer.rs     # 深度复盘
│   │   ├── embeddings/         # 🆕 嵌入系统
│   │   │   ├── mod.rs
│   │   │   ├── embedding.rs    # 嵌入配置
│   │   │   └── provider.rs     # 提供商实现
│   │   ├── memory/             # 记忆系统
│   │   ├── vector/             # 向量存储
│   │   ├── workflow/           # 工作流引擎
│   │   ├── export/             # 导出功能
│   │   │   ├── mod.rs
│   │   │   ├── pdf.rs          # PDF 导出
│   │   │   └── epub.rs         # EPUB 导出
│   │   ├── prompts/            # Prompt 管理
│   │   ├── versions/           # 版本管理
│   │   ├── chat/               # 对话系统
│   │   ├── analytics/          # 分析统计
│   │   ├── collab/             # 协同编辑
│   │   └── utils/              # 🆕 工具函数
│   │       ├── mod.rs
│   │       ├── text.rs         # 文本处理
│   │       ├── file.rs         # 文件操作
│   │       └── validation.rs   # 验证工具
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                         # 前端代码
│   ├── main.js                 # 主应用逻辑
│   ├── views.js                # 视图组件
│   └── mock-tauri.js           # 开发模拟
│
├── src-ui/                      # UI 组件 (预留)
├── docs/                        # 文档
├── index.html                   # 前端入口
└── Cargo.toml                   # Workspace 配置
```

---

## ✅ 功能实现详情

### 1. 核心数据层 (100% ✅)

| 功能 | 状态 | 说明 |
|------|------|------|
| SQLite 数据库 | ✅ | r2d2 连接池，完整 CRUD |
| Story 表 | ✅ | 故事元数据存储 |
| Chapter 表 | ✅ | 章节内容、大纲、状态 |
| Character 表 | ✅ | 角色信息、动态特质 |
| Repository 模式 | ✅ | 数据访问层封装 |

### 2. LLM 集成层 (100% ✅)

| 提供商 | 状态 | 功能 |
|--------|------|------|
| OpenAI | ✅ | GPT-4, GPT-4 Turbo, GPT-3.5 |
| Anthropic | ✅ | Claude 3 Opus, Sonnet |
| Ollama | ✅ | 本地模型支持 |
| 流式生成 | ✅ | 实时响应 |
| Prompt 管理 | ✅ | 模板系统 |

### 3. Agent 系统 (95% ✅)

| Agent | 状态 | 功能描述 |
|-------|------|----------|
| WriterAgent | ✅ | 智能章节写作 |
| InspectorAgent | ✅ | 质量检查、一致性验证 |
| OutlinePlannerAgent | ✅ | 大纲规划 |
| StyleMimicAgent | ✅ | 风格模仿学习 |
| PlotComplexityAgent | ✅ | 情节复杂度分析 |
| LoopDetector | ✅ | 重复内容检测 |

### 4. 向量检索 (90% ✅)

| 功能 | 状态 | 说明 |
|------|------|------|
| 向量存储 | ✅ | 纯 Rust 实现，TF 向量化 |
| 相似度搜索 | ✅ | 余弦相似度 |
| 章节嵌入 | ✅ | 自动向量化 |
| LanceDB | ⚠️ | 预留接口 |

### 5. 技能系统 (100% ✅) - v2.0 重构

**重大变更**: 原 Plugin 系统已替换为通用 Skills 系统

| 功能 | 状态 | 说明 |
|------|------|------|
| 技能导入 | ✅ | 目录/文件导入 |
| 技能分类 | ✅ | Writing/Analysis/Character/Plot/Style 等 |
| Prompt 技能 | ✅ | 基于 Prompt 的技能 |
| MCP 技能 | ✅ | 支持 MCP Server |
| Hook 系统 | ✅ | 事件钩子机制 |
| 内置技能 | ✅ | 5+ 内置技能 |

**内置技能清单**:
- `builtin.style_enhancer` - 文风增强器
- `builtin.plot_twist` - 情节反转生成器
- `builtin.character_voice` - 角色声音一致性
- `builtin.emotion_analyzer` - 情感曲线分析
- `builtin.pacing_optimizer` - 节奏优化器

### 6. MCP 支持 (90% ✅) - v2.0 新增

| 功能 | 状态 | 说明 |
|------|------|------|
| MCP Client | ✅ | 连接外部 MCP Server |
| MCP Server | ⚠️ | 框架就绪 |
| 工具调用 | ✅ | Tool Call 支持 |
| 资源读取 | ✅ | Resource 支持 |
| Stdio 传输 | ✅ | 标准输入输出传输 |

### 7. 状态管理 (100% ✅) - v2.0 新增

| 功能 | 状态 | 说明 |
|------|------|------|
| StoryState | ✅ | 完整故事状态 |
| CharacterState | ✅ | 角色状态追踪 |
| ChapterState | ✅ | 章节状态 |
| PlotProgression | ✅ | 情节推进追踪 |
| WorldState | ✅ | 世界观状态 |
| 数据验证 | ✅ | Schema 验证 |

### 8. 模型路由 (100% ✅) - v2.0 新增

| 功能 | 状态 | 说明 |
|------|------|------|
| 模型配置 | ✅ | 多模型管理 |
| 智能路由 | ✅ | 基于任务类型选择模型 |
| 成本计算 | ✅ | Token 成本追踪 |
| 质量分级 | ✅ | Ultra/High/Medium/Low |
| 速度分级 | ✅ | Fast/Normal/Slow/VerySlow |

### 9. 进化算法 (100% ✅) - v2.0 新增

| 功能 | 状态 | 说明 |
|------|------|------|
| 内容分析 | ✅ | 节奏、一致性、连贯性 |
| Skill 更新 | ✅ | 自动 Skill 优化 |
| 深度复盘 | ✅ | 叙事弧、主题发展分析 |
| 读者参与度预测 | ✅ | Engagement Prediction |

### 10. 导出功能 (100% ✅)

| 格式 | 状态 | 说明 |
|------|------|------|
| Markdown | ✅ | 标准 Markdown |
| PlainText | ✅ | 纯文本 |
| JSON | ✅ | 结构化数据 |
| HTML | ✅ | 带样式网页 |
| PDF | ✅ | 使用 printpdf |
| EPUB | ✅ | 使用 epub-builder |

### 11. 辅助功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 版本管理 | ✅ | 章节版本历史 |
| 对话系统 | ✅ | 创作对话助手 |
| 分析统计 | ✅ | 写作数据分析 |
| 协同编辑 | ⚠️ | 基础框架 |

---

## 📅 更新历史

### v2.0.0 (2025-04-11) - 重大重构

#### 新增模块
- **Skills 系统** - 替换原有 Plugin 系统，支持通用技能导入
- **MCP 支持** - Model Context Protocol 集成
- **State 管理** - 完整故事状态追踪
- **Router 路由** - 智能模型选择
- **Evolution 进化** - 内容分析与 Skill 进化
- **Embeddings** - 文本嵌入系统
- **Utils 工具** - 通用工具函数

#### 完善功能
- PDF/EPUB 导出完整实现
- 所有空模块补充实现
- 架构对齐 ARCHITECTURE.md

#### 优化点
- 代码结构更清晰
- 模块化程度更高
- 扩展性增强

### v1.5.0 (2025-04-08)

- Agent 系统完善
- 工作流引擎实现
- 向量存储实现

### v1.0.0 (2025-04-01)

- 基础架构搭建
- LLM 集成
- 数据库设计
- 前端界面

---

## 🚀 快速开始

### 环境要求
- Rust 1.70+
- Node.js 18+ (前端开发)
- SQLite 3

### 开发模式

```bash
# 1. 克隆项目
cd v2-rust

# 2. 安装依赖
cargo build

# 3. 运行开发服务器
cargo tauri dev

# 4. 构建发布版本
cargo tauri build
```

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
  },
  "llm_profiles": {},
  "embedding_profiles": {}
}
```

---

## 🛣️ 路线图 (Roadmap)

### 短期计划 (v2.1.0)
- [ ] 前端界面完善
- [ ] Monaco Editor 集成
- [ ] 实时预览功能
- [ ] 性能优化

### 中期计划 (v2.2.0)
- [ ] WebAssembly 支持
- [ ] 多语言支持
- [ ] 插件市场
- [ ] 云同步功能

### 长期计划 (v3.0.0)
- [ ] AI 辅助大纲生成
- [ ] 多人协作
- [ ] 移动端支持
- [ ] 发布平台集成

---

## 🐛 已知问题

1. **前端界面** - 部分视图需要完善
2. **协同编辑** - OT 算法需要完整实现
3. **MCP Server** - 服务端实现待完善

---

## 📚 相关文档

- [架构设计](ARCHITECTURE.md) - 详细架构说明
- [API 文档](docs/api.md) - API 接口文档
- [开发指南](docs/development.md) - 开发贡献指南

---

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE)

---

**StoryForge (草苔)** - 让创作更智能 🌿
