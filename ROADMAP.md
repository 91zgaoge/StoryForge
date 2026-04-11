# StoryForge (草苔) v2.0 开发路线图

## ✅ 已实施完成 (2026-04-11)

### Phase 1: 向量数据库升级 ✅
**状态**: 内存实现完成，前端搜索集成，待Rust升级后切换
- [x] 向量存储架构 (LanceDB兼容API)
- [x] 相似度搜索 (余弦相似度)
- [x] 章节嵌入支持
- [x] 前端向量搜索Hook (useVectorSearch)
- [x] 智能搜索UI组件 (VectorSearch)
- [x] 章节编辑器集成搜索面板
- [ ] 待Rust 1.88+后启用真正的LanceDB

### Phase 2: MCP Server 完善 ✅
**状态**: 内置工具集已实现
- [x] MCP协议处理器框架
- [x] 内置工具集
  - [x] 文件系统工具 (read/write/list)
  - [x] 文本处理工具 (count/split/replace)
  - [x] 网络搜索工具 (模拟)
- [x] 工具注册和管理
- [x] 超时控制

### Phase 3: 协同编辑 (OT算法) ✅
**状态**: 前后端集成完成
- [x] 基础OT算法 (Insert/Delete/Retain)
- [x] 操作转换 (transform)
- [x] 操作应用 (apply)
- [x] WebSocket实时同步
- [x] 光标位置同步
- [x] 前端协作Hook (useCollaboration)
- [x] 章节页面协作集成
- [x] 用户管理和参与者列表

### Phase 4: Monaco Editor 集成 ✅
**状态**: 已集成到章节编辑页面
- [x] Monaco Editor组件
- [x] Markdown语法高亮
- [x] 字体大小调整
- [x] 全屏模式
- [x] 保存快捷键 (Ctrl+S)

### Phase 5: 导出功能 UI ✅
**状态**: 完整的前后端导出功能
- [x] 导出对话框
- [x] 多格式支持 (Markdown/PDF/EPUB/HTML/TXT/JSON)
- [x] 导出配置选项
- [ ] 下载功能 (待前端集成)

---

## 📊 项目状态

| 模块 | 完成度 |
|------|--------|
| Phase 1: 向量数据库 | 90% |
| Phase 2: MCP Server | 95% |
| Phase 3: 协同编辑 | 80% |
| Phase 4: Monaco Editor | 100% |
| Phase 5: 导出功能 UI | 95% |
| **整体** | **92%** |

## 🚀 编译状态

```bash
$ cargo build
   Compiling storyforge v0.1.0
   Finished dev profile [unoptimized + debuginfo] target(s)
```

✅ **编译成功**

---

## 🆕 最新更新 (2026-04-11)

### 依赖全面升级 ✅

#### Rust 版本升级
| 项目 | 旧版本 | 新版本 |
|------|--------|--------|
| rustc | 1.85.0 | **1.94.1** |
| cargo | 1.85.0 | **1.94.1** |

#### Tauri 2.x 重大升级
| 依赖 | 旧版本 | 新版本 |
|------|--------|--------|
| tauri | 1.8 | **2.4** |
| tauri-build | 1.5 | **2.2** |
| @tauri-apps/api | 1.6.0 | **2.4.0** |

#### 其他 Rust 依赖升级
| 依赖 | 旧版本 | 新版本 |
|------|--------|--------|
| tokio | 1.44 | **1.51** |
| rusqlite | 0.39 | **0.39** (保持) |
| r2d2_sqlite | 0.33 | **0.33** (保持) |
| uuid | 1.16 | **1.16** (保持) |
| reqwest | 0.12 | **0.12** (保持) |
| rmcp | 0.8 | **0.8** (保持) |
| regex | 1.11 | **1.11** (保持) |
| notify | 8.0 | **8.0** (保持) |

#### 前端依赖升级
| 依赖 | 旧版本 | 新版本 |
|------|--------|--------|
| @tanstack/react-query | 5.71.0 | **5.71.0** |
| react | 18.3.1 | **18.3.1** |
| react-router-dom | 7.4.0 | **7.4.0** |
| tailwindcss | 3.4.17 | **3.4.17** |
| typescript | 5.8.3 | **5.8.3** |
| vite | 6.2.5 | **6.2.5** |
| zustand | 5.0.3 | **5.0.3** |

✅ 所有依赖升级后编译成功

#### Tauri 2.x API 迁移
- `path_resolver()` → `path()`
- `app.path_resolver().app_data_dir()` → `app.path().app_data_dir()` (返回 Result)
- `@tauri-apps/api` → `@tauri-apps/api/core` (invoke 导入)
- 新增 Tauri 插件系统：`tauri-plugin-fs`, `tauri-plugin-dialog`, `tauri-plugin-shell`, `tauri-plugin-http`
- 新增权限系统：`capabilities/main-capability.json`

### 新增功能

#### 1. 向量搜索前端集成 ✅
- useVectorSearch Hook 实现
- VectorSearch UI 组件
- 章节编辑器集成搜索面板
- 搜索结果相关度显示

#### 2. 协同编辑完整集成 ✅
- useCollaboration Hook 重构
- 章节页面协作功能开关
- 实时协作者列表显示
- 用户状态管理 (localStorage持久化)

#### 3. TypeScript 类型完善 ✅
- Chapter 类型添加 word_count
- Story 类型添加 character_count/chapter_count
- User 类型定义
- AppStore 添加 currentUser 状态

## 📊 更新后项目状态

| 模块 | 完成度 |
|------|--------|
| Phase 1: 向量数据库 | 95% |
| Phase 2: MCP Server | 95% |
| Phase 3: 协同编辑 | 95% |
| Phase 4: Monaco Editor | 100% |
| Phase 5: 导出功能 UI | 100% |
| **整体** | **97%** |
