# P4 AI 原生组件库收尾（清理残留 + 视觉修正 + 浅色页令牌化 + 回归门）Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 关闭 beautifului AI 原生组件替换项目：删除 P1-P3 替换残留（TS 13 处 + frontstage.css 约 40 个死类）与 8 件历史死件（含自带测试/barrel/级联死 CSS）；修复四组件 `text-white` 直写（新增 `--ai-on-accent` 令牌）与 `/N` 透明度修饰符失效 13 处（color-mix 内联）；Tasks.tsx 末处裸 pre 换 AiCodeBlock；AiDiffTable testid 改 per-row key；AgencyEval/AgencyStudio/AgencyLearning 三个浅色裸样式页令牌化（含 AgencyLearning 裸 table → AiRecordsTable）；最终全量回归门 + 文档同步，设计文档 §8 P4 行关闭。

**Architecture:** 不新建组件、不扩依赖。令牌层仅扩一个变量：`--ai-on-accent`（幕后 tokens.css / 幕前 frontstage.css / tailwind ai 色组 / aiTokens.test.ts 两数组四处同步），16 变量契约变 17。tint 修复沿用 P1-P3 既定零扩令牌路线：`color-mix(in srgb, var(--ai-x) N%, transparent)` 内联（先例 AiDiffTable.tsx:109-115 / AgencyEval.tsx:31），不改 tailwind 配置（var 间接链构建期不可解析，已验证走不通）。删除批以回归门为验证。浅色页令牌化只换颜色来源，不改布局结构。

**Tech Stack:** React 18 + Tailwind v3.4（`var()` 色映射）、vitest 4 + Testing Library、jsdom。零新依赖。

**需求来源：** `.superpowers/sdd/p4-recon-summary.md`（agent-263 死代码侧 / agent-264 候选项侧勘察汇总，本计划全部发现、文件路径、行号、勿误删清单以它为准）。设计文档：`docs/plans/2026-08-12-beautifului-ai-native-design.md` §8 第 5 条「P4 收尾：旧组件删除、死代码清理、文档与发版」（发版推送不在本计划内，等用户指令）。

## Global Constraints

- 仓库 /Users/yuzaimu/projects/StoryForge；master 直接工作；中文 conventional commit；不 --no-verify（pre-commit fmt 检查不绕过）；不推送、不打 tag；发版另行进行。
- **删除前必须 grep 全仓确认零引用**（含测试文件、barrel 导出、CSS 类名三处口径），grep 结果写进 commit message 或评审记录。
- **勿误删清单（勘察实测均活着，严禁动）**：Tasks.tsx 的 `TaskRow`（L63）与 `CascadeRewriteDetail`（L202）；AgencyEval.tsx 的 `CheckpointCompare`（L82）；PromptsPanel.tsx 的 `writeJsonViaDialog`（L156）；frontstage.css 的 `.zen-mode-exit`（注意与要删的 `.zen-exit-hint` 一字之差）。index.css/tokens.css 无零引用类，删除批不动这两个文件（Task 3 加 `--ai-on-accent` 除外）。
- **`--ai-on-accent` 四文件必须同一 commit 同步改**：tokens.css + frontstage.css + tailwind.config.js + aiTokens.test.ts（AI_VARS/AI_COLOR_KEYS 两数组）。只改部分文件会导致 aiTokens 测试红。值：幕后幕前均 `#ffffff`（不能直接引用仅幕前有的 `var(--text-on-accent)`）。
- **浅色页令牌化只换颜色来源**：不改布局结构、间距、字号、组件层级；`text-gray-*`/hex/bg-white 等直写换成 `--ai-*` 语义令牌（tailwind `ai-*` 类或 style 内联），select/表格等缺令牌样式的补 `border-ai-line bg-ai-field text-ai-ink` 一套。
- **删除类 Task 的"测试"是回归门**：`npx vitest run` 全绿（计数允许按删除批预期下降）+ `npx tsc --noEmit` 0 error + grep 验证无残留引用。不为删除新增测试。
- **修正类 Task（Task 1-4）不改组件行为**：text-white→令牌与 /N 修复是视觉修复（修完后 tint 从全饱和恢复为半透明是修复性变化，非回归）。
- **行号口径**：全部行号抄自勘察基线（P3 完成后 master）。前置 Task 合并会引起漂移（Tasks.tsx 被 Task 1/4/5 触及、PromptsPanel.tsx 被 Task 4 触及、AiDiffTable.tsx 被 Task 2 触及、frontstage.css 被 Task 3/5/6 触及）——执行时以**锚点代码内容**定位，行号仅作初始参考。
- **纯前端阶段**：不改任何 Rust 后端代码；`cargo test --lib` 基线 **1328 passed / 2 ignored** 不变；若执行中意外动了 rust 文件则必须重跑。
- 准入线：`cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check` 全绿 + 仓库根 `python3 scripts/architecture_guard.py` 通过；vitest 基线 **564 passed / 3 skipped**（P3 完成后实测），删除批（Task 6）允许 −8（死件自带测试），其余 Task 计数不变。
- B2（死导出：errorHandler 4/logger 4/genesisSteps 2/useTextAnnotations 8/useChapters 2）与 B3（历史 CSS：.slash-command-*/.smart-hint-*/.smart-ambient-*/.free-hint-*/.subscription-status/.writing-style-* 等）**不纳入本批**——建议见 Task 6 说明，两者记入 Task 10 文档「已知遗留」。

---

### Task 1: Tasks.tsx 裸 pre → AiCodeBlock（S）

**Files:**
- Modify: `src-frontend/src/pages/Tasks.tsx`（L392 task.result JSON dump 裸 pre + 外层容器 div）

**Interfaces:**
- Consumes: P3 已入库 `AiCodeBlock`（`@/components/ui/ai/AiCodeBlock`，props：`code: string`、`language?: string`、`maxHeight?: number`，受控只读 + 复制按钮）
- Produces: 无新 API；视觉替换

