# StoryForge (草苔) v3.1 项目完成状态

> 最后更新: 2026-04-13（v3.1.1 Waza 设计原则落地 + CI 图标修复完成）
> GitHub: https://github.com/91zgaoge/StoryForge

---

## ✅ 已完成功能

### v3.1.1 新增功能（2026-04-13）

#### 🎭 幕前界面重构 (100%)

| 功能模块 | 状态 | 完成度 | 备注 |
|---------|------|--------|------|
| 精简侧边栏 | ✅ | 100% | 仅保留"幕后"按钮，120px 极简宽度 |
| OKLCH 颜色系统 | ✅ | 100% | 全站 OKLCH 色值，60-30-10 视觉权重 |
| LXGW WenKai 字体 | ✅ | 100% | 移除 Crimson/Inter，统一霞鹜文楷 |
| Blockquote 重设计 | ✅ | 100% | 背景色块 + 引号装饰，去左边框模板 |
| 微交互规范 | ✅ | 100% | 全按钮 `active:scale-95`，清除 `transition: all` |
| 顶部动态状态栏 | ✅ | 100% | 字数、字号、快捷键、保存状态 |
| 底部 LLM 对话栏 | ✅ | 100% | 悬停显示，集成模型状态灯，去除模式切换图标 |
| 流式对话 | ✅ | 100% | Enter 发送，Shift+Enter 换行 |

#### 🤖 本地模型配置 (100%)

| 模型 | 类型 | 状态 | 备注 |
|------|------|------|------|
| Gemma-4-31B-it-Q6_K | 多模态 | ✅ | http://10.62.239.13:17099/v1 |
| Qwen3.5-27B-Uncensored | 语言 | ✅ | http://10.62.239.13:17098/v1 |
| bge-m3 | Embedding | ✅ | http://10.62.239.13:8089 |

#### 🖥️ Tauri 构建与 CI (100%)

| 目标 | 状态 | 说明 |
|------|------|------|
| Release 编译 | ✅ | Rust 后端编译通过（189 warnings，非阻塞） |
| MSI 安装包 | ✅ | `StoryForge_0.1.0_x64_en-US.msi` (12.3 MB) |
| NSIS 安装包 | ✅ | `StoryForge_0.1.0_x64-setup.exe` (8.1 MB) |
| `rust-check` (Ubuntu) | ✅ | GitHub Actions 通过 |
| `rust-check` (Windows) | ✅ | GitHub Actions 通过 |
| `rust-check` (macOS) | ✅ | GitHub Actions 通过 |
| `tauri-build` Windows | ✅ | GitHub Actions 通过 |
| `tauri-build` macOS | 🔄 | 图标修复后重新验证中 |
| `tauri-build` Ubuntu | 🔄 | 图标修复后重新验证中 |

### v3.1.0 核心功能

#### 📜 场景版本系统 (100%)

| 功能模块 | 状态 | 完成度 | 备注 |
|---------|------|--------|------|
| SceneVersionRepository | ✅ | 100% | 版本CRUD、版本链管理 |
| SceneVersionService | ✅ | 100% | 比较、恢复、统计 |
| VersionTimeline 组件 | ✅ | 100% | 垂直时间线、版本选择 |
| DiffViewer 组件 | ✅ | 100% | 行级差异对比 |
| ConfidenceIndicator | ✅ | 100% | 圆形/条形置信度指示 |
| useSceneVersions hooks | ✅ | 100% | React Query封装 |
| Tauri 命令 | ✅ | 100% | 7个版本管理命令 |

#### 🔍 混合搜索系统 (100%)

| 功能模块 | 状态 | 完成度 | 备注 |
|---------|------|--------|------|
| Bm25Search | ✅ | 100% | CJK二元组分词、TF-IDF |
| HybridSearch | ✅ | 100% | RRF融合排序 |
| EntityHybridSearch | ✅ | 100% | 名称+向量混合 |
| LanceVectorStore | ✅ | 100% | LanceDB兼容API |
| 实体嵌入 | ✅ | 100% | 384维嵌入生成 |

#### 🧠 记忆保留系统 (100%)

| 功能模块 | 状态 | 完成度 | 备注 |
|---------|------|--------|------|
| RetentionManager | ✅ | 100% | 遗忘曲线计算 |
| 优先级分级 | ✅ | 100% | 五级优先级 |
| 遗忘预测 | ✅ | 100% | 遗忘时间预测 |
| 保留报告 | ✅ | 100% | 自动报告生成 |
| 上下文优化 | ✅ | 100% | 预算控制选择 |

### v3.0 核心功能

#### 🎪 场景化叙事系统 (100%)

| 功能模块 | 状态 | 完成度 | 备注 |
|---------|------|--------|------|
| Scene 数据模型 | ✅ | 100% | 戏剧目标、外部压迫、冲突类型、角色冲突 |
| SceneRepository | ✅ | 100% | CRUD + reorder_scenes 拖拽排序 |
| StoryTimeline 组件 | ✅ | 100% | @dnd-kit 拖拽、场景卡片、冲突标签 |
| SceneEditor 组件 | ✅ | 100% | 三标签页（基础/戏剧/内容） |
| ConflictType 枚举 | ✅ | 100% | 6 种标准冲突类型 |
| 场景页面 | ✅ | 100% | Scenes.tsx 完整实现 |
| Tauri 命令 | ✅ | 100% | 12 个场景相关命令 |

