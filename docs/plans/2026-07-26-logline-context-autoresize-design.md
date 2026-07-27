# 上下文感知 Logline 后缀 + 输入框自适应高度设计

## 背景

v0.30.24-v0.30.26 已上线 Logline 幽灵提示：用户在底部输入框输入简短创世指令（<100 字符）时，后台生成一段增强后缀，以内联幽灵文本显示在输入内容之后，按 `→` 后原输入+后缀一并提交。

当前问题：
1. **后缀与上下文脱节**：当作品已有故事大纲、场景大纲、角色、正文等后台资产时，`generate_logline_hint` 仍使用通用 prompt，生成的后缀可能文不对题。
2. **输入框高度固定**：textarea 与幽灵层都限制 `max-height: 60px`，长后缀或长输入会溢出或被截断，视觉上不完整。

## 目标

1. 当存在 `story_id`（以及可选的 `chapter_number`）时，`generate_logline_hint` 必须拉取现有资产并生成贴合当前剧情的后缀；无上下文时保持原有通用行为。
2. 底部输入框（含幽灵文本）根据内容长度自动增高，避免溢出，同时保留滚动条作为上限兜底。

## 方案

### 方案 A：后端拉取上下文（推荐）

- `generate_logline_hint` 增加可选参数 `story_id: Option<String>`、`chapter_number: Option<i32>`。
- 后端通过已有 Repository 拉取：
  - 故事大纲 `story_outlines.content`
  - 当前章节大纲 `chapters.outline`
  - 角色列表 `kg_entities`（名称、背景、目标、性格）
  - 最近正文 `chapter_repository.get_content`（截断到约 1000 字符）
- 渲染新 prompt 资产 `agency_logline_suffix_contextual`，变量 `story_outline`、`scene_outline`、`characters`、`current_content`。
- 读取失败或无上下文时回退到原 `agency_logline_suffix`。

**优点**：前端改动小；数据访问集中；与现有权限/事务一致。

### 方案 B：前端拉取上下文

前端分别调用多个命令获取资产后传给后端。

**缺点**：多次 IPC；泄露数据职责到 UI；竞态复杂。

### 方案 C：新建专用服务

在 agency/coordinator 中新增服务方法。

**缺点**：过度设计；当前命令已足够独立。

**结论**：采用方案 A。

## 输入框自适应设计

- `FrontstageBottomBar` 增加 `textareaRef`，`useEffect` 监听 `inputValue`、`ghostHint`、`loglineHint`。
- 计算 `inputValue + ghostHint + loglineHint` 拼接后的高度，使用隐藏测量或 `textarea.scrollHeight`。
- 设置 `textarea.style.height = Math.min(scrollHeight, MAX_HEIGHT) + 'px'`（`MAX_HEIGHT ≈ 200px`），超出显示滚动条。
- 幽灵层 `frontstage-input-ghost-inline` 去掉固定 `max-height: 60px`，改为跟随 wrapper 高度；`frontstage-input-textarea` 同步增长。
- 保持 `rows={1}` 作为初始高度，通过 JS 动态调整。

## 数据流

```
FrontstageApp.tsx
  ├─ 输入防抖 effect ── generateLoglineHint(trimmed, currentStory?.id, currentChapter?.chapter_number)
  │                     └─ Tauri command generate_logline_hint
  │                          ├─ 有 story_id → 拉取上下文 → 渲染 agency_logline_suffix_contextual
  │                          └─ 无/失败 → 渲染 agency_logline_suffix（原行为）
  └─ 将 loglineHint 传给 FrontstageBottomBar

FrontstageBottomBar.tsx
  ├─ textareaRef + auto-resize effect
  └─ 渲染 inline ghost：前缀（隐藏）+ 后缀 + 提示
```

## 错误处理

- 上下文读取失败只记录 warn，不影响 logline 生成，回退通用 prompt。
- LLM 失败/超时仍静默返回 `None`。
- 前端 auto-resize 失败不影响输入功能。

## 测试

- `cargo test -p storymoss`：确保 Rust 编译与现有测试通过。
- `npm run type-check`：TypeScript 类型检查。
- `npm run test:run`：前端单元测试，包括 `FrontstageBottomBar.test.tsx`。
- 手动验证：已有资产的作品输入简短指令，后缀贴合剧情；长输入/长后缀时输入框自动增高。

## 版本

目标版本：`v0.30.27`。