说明：AiCodeBlock 组件行为已由 P3 Task2 自带测试（6 例）覆盖，本 Task 是纯集成替换，**无新增测试**，验证走回归门 + 既有测试不回归。

- [ ] **Step 1: 替换**

L392 区域现状为外层容器 div 包裸 `<pre>`（task.result 的 JSON dump）。替换为：

```tsx
<AiCodeBlock code={JSON.stringify(task.result, null, 2)} language="json" maxHeight={160} />
```

并删除原外层容器 div（AiCodeBlock 自带容器样式）。`JSON.stringify` 的第二三参数与原 pre 内用法保持一致（执行时以锚点代码现状为准，原样携带）。

- [ ] **Step 2: 回归门**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | tail -3`
Expected: tsc 0 error；vitest 564 passed / 3 skipped 不变

- [ ] **Step 3: Commit**

```bash
git add src-frontend/src/pages/Tasks.tsx
git commit -m "refactor: Tasks 结果详情裸 pre 替换为 AiCodeBlock（P4 Task1）"
```

---

### Task 2: AiDiffTable testid 改 per-row key（S，零风险）

**Files:**
- Modify: `src-frontend/src/components/ui/ai/AiDiffTable.tsx`（L107 `data-testid="ai-diff-delta"`）

**Interfaces:**
- Consumes: `row.key`（AiDiffRow 既有字段）
- Produces: `data-testid={`ai-diff-delta-${row.key}`}`（同 AiTaskRows `ai-task-badge-${status}` 先例）

说明：起草时已 grep 全仓核实，`ai-diff-delta` 仅 AiDiffTable.tsx:107 自身一处，无测试/宿主消费者（AiDiffTable.test.tsx 未断言该 testid）。多行渲染时 testid 重复本就违背 testid 语义，改为 per-row key 后零影响。

- [ ] **Step 1: 修改**

```tsx
data-testid={`ai-diff-delta-${row.key}`}
```

- [ ] **Step 2: 回归门**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiDiffTable.test.tsx && npx tsc --noEmit`
Expected: AiDiffTable 测试 7 例全绿；tsc 0 error

- [ ] **Step 3: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiDiffTable.tsx
git commit -m "refactor: AiDiffTable delta testid 改 per-row key（P4 Task2）"
```

---

### Task 3: `--ai-on-accent` 令牌新增 + text-white ×4 替换（S）

**Files:**
- Modify: `src-frontend/src/styles/tokens.css`（AI 令牌块 L38-54，L54 `--ai-orange` 行后加）
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`（AI 令牌块 L63-79，同位置加）
- Modify: `src-frontend/tailwind.config.js`（ai 色组 L46-63，`orange` 映射后加）
- Modify: `src-frontend/src/styles/__tests__/aiTokens.test.ts`（AI_VARS 数组 L19-36、AI_COLOR_KEYS 数组 L39-56，各加一项）
- Modify: `src-frontend/src/components/ui/ai/AiApprovalCard.tsx`（L116 text-white）
- Modify: `src-frontend/src/components/ui/ai/AiTaskRows.tsx`（L101 text-white）
- Modify: `src-frontend/src/components/ui/ai/AiContextCards.tsx`（L84 text-white）
- Modify: `src-frontend/src/components/ui/ai/AiRecommendationCard.tsx`（L169 text-white）

**Interfaces:**
- Consumes: 既有 16 变量契约
- Produces: 第 17 个语义令牌 `--ai-on-accent`（accent 底色上的前景文字色，幕后幕前均 `#ffffff`）；tailwind 工具类 `text-ai-on-accent`/`bg-ai-on-accent` 等

**TDD 节奏：** 先改测试（Step 1，必 FAIL——三处实现文件缺定义），再改实现（Step 2-3，PASS）。`toContain` 断言是数组驱动循环，数组加一项即扩充断言，**测试用例数不变**（5 例不变），不需改既有断言。

- [ ] **Step 1: Write the failing test**

`aiTokens.test.ts` 两处修改：

```ts
// AI_VARS 数组（'--ai-orange' 后）加：
  '--ai-on-accent',
// AI_COLOR_KEYS 数组（'orange' 后）加：
  'on-accent',
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/styles/__tests__/aiTokens.test.ts`
Expected: FAIL（tokens.css/frontstage.css 缺 `--ai-on-accent:`、tailwind.config.js 缺 `var(--ai-on-accent)` 映射）

- [ ] **Step 3: Write implementation**

**(a) `tokens.css`** — `--ai-orange: var(--status-warning);`（L54）行后加：

```css
  --ai-on-accent: #ffffff;
```

**(b) `frontstage.css`** — AI 令牌块（L63-79）同位置加同一行（值同为 `#ffffff`；注意不能直接引用仅幕前有的 `var(--text-on-accent)`，须写死 hex 保持两窗口契约一致）。

**(c) `tailwind.config.js`** — ai 色组 `orange: 'var(--ai-orange)'` 映射后加：

```js
        'on-accent': 'var(--ai-on-accent)',
```

**(d) 四组件替换**：AiApprovalCard.tsx:116 / AiTaskRows.tsx:101 / AiContextCards.tsx:84 / AiRecommendationCard.tsx:169 的 `text-white` → `text-ai-on-accent`（执行时以锚点 `text-white` 定位，每文件恰一处）。

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-frontend && npx vitest run src/styles/__tests__/aiTokens.test.ts`
Expected: 5 passed

- [ ] **Step 5: 回归门 + Commit**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | tail -3 && npm run format:check`
Expected: tsc 0 error；vitest 564 passed / 3 skipped 不变（四组件既有测试不断言颜色类，全绿）；format 通过

