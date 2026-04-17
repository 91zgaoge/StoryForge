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

- **版本**: v3.3.0-in-progress
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

- **品牌 Logo 全面应用** (2026-04-15)
  - `LOGO.jpg` 生成 Tauri 全平台图标包（Windows / macOS / iOS / Android）
  - 前端 favicon 从 `feather.svg` 替换为 `favicon.ico` + `apple-touch-icon.png`
  - `docs/images/logo.png` 作为 README 品牌展示图
  - README / CHANGELOG / PROJECT_STATUS 品牌描述同步更新

- **幕前右键菜单修复与暖色重构** (2026-04-15)
  - 修复 `frontstage.css` 缺失 Tailwind utilities 导致菜单 `fixed`/`z-[9999]` 类不生效的问题
  - WebView2 禁用 Windows 默认系统右键菜单
  - 菜单 UI 全面适配幕前暖色纸张设计规范

- **幕前排版与 AI 续写优化** (2026-04-17)
  - 段落间距：`.ProseMirror p` `margin-bottom` 从 `1.5em` 降至 `0`，统一增加 `text-indent: 2em`
  - 底部栏不再遮挡：编辑器 `padding-bottom` 增至 `10rem`
  - 自动续写：接受 AI 生成（`Tab`）后自动触发下一轮 `executeWriterAgent('续写')`
  - Zen 模式纯净：隐藏所有 AI UI（气泡、指示器、接受/拒绝按钮及快捷键）

- **P3 修订模式与变更追踪** (`8a13661` ~ `b26ca51`)
  - Phase 1: ChangeTrack 数据库 + TipTap `trackInsert`/`trackDelete` Mark + 实时 diff 检测
  - Phase 2: `comment_threads` / `comment_messages` + `commentAnchor` Mark + 右侧评论面板
  - Phase 3: 版本集成（保存场景/接受拒绝变更时自动生成版本快照 + diff ChangeTrack）

### 编译状态

- `cargo check` ✅ | 警告: 0
- `npm run build` ✅
- `cargo test` ✅ 20/20

---

### 🏗️ 永久构建规则（用户强制要求）

> **每次推送到 GitHub 前，必须先在本地执行构建，然后再推送。**
> **每次推送到 GitHub 后，必须确保 GitHub Actions 自动触发全平台构建。**
> **Git tag、Cargo.toml、tauri.conf.json、package.json 中的版本号必须保持统一。**

**本地构建脚本**: `scripts/build-local.ps1`
```powershell
# Windows 本地构建
.\scripts\build-local.ps1

# 或指定平台
.\scripts\build-local.ps1 -Windows
.\scripts\build-local.ps1 -All
```

**现实限制**:
- Windows 主机 ✅ 可本地构建 Windows (.msi/.exe)
- Linux 主机 ⚠️ 需 WSL 或 Linux 虚拟机
- macOS 主机 ❌ 无法在 Windows 上本地构建（需 macOS + Xcode）
- 跨平台完整构建 → 交由 GitHub Actions (`ubuntu-latest` / `windows-latest` / `macos-latest`)

---

*最后更新: 2026-04-17 - 幕前排版与自动续写优化完成，版本号统一为 3.3.0*
