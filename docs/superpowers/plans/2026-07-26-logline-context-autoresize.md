# 上下文感知 Logline 后缀 + 输入框自适应高度实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 `generate_logline_hint` 在作品已有资产时结合上下文生成后缀，并让底部输入框根据内容自动调整高度。

**Architecture:** 后端命令新增可选 `story_id`/`chapter_number` 参数，通过已有 Repository 拉取上下文后渲染新 prompt 资产；前端在防抖 effect 中传入当前作品/章节信息，并在 `FrontstageBottomBar` 中通过 `textareaRef` 与 `scrollHeight` 实现自适应高度。

**Tech Stack:** Rust (Tauri), React + TypeScript, CSS.

## Global Constraints

- 最小化改动；保持现有接口向后兼容。
- LLM 失败/超时仍静默降级，不阻塞 UI。
- 上下文读取失败只记录日志，回退通用 prompt。
- 前端 auto-resize 失败不影响输入功能。
- 不提交 `.zcode/` 目录。
- 改完后需更新 CHANGELOG 并打 `v0.30.27` tag。

---

### Task 1: 创建上下文感知 prompt 资产

**Files:**
- Create: `resources/prompts/agency/agency_logline_suffix_contextual.md`

**Interfaces:**
- Consumes: 模板变量 `story_outline`、`scene_outline`、`characters`、`current_content`
- Produces: 新 prompt ID `agency_logline_suffix_contextual`

- [ ] **Step 1: 编写 prompt 资产**

```markdown
---
id: agency_logline_suffix_contextual
name: "Logline 后缀增强（上下文感知）"
description: "当作品已有后台资产时，结合故事大纲、场景大纲、角色与当前正文生成贴合剧情的 logline 后缀"
category: system
version: 0.30.27
variables:
  - story_outline
  - scene_outline
  - characters
  - current_content
---

你是好莱坞资深故事概念设计师，精通 Erik Bork 的《The Idea》方法论。

## 任务

用户正在创作一个故事，并已积累以下后台资产。请根据这些上下文，为用户刚刚输入的简短指令生成一段**应直接追加到该指令之后**的增强后缀，使整句变成贴合当前剧情的强力 logline。

## 已输入

{{user_input}}

## 故事大纲

{{story_outline}}

## 当前章节/场景大纲

{{scene_outline}}

## 主要角色

{{characters}}

## 最近正文

{{current_content}}

## 输出要求

- **不要重复用户原输入**，只输出要追加的后缀。
- 后缀必须基于上述上下文，体现当前剧情走向、角色目标与冲突。
- 若上下文为空或不足以推断，则按通用 logline 原则生成。
- 只输出一段后缀文本，不要分析、解释、标号或 markdown。
- 总长度控制在 60-120 字。
```

- [ ] **Step 2: 验证文件创建**

Run: `ls resources/prompts/agency/agency_logline_suffix_contextual.md`
Expected: file exists

---

### Task 2: 后端 `generate_logline_hint` 支持上下文参数

**Files:**
- Modify: `src-tauri/src/commands/orchestrator.rs:964-1027`

**Interfaces:**
- Consumes: `StoryOutlineRepository`、`ChapterRepository`、`CharacterRepository`、`SceneRepository`（已有）
- Produces: `generate_logline_hint(user_input, story_id, chapter_number, app_handle)`

- [ ] **Step 1: 修改函数签名**

将
```rust
pub async fn generate_logline_hint(
    user_input: String,
    app_handle: AppHandle,
) -> Result<Option<String>, AppError>
```
改为
```rust
pub async fn generate_logline_hint(
    user_input: String,
    story_id: Option<String>,
    chapter_number: Option<i32>,
    app_handle: AppHandle,
) -> Result<Option<String>, AppError>
```

- [ ] **Step 2: 在函数内拉取上下文并渲染 contextual prompt**

在 `let trimmed = user_input.trim();` 之后插入：