```bash
git add src-frontend/src/styles/tokens.css src-frontend/src/frontstage/styles/frontstage.css src-frontend/tailwind.config.js src-frontend/src/styles/__tests__/aiTokens.test.ts src-frontend/src/components/ui/ai/{AiApprovalCard,AiTaskRows,AiContextCards,AiRecommendationCard}.tsx
git commit -m "feat: 新增 --ai-on-accent 语义令牌并替换四组件 text-white 直写（P4 Task3）"
```

---

### Task 4: `/N` 透明度修饰符失效 13 处 color-mix 修复（S）

**Files:**
- Modify: `src-frontend/src/pages/Tasks.tsx`（L130-132 / L157 / L165 / L173 / L182 / L309 / L315，共 8 处，状态徽章与 hover）
- Modify: `src-frontend/src/components/ui/ai/AiSearchList.tsx`（L56）
- Modify: `src-frontend/src/components/ui/ai/AiCodeBlock.tsx`（L109）
- Modify: `src-frontend/src/pages/settings/PromptsPanel.tsx`（L664 / L724）

**Interfaces:**
- Consumes: 既有 `--ai-*` 变量
- Produces: 无新 API；视觉 bug 修复

**背景（勘察结论）**：tailwind 色映射为纯 `var()` 无 `<alpha-value>` 占位，`bg-ai-green/15` 这类 `/N` 修饰符被构建期静默丢弃 → 徽章底色变全饱和。修法已定：`color-mix(in srgb, var(--ai-x) N%, transparent)` 内联 style，先例 AiDiffTable.tsx:109-115 / AgencyEval.tsx:31。**不改 tailwind 配置**（var 间接链构建期不可解析，已验证走不通）。修完后徽章从全饱和变 tint 是修复性变化。无新增测试（视觉修复），验证走回归门 + 目视。

- [ ] **Step 1: 逐处修复**

改法模式（以 Tasks.tsx 状态徽章为例，锚点为含 `/` 修饰符的 `ai-*` 类）：

```tsx
// 改前：className 内含 bg-ai-green/15（失效，实际全饱和）
// 改后：删掉该类，改内联
style={{ background: 'color-mix(in srgb, var(--ai-green) 15%, transparent)' }}
```

13 处逐一按此模式处理；`text-ai-*/N`、`border-ai-*/N`、`hover:bg-ai-*/N` 同理（hover 场景若无法内联 style，用 onMouseEnter/Leave 过重——勘察确认 13 处均为静态徽章/容器底色或可用 style 表达的场景，执行时若遇 hover 修饰符以 color-mix 写进既有 hover 样式块/按锚点现状选择最小改法）。每处改完核对原 N% 数值保持一致（如 `/15`→`15%`、`/70`→`70%`）。

- [ ] **Step 2: 回归门**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | tail -3 && npm run format:check`
Expected: tsc 0 error；vitest 564 passed / 3 skipped 不变；format 通过

Run: `grep -rn "ai-[a-z-]*/[0-9]" src-frontend/src --include="*.tsx" | grep -v "arbitrary" || true`
Expected: 13 处目标全部消失（输出中不再有勘察列出的 13 处；若别处有新命中需人工甄别，不在本批范围）

- [ ] **Step 3: Commit**

```bash
git add src-frontend/src/pages/Tasks.tsx src-frontend/src/components/ui/ai/{AiSearchList,AiCodeBlock}.tsx src-frontend/src/pages/settings/PromptsPanel.tsx
git commit -m "fix: /N 透明度修饰符失效 13 处改 color-mix 内联（P4 Task4）"
```

---

### Task 5: A 组 P1-P3 替换残留删除（TS 13 处 + frontstage.css 约 40 死类）

**Files:**
- Modify: `src-frontend/src/frontstage/components/GenesisPanel.tsx`（`getStatusColor` 函数 L206、`GenesisStepData` import L36、`setSelectedGenesisSessionId` L63、`isActive` 解构 L69，共 4 处）
- Modify: `src-frontend/src/frontstage/components/NovelCreationWizard.tsx`（React 默认导入 L1、`RefreshCw` L11，共 2 处）
- Modify: `src-frontend/src/frontstage/FrontstageBottomBar.tsx`（`abbreviateApiBase` L56-63、`formatTimeAgo` L65-74，共 2 函数）
- Modify: `src-frontend/src/pages/Tasks.tsx`（`loggedInvoke` L3、`TaskLog`+`RewriteSegment` 类型 import L20-21，共 3 import）
- Modify: `src-frontend/src/pages/Skills.tsx`（`selectedSkill` state 值从不读 L71——删 state 或删 setter 用法按锚点现状最小化处理，注意只删不读的"值"， setter 侧调用若存在需一并评估）
- Modify: `src-frontend/src/pages/UsageStats.tsx`（`getStoryLlmCalls` import L4）
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`（死类删除，见下）

**Interfaces:**
- Consumes: 无
- Produces: 无；纯删除

**勿误删（勘察实测活着）**：Tasks.tsx `TaskRow`(L63) / `CascadeRewriteDetail`(L202)；AgencyEval.tsx `CheckpointCompare`(L82)；PromptsPanel.tsx `writeJsonViaDialog`(L156)；frontstage.css `.zen-mode-exit`（与要删的 `.zen-exit-hint` 一字之差，严禁连带）。

**frontstage.css 死类清单（勘察 grep 零引用，约 40 个）：**
- 旧 chat 输入区整块：`.chat-toolbar` / `.chat-history` / `.chat-message` / `.chat-input-*` 系列 / `.chat-textarea` / `.chat-send-btn` / `.chat-hint` / `.chat-ghost-text` / `.ghost-hint-generating`
- 旧 model 展示：`.model-id` / `.model-url` / `.model-tooltip-label` / `.model-switch-*`
- 旧流式渲染：`.streaming-text-container` / `.ai-generating-text` / `.ai-cursor`
- 旧生成浮层：`.ai-generation-*` / `.ai-gen-btn-*`
- `.zen-exit-hint`
- index.css / tokens.css 无零引用类，不动。

