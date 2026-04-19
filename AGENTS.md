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

- **版本**: v3.4.0
- **GitHub**: https://github.com/91zgaoge/StoryForge
- **技术栈**: Tauri 2.4 + Rust 1.94 + React 18 + TypeScript 5.8 + SQLite

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
- `npm run build` ✅
- `cargo test` ✅ 71/71

---

### 🏗️ 永久构建规则（用户强制要求）

> **每次推送到 GitHub 前，必须先在本地执行构建，然后再推送。**
> **每次推送到 GitHub 后，必须确保 GitHub Actions 自动触发全平台构建。**
> **推送 GitHub 的同时，必须在本地打包生成 Windows `.exe` / `.msi` 安装包。**
> **Git tag、Cargo.toml、tauri.conf.json、package.json 中的版本号必须保持统一。**

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