```rust
let mut context_vars: std::collections::HashMap<String, String> = std::collections::HashMap::new();
context_vars.insert("user_input".to_string(), trimmed.to_string());

let contextual_system = if story_id.is_some() {
    if let Ok(ctx) = build_logline_context(
        story_id.as_deref(),
        chapter_number,
        app_handle.state::<crate::db::DbPool>(),
    )
    .await
    {
        context_vars.insert("story_outline".to_string(), ctx.story_outline);
        context_vars.insert("scene_outline".to_string(), ctx.scene_outline);
        context_vars.insert("characters".to_string(), ctx.characters);
        context_vars.insert("current_content".to_string(), ctx.current_content);
        crate::prompts::registry::resolve_prompt_default_with_vars(
            "agency_logline_suffix_contextual",
            &context_vars,
        )
    } else {
        None
    }
} else {
    None
};

let system = contextual_system.unwrap_or_else(|| {
    crate::prompts::registry::resolve_prompt_default_with_vars(
        "agency_logline_suffix",
        &std::collections::HashMap::new(),
    )
    .unwrap_or_else(|| {
        "你是故事概念设计师。用户输入了一句简单的创世指令。\
         请只输出一段应直接追加到该指令后的增强后缀..."
            .to_string()
    })
});
```

- [ ] **Step 3: 添加 `build_logline_context` 辅助函数**

在 `orchestrator.rs` 同一模块内添加：

```rust
#[derive(Debug, Default)]
struct LoglineContext {
    story_outline: String,
    scene_outline: String,
    characters: String,
    current_content: String,
}

async fn build_logline_context(
    story_id: Option<&str>,
    chapter_number: Option<i32>,
    db_pool: tauri::State<'_, crate::db::DbPool>,
) -> Result<LoglineContext, AppError> {
    let story_id = story_id.ok_or_else(|| AppError::ValidationError("story_id required".to_string()))?;

    let pool = db_pool.inner().clone();
    let story_id_owned = story_id.to_string();

    let ctx = tokio::task::spawn_blocking(move || {
        let story_outline_repo = crate::db::repositories::StoryOutlineRepository::new(pool.clone());
        let chapter_repo = crate::db::repositories::ChapterRepository::new(pool.clone());
        let character_repo = crate::db::repositories::CharacterRepository::new(pool.clone());

        let story_outline = story_outline_repo
            .get_by_story(&story_id_owned)
            .ok()
            .flatten()
            .map(|o| o.content)
            .unwrap_or_default();

        let chapters = chapter_repo.get_by_story(&story_id_owned).unwrap_or_default();
        let target_chapter = chapter_number.and_then(|cn| {
            chapters.iter().find(|c| c.chapter_number == cn)
        });

        let scene_outline = target_chapter
            .as_ref()
            .map(|c| c.outline.clone().unwrap_or_default())
            .unwrap_or_default();

        let current_content = target_chapter
            .as_ref()
            .and_then(|c| chapter_repo.get_content(&c.id).ok())
            .map(|s| truncate_chars(&s, 1200))
            .unwrap_or_default();

        let characters = character_repo
            .get_by_story(&story_id_owned)
            .unwrap_or_default()
            .iter()
            .map(|c| {
                format!(
                    "{}：背景{}；目标{}；性格{}",
                    c.name,
                    c.background.as_deref().unwrap_or("无"),
                    c.goals.as_deref().unwrap_or("无"),
                    c.personality.as_deref().unwrap_or("无")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        LoglineContext {
            story_outline,
            scene_outline,
            characters,
            current_content,
        }
    })
    .await
    .map_err(|e| AppError::InternalError(format!("context join failed: {}", e)))?;

    Ok(ctx)
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect::<String>() + "…"
    }
}
```

- [ ] **Step 4: 编译检查**

Run: `cd /Users/yuzaimu/projects/StoryForge && cargo check -p storymoss`
Expected: 0 errors

---

### Task 3: 前端 `generateLoglineHint` 扩展参数

**Files:**
- Modify: `src-frontend/src/services/api/stream.ts:18-20`
- Modify: `src-frontend/src/frontstage/FrontstageApp.tsx:981`

**Interfaces:**
- Consumes: `currentStory?.id`, `currentChapter?.chapter_number`
- Produces: `generateLoglineHint(userInput, storyId?, chapterNumber?)`

- [ ] **Step 1: 修改 service 签名**

```typescript
export const generateLoglineHint = (
  userInput: string,
  storyId?: string | null,
  chapterNumber?: number | null,
) =>
  loggedInvoke<string | null>('generate_logline_hint', {
    user_input: userInput,
    story_id: storyId ?? null,
    chapter_number: chapterNumber ?? null,
  });
```