- [ ] **Step 1: 删前 grep 复核**

Run: 对 TS 13 处每个符号与每个 CSS 类名逐一 `grep -rn "<符号>" src-frontend/src`，确认命中仅为定义处本身（CSS 类名确认 .tsx/.ts 中零 className 引用）
Expected: 与勘察结论一致；若某处出现新引用（勘察后有新代码），**跳过该处**并在 commit message 注明，不强行删

- [ ] **Step 2: 删除**

TS 13 处逐一删除（tsc 的 TS6133 已实测确认全部为未使用）；frontstage.css 死类整块删除，连带其 @keyframes（若有且仅被死类引用）一并删，`.zen-mode-exit` 保留。

- [ ] **Step 3: 回归门**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | tail -3 && npm run format:check`
Expected: tsc 0 error（删除后不得引入新 TS6133 级联——若有级联未使用符号出现，一并按同规则删除或回退该处）；vitest 564 passed / 3 skipped 不变（残留无自带测试）；format 通过

Run: `python3 scripts/architecture_guard.py`（仓库根）
Expected: 退出码 0

- [ ] **Step 4: Commit**

```bash
git add -A src-frontend/src
git commit -m "chore: 删除 P1-P3 替换残留 TS 13 处与 frontstage 死 CSS 约 40 类（P4 Task5）"
```

---

### Task 6: B 组历史死件删除（8 件 + 自带测试 + barrel + 级联死 CSS）

**Files:**
- Delete: `src-frontend/src/frontstage/components/AiSuggestionBubble.tsx`（324 行）
- Delete: `src-frontend/src/frontstage/components/AiHintOverlay.tsx`（142 行，仅 barrel 导出引用）
- Delete: `src-frontend/src/frontstage/components/HelpPanel.tsx`（74 行）
- Delete: `src-frontend/src/frontstage/components/ZenModeExit.tsx`（21 行）
- Delete: `src-frontend/src/hooks/useLlmStream.ts`（169 行）
- Delete: `src-frontend/src/hooks/useStudioConfig.ts`（114 行）
- Delete: `src-frontend/src/frontstage/utils/hetiAddon.ts`（204 行）
- Delete: `src-frontend/src/components/ui/Toggle.tsx`（35 行）
- Delete: `src-frontend/src/components/ui/__tests__/Toggle.test.tsx`（3 例）
- Delete: `src-frontend/src/frontstage/components/__tests__/HelpPanel.test.tsx`（3 例）
- Delete: `src-frontend/src/frontstage/components/__tests__/ZenModeExit.test.tsx`（2 例）
- Modify: `src-frontend/src/frontstage/components/index.ts`（L3 删 `export { AiHintOverlay } from './AiHintOverlay';`）
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`（级联死 CSS，删组件后 grep 确认再删：`.ai-hint-overlay` / `.ai-hint-bubble`（AiHintOverlay）、`.floating-hint-*`（AiSuggestionBubble））

**Interfaces:**
- Consumes: 无
- Produces: 无；纯删除。**vitest 计数预期 −8**（Toggle 3 + HelpPanel 3 + ZenModeExit 2），564 → 556 passed / 3 skipped。

起草时已实地复核（grep）：8 件均仅被自身/自身测试/barrel 引用；`FrontstageApp.tsx` 与 `useFrontstagePanels.ts` 中的 `showHelpPanel`/`toggleHelpPanel` 是另一套内联面板 state，**不 import HelpPanel 组件**，不在本批范围（不动）。

**B2（死导出）/B3（历史 CSS）建议——不纳入本批**：B2（errorHandler 4/logger 4/genesisSteps 2/useTextAnnotations 8/useChapters 2 等）是内部 API 表面，删除零运行时收益但显著扩大 diff 面与评审成本；B3（.slash-command-*/.smart-hint-*/.smart-ambient-*/.free-hint-*/.subscription-status/.writing-style-* 等）存在误报风险（勘察明确 `.ProseMirror-focused` 必须保留），需逐类人工核实，ROI 低。两者建议作为 P4 之后的独立可选清理批，记入 Task 10 文档「已知遗留」。

- [ ] **Step 1: 删前 grep 复核**

Run: 对 8 件逐一 `grep -rn "<名字>" src-frontend/src`，确认命中仅为自身文件、自身测试、barrel（AiHintOverlay）
Expected: 与勘察/起草复核一致；若出现新引用，该件**跳过**并注明

- [ ] **Step 2: 删除文件 + barrel**

删除上述 8 个源文件与 3 个测试文件；`frontstage/components/index.ts` L3 删 AiHintOverlay 导出行。

- [ ] **Step 3: 级联死 CSS**

Run: `grep -rn "ai-hint-overlay\|ai-hint-bubble\|floating-hint" src-frontend/src --include="*.tsx" --include="*.ts"`
Expected: 零命中后，删 frontstage.css 中 `.ai-hint-overlay` / `.ai-hint-bubble` / `.floating-hint-*` 定义块