#### 🧠 增强记忆系统 (95%)

| 功能模块 | 状态 | 完成度 | 备注 |
|---------|------|--------|------|
| CJK Tokenizer | ✅ | 100% | Bigram 分词，中日韩支持 |
| Ingest Pipeline | ✅ | 100% | 两步思维链：分析→生成 |
| Knowledge Graph | ✅ | 90% | Entity/Relation 带强度评分 |
| Query Pipeline | ✅ | 100% | 四阶段检索管线 |
| Multi-Agent Sessions | ✅ | 100% | 6 种助手类型独立会话 |
| 数据库存储 | ✅ | 100% | kg_entities, kg_relations 表 |
| Tauri 命令 | ✅ | 100% | 8 个记忆系统命令 |

#### 🤖 AI 智能生成 (100%)

| 功能模块 | 状态 | 完成度 | 备注 |
|---------|------|--------|------|
| NovelCreationAgent | ✅ | 100% | 世界观/角色/文风/场景生成 |
| NovelCreationWizard | ✅ | 100% | 4 步引导式创建 |
| 卡片式 UI | ✅ | 100% | 单击选择、双击编辑 |
| 首个场景生成 | ✅ | 100% | 创建完成后自动生成 |
| Tauri 命令 | ✅ | 100% | 4 个创建相关命令 |

#### 📦 工作室配置系统 (100%)

| 功能模块 | 状态 | 完成度 | 备注 |
|---------|------|--------|------|
| StudioConfig 模型 | ✅ | 100% | 每部小说独立配置 |
| StudioManager | ✅ | 100% | ZIP 导入/导出、冲突处理 |
| 默认主题 | ✅ | 100% | 幕前暖色/幕后暗色 |
| Tauri 命令 | ✅ | 100% | 2 个配置管理命令 |

---

### 架构基础 (100%)

- ✅ Tauri + Rust 桌面应用框架
- ✅ 幕前幕后双窗口架构
- ✅ 窗口间通信机制 (Events)
- ✅ SQLite 数据库 (r2d2 连接池)
- ✅ 前端 React 18 + TypeScript 5.8 + Vite 6
- ✅ @dnd-kit 拖拽排序

---

## 📊 v3.1.1 新增文件清单

### 前端 (src-frontend/src/)

- `config/models.ts` - 本地三模型配置
- `hooks/useModel.ts` - 模型状态管理与对话 Hook
- `services/modelService.ts` - 模型 HTTP API 服务层

### 截图 (e2e/screenshots/)

- 幕前界面各状态截图（侧边栏、对话栏、模型状态等）

---

## 📈 整体完成度

### v3.1 模块完成度

| 模块 | 完成度 | 权重 | 加权得分 |
|------|--------|------|----------|
| 场景化叙事系统 | 100% | 20% | 20.0 |
| 增强记忆系统 | 95% | 20% | 19.0 |
| AI 智能生成 | 100% | 15% | 15.0 |
| 工作室配置 | 100% | 10% | 10.0 |
| 幕前界面 | 100% | 15% | 15.0 |
| 本地模型集成 | 100% | 10% | 10.0 |
| 后端架构 | 100% | 5% | 5.0 |
| 桌面构建打包 | 100% | 5% | 5.0 |
| **v3.1 总计** | - | 100% | **99.0%** |

---

## 🎯 待完善功能

### v3.2.x 计划

#### P1 - 重要功能
1. **向量存储持久化**
   - 位置: `src-tauri/src/vector/lancedb_store.rs`
   - 说明: LanceDB 完整集成，实体向量自动更新

2. **知识图谱可视化**
   - 位置: `src-frontend/src/components/KnowledgeGraph/`
   - 说明: 实体关系图谱可视化组件

3. **自动归档系统**
   - 位置: `src-tauri/src/memory/retention.rs`
   - 说明: 基于遗忘曲线的自动归档建议

#### P2 - 增强功能
4. **Ingest 管线性能优化**
   - 说明: 批量处理、异步优化

5. **查询缓存机制**
   - 说明: 缓存常用查询结果

6. **更多冲突类型**
   - 说明: 扩展 ConflictType 枚举

---

## 🐛 已知问题

### v3.1.1 已知问题
1. **编译警告**
   - 描述: 约 189 个非阻塞性警告（主要是未使用代码）
   - 影响: 无功能影响
   - 解决: 后续清理

### v3.1 已知问题（已解决）

1. ✅ **Windows 下 Tauri beforeBuildCommand 路径问题** - v3.1.1 已修复
2. ✅ **Tauri 文件锁阻塞** - v3.1.1 已解决并构建成功
3. ✅ **GitHub Actions macOS/Ubuntu 缺少 `icon.icns`** - v3.1.1 已修复并推送

---

## 📚 相关文档

- [README.md](../README.md) - 项目简介
- [ARCHITECTURE.md](../ARCHITECTURE.md) - 架构文档
- [ROADMAP.md](../ROADMAP.md) - 开发路线图
- [CHANGELOG.md](../CHANGELOG.md) - 更新日志
- [docs/plans/ARCHITECTURE_V3_PLAN.md](plans/ARCHITECTURE_V3_PLAN.md) - V3 详细设计