- [ ] **Step 2: 在 effect 中传入上下文**

将 `FrontstageApp.tsx:981` 的
```typescript
const result = await generateLoglineHint(trimmed);
```
改为
```typescript
const result = await generateLoglineHint(
  trimmed,
  currentStoryRef.current?.id,
  currentChapterRef.current?.chapter_number ?? null,
);
```

- [ ] **Step 3: TypeScript 类型检查**

Run: `cd /Users/yuzaimu/projects/StoryForge/src-frontend && npm run type-check`
Expected: 0 errors

---

### Task 4: 输入框自适应高度

**Files:**
- Modify: `src-frontend/src/frontstage/components/FrontstageBottomBar.tsx`
- Modify: `src-frontend/src/frontstage/styles/frontstage.css:3621-3665`

**Interfaces:**
- Consumes: `inputValue`, `ghostHint`, `loglineHint`
- Produces: 动态 `textarea.style.height`

- [ ] **Step 1: 引入 useRef/useEffect 与高度逻辑**

在组件顶部添加：

```typescript
import React, { useState, useRef, useEffect } from 'react';
```

在 `FrontstageBottomBar` 内添加：

```typescript
const textareaRef = useRef<HTMLTextAreaElement>(null);
const MAX_TEXTAREA_HEIGHT = 200;

useEffect(() => {
  const el = textareaRef.current;
  if (!el) return;

  el.style.height = 'auto';
  const scrollHeight = el.scrollHeight;
  const newHeight = Math.min(scrollHeight, MAX_TEXTAREA_HEIGHT);
  el.style.height = `${newHeight}px`;

  if (scrollHeight > MAX_TEXTAREA_HEIGHT) {
    el.style.overflowY = 'auto';
  } else {
    el.style.overflowY = 'hidden';
  }
}, [inputValue, ghostHint, loglineHint]);
```

- [ ] **Step 2: 将 ref 绑定到 textarea**

```html
<textarea
  ref={textareaRef}
  className="frontstage-input-textarea"
  ...
/>
```

- [ ] **Step 3: 调整 CSS**

修改 `.frontstage-input-textarea`：
```css
.frontstage-input-textarea {
  ...
  max-height: 200px;
  min-height: 24px;
  overflow-y: hidden;
  padding: 3px 0;
}
```

修改 `.frontstage-input-ghost-inline`：
```css
.frontstage-input-ghost-inline {
  white-space: pre-wrap;
  word-break: break-word;
  overflow: hidden;
  text-overflow: clip;
  max-width: 100%;
  pointer-events: none;
}
```

- [ ] **Step 4: 前端测试**

Run: `cd /Users/yuzaimu/projects/StoryForge/src-frontend && npm run test:run`
Expected: all tests pass

---

### Task 5: 文档、提交与 Tag

**Files:**
- Modify: `CHANGELOG.md`（添加 v0.30.27 条目）

- [ ] **Step 1: 更新 CHANGELOG**

在 CHANGELOG 顶部添加：

```markdown
## v0.30.27

### Added
- `generate_logline_hint` 支持上下文感知：当作品已有故事大纲、场景大纲、角色与正文时，生成的 logline 后缀会贴合当前剧情。

### Changed
- 底部输入框（含内联幽灵文本）现在会根据内容自动调整高度，避免文字溢出。
```

- [ ] **Step 2: 跑完整验证**

Run:
```bash
cd /Users/yuzaimu/projects/StoryForge
cargo test -p storymoss
cd src-frontend && npm run type-check && npm run test:run
cargo fmt --all
cd src-frontend && npm run lint -- --fix
```

Expected: all green

- [ ] **Step 3: 提交并推送到 master**

```bash
git add -A
git commit -m "feat(v0.30.27): contextual logline hint + auto-resize input"
git tag v0.30.27
git push origin master --tags
```

Expected: push succeeds

---

## Spec Coverage

- [x] 上下文感知后缀生成：Task 1 + Task 2
- [x] 无上下文回退通用 prompt：Task 2 fallback
- [x] 输入框自适应高度：Task 4
- [x] 幽灵文本不溢出：Task 4 CSS
- [x] 文档与 tag：Task 5

## Placeholder Scan

无 TBD/TODO/实现后续；所有步骤含具体代码/命令。