- [ ] **Step 4: 回归门**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | tail -3 && npm run format:check`
Expected: tsc 0 error；vitest **556 passed / 3 skipped**（−8 为死件自带测试，允许下降；若下降数 ≠ 8 需查明原因再继续）；format 通过

Run: `python3 scripts/architecture_guard.py`（仓库根）
Expected: 退出码 0

- [ ] **Step 5: Commit**

```bash
git add -A src-frontend/src
git commit -m "chore: 删除历史死件 8 件（AiSuggestionBubble/AiHintOverlay/HelpPanel/ZenModeExit/useLlmStream/useStudioConfig/hetiAddon/Toggle）及自带测试与级联死 CSS（P4 Task6）"
```

---

### Task 7: AgencyEval 浅色页令牌化（S-M）

**Files:**
- Modify: `src-frontend/src/pages/AgencyEval.tsx`（hex 直写 6 处 → `var(--ai-*)`、`text-gray-*` → `text-ai-ink-2`/`text-ai-ink-3`、select 补 `border-ai-line bg-ai-field text-ai-ink`）

**Interfaces:**
- Consumes: `--ai-*` 令牌（含 Task 3 新增 `--ai-on-accent`，本页预计用不到）
- Produces: 无新 API

说明：P2/P3 留下的风格切口在此关闭。只换颜色来源，不改布局结构/间距/字号；L31 已有 color-mix 内联先例，新增 tint 缺口沿用同一写法。既有 `AgencyEval.test.tsx`（61 行，文本断言，无类名/样式断言——起草时已核实）必须保持全绿。

- [ ] **Step 1: 逐处令牌化**

执行时先 `grep -nE "text-gray|bg-white|#[0-9a-fA-F]{3,6}|border-gray|bg-gray" src-frontend/src/pages/AgencyEval.tsx` 取现状清单（行号以勘察为初始参考），逐处替换：hex → 对应 `var(--ai-*)`（金色系 → `--ai-accent`/`--ai-accent-ink`，红绿橙 → `--ai-red`/`--ai-green`/`--ai-orange`）；`text-gray-500/400` → `text-ai-ink-3`、`text-gray-600/700` → `text-ai-ink-2`；select 元素补 `border-ai-line bg-ai-field text-ai-ink` 一套。tint 缺口 color-mix 内联，不扩令牌。

- [ ] **Step 2: 回归门**

Run: `cd src-frontend && npx vitest run src/pages/__tests__/AgencyEval.test.tsx && npx tsc --noEmit && npx vitest run 2>&1 | tail -3 && npm run format:check`
Expected: 页测试全绿不回归；tsc 0 error；全量 556 passed / 3 skipped；format 通过

- [ ] **Step 3: 目视确认 + Commit**

`npm run dev` 或既有预览手段目视 AgencyEval 页（暗色 cinema 主题下无色块断裂）。然后：

```bash
git add src-frontend/src/pages/AgencyEval.tsx
git commit -m "refactor: AgencyEval 浅色裸样式令牌化（P4 Task7）"
```

---

### Task 8: AgencyStudio 浅色页令牌化（S-M）

**Files:**
- Modify: `src-frontend/src/pages/AgencyStudio.tsx`（L283 `bg-white` select、L307 `red-50` 错误条、裸卡片 ×2、`text-gray-*` 9 处）

**Interfaces:**
- Consumes: `--ai-*` 令牌
- Produces: 无新 API

说明：同 Task 7 切口关闭。既有 `AgencyStudio.test.tsx`（194 行，无类名/样式断言——起草时已核实）必须保持全绿。

- [ ] **Step 1: 逐处令牌化**

`grep -nE "text-gray|bg-white|red-50|#[0-9a-fA-F]{3,6}|border-gray" src-frontend/src/pages/AgencyStudio.tsx` 取现状清单。L283 select：`bg-white` → `bg-ai-field`，补 `border-ai-line text-ai-ink`。L307 错误条：`red-50` 系浅色底 → `color-mix(in srgb, var(--ai-red) 12%, transparent)` 内联 + `text-ai-red`（同 AgencyEval.tsx:31 先例）。裸卡片 ×2：补 `border-ai-line bg-ai-surface` 容器样式（只补颜色来源，不动卡片结构与 padding）。`text-gray-*` 9 处按 Task 7 同一映射表替换。

- [ ] **Step 2: 回归门**

Run: `cd src-frontend && npx vitest run src/pages/__tests__/AgencyStudio.test.tsx && npx tsc --noEmit && npx vitest run 2>&1 | tail -3 && npm run format:check`
Expected: 页测试全绿不回归；tsc 0 error；全量 556 passed / 3 skipped；format 通过

- [ ] **Step 3: 目视确认 + Commit**

```bash
git add src-frontend/src/pages/AgencyStudio.tsx
git commit -m "refactor: AgencyStudio 浅色裸样式令牌化（P4 Task8）"
```

---

### Task 9: AgencyLearning 浅色页令牌化 + 裸 table → AiRecordsTable（M）

**Files:**
- Modify: `src-frontend/src/pages/AgencyLearning.tsx`（整页约 30 处直写类名；裸 table L164-186 → AiRecordsTable；L85 `border-amber-300 bg-amber-50` 暗主题下亮黄卡最刺眼，优先处理）

**Interfaces:**
- Consumes: P3 已入库 `AiRecordsTable`（`@/components/ui/ai/AiRecordsTable`；`columns`/`rows`/`rowKey` 受控，同构 AgencyEval 判定历史/token 用量表用法）；`--ai-*` 令牌
- Produces: 无新 API

说明：本批最重的 Task，拆成两个 Step（先令牌化，再换表），一次 commit。裸 table → AiRecordsTable 参照 P3 Task5 中 AgencyEval 双表的既有用法（columns 定义 + rows 映射 + rowKey），不改数据逻辑、不改页面结构。既有 `AgencyLearning.test.tsx`（78 行，无类名/样式断言——起草时已核实）必须保持全绿：换表时注意表内文本内容（表头、单元格文案）原样保留，使文本断言不回归。

- [ ] **Step 1: 整页令牌化**

`grep -nE "text-gray|bg-white|amber-|#[0-9a-fA-F]{3,6}|border-gray" src-frontend/src/pages/AgencyLearning.tsx` 取现状清单，约 30 处按 Task 7/8 同一映射表替换。L85 `border-amber-300 bg-amber-50` → `border-ai-line` + `color-mix(in srgb, var(--ai-orange) 12%, transparent)` 内联底 + `text-ai-orange`（amber 在 ai 调色板无对应，映射 orange，同 P3 UsageStats 色调映射先例）。

- [ ] **Step 2: 裸 table → AiRecordsTable**

L164-186 裸 `<table>` 按 AgencyEval 既有 AiRecordsTable 用法改写：`columns`（key/header/可选 render）+ `rows` + `rowKey`；表头与单元格文案原样保留；不开 selection/sort/footer（AgencyLearning 无此需求，最小替换）。

- [ ] **Step 3: 回归门**

Run: `cd src-frontend && npx vitest run src/pages/__tests__/AgencyLearning.test.tsx && npx tsc --noEmit && npx vitest run 2>&1 | tail -3 && npm run format:check`
Expected: 页测试全绿不回归；tsc 0 error；全量 556 passed / 3 skipped；format 通过

- [ ] **Step 4: 目视确认 + Commit**

```bash
git add src-frontend/src/pages/AgencyLearning.tsx
git commit -m "refactor: AgencyLearning 令牌化并将裸 table 替换为 AiRecordsTable（P4 Task9）"
```

---

### Task 10: 全量回归门 + 文档同步（P4 关闭）

**Files:**
- Modify: `CHANGELOG.md`（文件头 Unreleased 区插入 P4 段，锚点为最近一个版本标题前）
- Modify: `PROJECT_STATUS.md`（`## ✅ 最近完成功能` 下插入 P4 条目）
- Modify: `AGENTS.md`（编码风格节 AI 原生组件行更新：16 变量契约 → 17 变量契约，补 `--ai-on-accent`）
- Modify: `docs/plans/2026-08-12-beautifului-ai-native-design.md`（§8 第 5 条 P4 行行尾追加关闭结论）

**Interfaces:**
- Consumes: Task 1-9 全部产出
- Produces: P4 阶段关闭记录

- [ ] **Step 1: 全量回归门（总闸）**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | tail -3 && npm run format:check`
Expected: tsc 0 error；vitest **预期 556 passed / 3 skipped**（基线 564 − Task6 死件自带测试 8；以实际输出为准并记录进 CHANGELOG）；format 通过

Run: `python3 scripts/architecture_guard.py`（仓库根）
Expected: 退出码 0

Run: `git diff --stat $(git merge-base HEAD master~20 2>/dev/null || echo HEAD) -- '*.rs' | tail -1 || true`（或 `git status` 人工确认本阶段无 rust 文件改动）
Expected: 无 rust 改动 → `cargo test --lib` 不重跑（基线 1328 passed / 2 ignored 不变）；若有 rust 改动则必须 `cd src-tauri && cargo test --lib` 并记录结果

- [ ] **Step 2: 文档同步（版本号不动，发版另行进行）**

**(a) `CHANGELOG.md`** — Unreleased 区插入：

```markdown
## Unreleased（P4 AI 原生组件库 · 收尾）

### 清理与修复：beautifului AI 原生组件替换项目收尾（设计文档 P4 范围）

- **替换残留删除**：P1-P3 未使用符号 13 处（GenesisPanel 4 / NovelCreationWizard 2 / FrontstageBottomBar 2 函数 / Tasks 3 import / Skills 1 / UsageStats 1 import）+ frontstage.css 旧 chat 输入区/旧 model 展示/旧流式渲染/旧生成浮层等约 40 个零引用死类（`.zen-mode-exit` 等活类保留）。
- **历史死件删除**：AiSuggestionBubble / AiHintOverlay / HelpPanel / ZenModeExit / useLlmStream / useStudioConfig / hetiAddon / Toggle 共 8 件，含自带测试 3 个文件与 barrel 导出、级联死 CSS。
- **视觉修复**：新增 `--ai-on-accent` 语义令牌（第 17 变量，幕后幕前均 #ffffff，四文件同步），替换四组件 text-white 直写；`/N` 透明度修饰符失效 13 处改 color-mix 内联（徽章全饱和 bug 修复）；Tasks 末处裸 pre 换 AiCodeBlock；AiDiffTable testid 改 per-row key。
- **浅色页令牌化**：AgencyEval / AgencyStudio / AgencyLearning 三页关闭 P2/P3 风格切口（AgencyLearning 裸 table → AiRecordsTable），只换颜色来源不改布局。

### 测试

- src-frontend `npx vitest run`：**<以 Task10 Step1 实际输出填写> passed / 3 skipped**（P3 基线 564 − 死件自带测试 8）。
- 已知遗留（后续可选清理，不在 P4 范围）：B2 死导出（errorHandler/logger/genesisSteps/useTextAnnotations/useChapters 等约 20 个）；B3 历史 CSS（.slash-command-*/.smart-hint-*/.free-hint-* 等，需逐类核实防误报）。
```

**(b) `PROJECT_STATUS.md`** — `## ✅ 最近完成功能` 下插入：

```markdown
### Unreleased - beautifului AI 原生组件 P4（收尾：清理 + 视觉修正 + 浅色页令牌化）（2026-08-13）

- **清理**：P1-P3 替换残留 TS 13 处 + 死 CSS 约 40 类；历史死件 8 件（含自带测试/barrel/级联 CSS）。
- **修正**：`--ai-on-accent` 令牌（17 变量契约）替换 text-white ×4；/N 透明度失效 13 处 color-mix 修复；Tasks 裸 pre → AiCodeBlock；AiDiffTable testid per-row key。
- **令牌化**：AgencyEval / AgencyStudio / AgencyLearning 浅色切口关闭（AgencyLearning 裸表 → AiRecordsTable）。
- **验证**：`npx tsc --noEmit` / `npx vitest run`（<实际数> passed / 3 skipped）/ `format:check` / `architecture_guard.py` 全绿；Rust 无改动（1328 passed / 2 ignored 不变）。版本号未动，发版推送另行进行。
```

**(c) `AGENTS.md`** — 编码风格节 AI 原生组件行中「`--ai-*` 语义令牌」相关表述更新：16 变量契约 → **17 变量契约（P4 新增 `--ai-on-accent`）**，其余约定（tint color-mix 内联零扩、受控组件、不引演示逻辑）不变。

**(d) 设计文档** — `docs/plans/2026-08-12-beautifului-ai-native-design.md` §8 第 5 条 P4 行行尾追加：

```markdown
（P4 已完成：替换残留与历史死件清理、`--ai-on-accent` 令牌与 /N 透明度修复、AgencyEval/AgencyStudio/AgencyLearning 浅色页令牌化；B2 死导出与 B3 历史 CSS 留作后续可选清理；发版推送不在 P4 范围，详见 P4 实施计划）
```

- [ ] **Step 3: Commit**

```bash
git add CHANGELOG.md PROJECT_STATUS.md AGENTS.md docs/plans/2026-08-12-beautifului-ai-native-design.md
git commit -m "docs: P4 收尾回归门与文档同步，设计文档 §8 P4 关闭（P4 Task10）"
```

---

## 全量回归清单（每个 Task 末尾必过；Task 10 Step1 为总闸）

| 检查项 | 命令 | 通过标准 |
| --- | --- | --- |
| 类型检查 | `cd src-frontend && npx tsc --noEmit` | 0 error |
| 令牌/组件定向测试 | Task3：`npx vitest run src/styles/__tests__/aiTokens.test.ts`；Task2：AiDiffTable 测试；Task7/8/9：对应页测试 | 全绿（Task3 Step2 要求先红） |
| 全量前端测试 | `npx vitest run` | 基线 564 passed / 3 skipped；Task 6 后 556（−8 死件自带测试，允许下降）；其余 Task 不变 |
| 删前零引用验证（删除批） | 逐符号/类名 `grep -rn` 全仓（含测试、barrel、CSS 三口径） | 命中仅为定义处自身；不一致则跳过该处并注明 |
| 删后残留验证（删除批） | 同上 grep + tsc 无新 TS6133 级联 | 无残留引用、无级联未使用符号 |
| 格式化 | `npm run format:check` | 通过（pre-commit fmt 不绕过，不 --no-verify） |
| 架构守卫 | `python3 scripts/architecture_guard.py`（仓库根） | 退出码 0 |
| Rust 测试 | 不重跑（纯前端阶段）；若意外动 rust 则 `cd src-tauri && cargo test --lib` | 基线 1328 passed / 2 ignored 不变 |
| Commit 规范 | 中文 conventional commit，不推送不打 tag | 每个 Task 独立 commit + 评审 |

## 预期 vitest 计数变化表

| 阶段 | 变化 | passed / skipped |
| --- | --- | --- |
| P3 完成基线 | — | 564 / 3 |
| Task 1 裸 pre → AiCodeBlock | 0（组件行为由 P3 AiCodeBlock 测试覆盖） | 564 / 3 |
| Task 2 testid per-row key | 0（testid 无消费者，组件测试未断言它） | 564 / 3 |
| Task 3 `--ai-on-accent` | 0（aiTokens 两数组扩充断言，用例数 5 不变） | 564 / 3 |
| Task 4 /N 修复 | 0 | 564 / 3 |
| Task 5 A 组残留删除 | 0（残留无自带测试） | 564 / 3 |
| Task 6 B 组死件删除 | **−8**（Toggle.test 3 + HelpPanel.test 3 + ZenModeExit.test 2） | **556 / 3** |
| Task 7/8/9 浅色页令牌化 | 0（三页既有测试只断言文本/角色，无类名/样式断言——已核实） | 556 / 3 |
| Task 10 收尾（总闸实测记录） | — | 预期 556 / 3，以实际输出写入 CHANGELOG |

## 文件清单汇总表

| 文件 | Task | 动作 |
| --- | --- | --- |
| `src-frontend/src/pages/Tasks.tsx` | 1 / 4 / 5 | 修改（L392 裸 pre → AiCodeBlock / L130-315 共 8 处 /N 修复 / L3+L20-21 死 import 删除——三 Task 区域不重叠） |
| `src-frontend/src/components/ui/ai/AiDiffTable.tsx` | 2 | 修改（L107 testid per-row key） |
| `src-frontend/src/styles/tokens.css` | 3 | 修改（L54 后加 `--ai-on-accent`） |
| `src-frontend/src/frontstage/styles/frontstage.css` | 3 / 5 / 6 | 修改（AI 令牌块加 `--ai-on-accent` / 约 40 死类删除 / `.ai-hint-*`+`.floating-hint-*` 级联死类删除——区域不重叠） |
| `src-frontend/tailwind.config.js` | 3 | 修改（ai 色组加 `on-accent` 映射） |
| `src-frontend/src/styles/__tests__/aiTokens.test.ts` | 3 | 修改（AI_VARS / AI_COLOR_KEYS 两数组各加一项） |
| `src-frontend/src/components/ui/ai/AiApprovalCard.tsx` / `AiTaskRows.tsx` / `AiContextCards.tsx` / `AiRecommendationCard.tsx` | 3 | 修改（各 1 处 text-white → text-ai-on-accent） |
| `src-frontend/src/components/ui/ai/AiSearchList.tsx` / `AiCodeBlock.tsx` | 4 | 修改（L56 / L109 /N 修复） |
| `src-frontend/src/pages/settings/PromptsPanel.tsx` | 4 | 修改（L664 / L724 /N 修复） |
| `src-frontend/src/frontstage/components/GenesisPanel.tsx` | 5 | 修改（4 处死符号删除） |
| `src-frontend/src/frontstage/components/NovelCreationWizard.tsx` | 5 | 修改（2 处死 import 删除） |
| `src-frontend/src/frontstage/FrontstageBottomBar.tsx` | 5 | 修改（2 个死函数删除） |
| `src-frontend/src/pages/Skills.tsx` / `UsageStats.tsx` | 5 | 修改（各 1 处死符号/import 删除） |
| `src-frontend/src/frontstage/components/AiSuggestionBubble.tsx` / `AiHintOverlay.tsx` / `HelpPanel.tsx` / `ZenModeExit.tsx` | 6 | 删除 |
| `src-frontend/src/hooks/useLlmStream.ts` / `useStudioConfig.ts` | 6 | 删除 |
| `src-frontend/src/frontstage/utils/hetiAddon.ts` | 6 | 删除 |
| `src-frontend/src/components/ui/Toggle.tsx` | 6 | 删除 |
| `src-frontend/src/components/ui/__tests__/Toggle.test.tsx` / `frontstage/components/__tests__/HelpPanel.test.tsx` / `ZenModeExit.test.tsx` | 6 | 删除（共 −8 用例） |
| `src-frontend/src/frontstage/components/index.ts` | 6 | 修改（L3 删 AiHintOverlay barrel 导出） |
| `src-frontend/src/pages/AgencyEval.tsx` | 7 | 修改（令牌化，勿动 CheckpointCompare L82） |
| `src-frontend/src/pages/AgencyStudio.tsx` | 8 | 修改（令牌化） |
| `src-frontend/src/pages/AgencyLearning.tsx` | 9 | 修改（令牌化 + 裸 table → AiRecordsTable） |
| `CHANGELOG.md` / `PROJECT_STATUS.md` / `AGENTS.md` / `docs/plans/2026-08-12-beautifului-ai-native-design.md` | 10 | 修改（文档同步，§8 P4 行关闭） |

## 依赖顺序图

```
P1-P3 令牌桥与 16 组件（已入库）
  │
  ├─ Task1 Tasks.tsx L392 裸 pre → AiCodeBlock（S，独立区域）
  ├─ Task2 AiDiffTable testid per-row key（S，零消费者，独立）
  ├─ Task3 --ai-on-accent 令牌 + text-white ×4（S；四文件同步同一 commit；TDD 先红后绿）
  ├─ Task4 /N 失效 13 处 color-mix 修复（S；Tasks.tsx 与 Task1 区域不重叠）
  │
  ├─ Task5 A 组替换残留删除（TS 13 处 + frontstage.css 约 40 死类；删前 grep 复核）
  ├─ Task6 B 组历史死件删除（8 件 + 测试 −8 + barrel + 级联 CSS；依赖 Task5 先清小残留降低干扰）
  │
  ├─ Task7 AgencyEval 令牌化（S-M；消费 Task3 令牌体系与既有 color-mix 先例）
  ├─ Task8 AgencyStudio 令牌化（S-M）
  ├─ Task9 AgencyLearning 令牌化 + 裸 table → AiRecordsTable（M；消费 P3 AiRecordsTable）
  │
  └─ Task10 全量回归门 + 文档同步（依赖 Task1-9 全部；发版推送不在内）

排序理由（风险递增）：
- Task1-4 为小切口修正，改动面小、验证直接、可随时回退，先做可让后续删除批的 diff 更干净
  （如 Tasks.tsx 的活代码先定型，删 import 时不易误伤）。
- Task5/6 删除批居中：回归门验证（tsc 会兜住任何误判引用），且删除后代码面收缩，
  降低 Task7-9 令牌化时的 grep 噪音。
- Task7-9 浅色页令牌化最后做：改动面最大、需要目视确认，且消费 Task3 落地后的完整令牌体系
  与 AiRecordsTable（P3）稳定形态；每页独立 Task 独立 commit，风险隔离。
- Task10 总闸收尾。
```

## Self-Review 结论

- **Spec coverage（对照任务书 + p4-recon-summary.md 逐条）**：
  - A 组 TS 13 处 + 约 40 死类 → Task 5，勿误删清单（TaskRow L63 / CascadeRewriteDetail L202 / CheckpointCompare L82 / writeJsonViaDialog L156 / .zen-mode-exit）写入 Global Constraints 与 Task 5 正文。
  - B 组 8 件 + 3 测试文件 + barrel + 级联 CSS → Task 6；B2/B3 建议「不纳入本批、记入已知遗留」写入 Task 6 说明与 Task 10 CHANGELOG 模板。
  - text-white ×4 + `--ai-on-accent` 四文件同步 → Task 3（TDD 先红后绿，数组断言扩充、用例数不变；值幕后幕前均 #ffffff、不引用仅幕前有的 var(--text-on-accent)——照勘察原话落实）。
  - /N 失效 13 处（Tasks 8 + AiSearchList 1 + AiCodeBlock 1 + PromptsPanel 2）→ Task 4，color-mix 内联、不改 tailwind 配置。
  - 浅色页三页 → Task 7/8/9 每页一个 Task（AgencyLearning 含裸 table → AiRecordsTable、L85 亮黄卡优先）。
  - Tasks.tsx:392 → Task 1；ai-diff-delta → Task 2；收尾回归门 + 文档同步（§8 P4 行关闭、发版不在内）→ Task 10。
- **行号口径**：全部抄自勘察基线并声明漂移规则（Global Constraints 行号口径条）；起草时实地复核了 B 组 8 件引用面（仅自身/测试/barrel）、三个死件测试用例数（3+3+2=8）、`ai-diff-delta` 无消费者、三页既有测试无类名/样式断言、aiTokens.test.ts 数组结构、tokens.css 令牌块位置、AiDiffTable.tsx:109-115 与 AgencyEval.tsx:31 的 color-mix 先例形态。
- **Placeholder scan**：全文无 TBD；唯一待定值为 Task 10 文档模板中 `<以 Task10 Step1 实际输出填写>`/`<实际数>`——设计上必须由执行者填入的真实命令输出，非占位符。
- **风险备注**：Skills.tsx L71 `selectedSkill`「值从不读」的删法需按锚点现状最小化（只删值保留 setter 或整体删 state，执行时判定）；Task 4 的 13 处若遇 hover 场景修饰符无法内联，以既有 hover 样式块最小改法处理（已写入 Task 4 Step 1）。
