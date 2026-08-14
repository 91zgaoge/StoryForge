# 墨纸 / 机械视觉进化 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 按 `docs/plans/2026-08-13-ink-paper-visual-evolution-design.md` 把幕前墨纸、幕后机械定向进化到安静贵气：P0 输入条样板入库，P1 本地霞鹜文楷，P2 色板/阴影收软，P3 press/spring 与去装饰脉冲，P4 幕后 bezel/侧栏减噪与剩余实心块，P5 空态统一。

**Architecture:** 不改 IPC / Agency / PersistMode。令牌仍是双窗各自定义的 `--ai-*` 17 变量 + 幕前 OKLCH / 幕后 `backstageThemes`。新运动令牌 `--transition-press` 与已有 `--transition-spring` 并列，经 `prefers-reduced-motion` 冻结。字体 woff2 进 `src-frontend/public/fonts/`（Vite 拷入 dist，Tauri 吃 `frontendDist`），去掉幕前 HTML 里的运行时 CDN。

**Tech Stack:** React 18 + Tailwind v3.4、vitest 4 + Testing Library、lucide-react（不换库）、霞鹜文楷 SIL OFL（本地 woff2）。零新 npm 运行时依赖。

**需求来源：** `docs/plans/2026-08-13-ink-paper-visual-evolution-design.md`。前序：`2026-07-27-ui-redesign-design.md`（Ghost Chrome / 纯黑阴影 / `scale-95` 已作废）、`2026-08-12-beautifului-ai-native-design.md`（`--ai-*` 契约保留）。

---

## Global Constraints

- 仓库 `/Users/yuzaimu/projects/StoryForge`；master 直接工作；中文 conventional commit；不 `--no-verify`；**不推送、不打 tag、不 bump 版本**，发版等用户指令。
- **Commit 步骤**：仅当用户在本会话明确说「提交」时执行各 Task 末的 commit。未授权则做完代码+测试停在工作区。
- **禁止**：岛式顶栏、`py-24`、Bento、GSAP、新图标包、运行时字体 CDN、复活 Ghost Chrome、炭黑填充发射键、幕前双层边框、空闲 `animate-pulse` / `animate-ping`。
- **骨架屏**（数据未到，如 `IntentionGraphDiagnostics` 的占位块）不算空闲装饰，P3 **不要**删。
- **`--ai-*` 17 变量**两窗口必须始终各自定义。P2 只改映射值，不增删变量名。
- **`backstageThemes.test.ts`** 的 `TOKENS_CSS_CURRENT` 必须与 `tokens.css` + `backstageThemes.warm.vars` 三方同 commit 改，否则「warm 零回归」测试会红——P2 是**有意的视觉变化**，测试期望值跟着新值走，不是保持 `#050508`。
- 纯前端。不改 Rust。误碰 rust 则必须重跑 `cargo test --lib`。
- 每阶段对照设计 §13 用户可感知项；只改 className 未看三态不得勾完成。
- 准入线（每 Task 回归）：`cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check`；仓库根 `python3 scripts/architecture_guard.py`。vitest 只允许增加。
- 行号会漂，执行以**锚点代码**定位。

## File map（分解锁定）

| 文件 | 职责 |
|---|---|
| `src-frontend/src/components/ui/ai/AiPromptBar.tsx` | `variant` flush/card；发射键陶土淡彩（P0 已改） |
| `src-frontend/src/frontstage/components/FrontstageBottomBar.tsx` | 一层纸面、取消键、信号分隔、去 ping（P0 部分已改） |
| `src-frontend/src/frontstage/styles/frontstage.css` | 幕前 token、`@font-face`、幽灵偏移、`--shadow-float`、`--transition-press` |
| `src-frontend/frontstage.html` | 删 jsDelivr / Google Fonts |
| `src-frontend/public/fonts/*` | 霞鹜文楷 woff2 + OFL |
| `src-frontend/src/styles/tokens.css` | 幕后 950/阴影/velvet/状态色、`--transition-press` |
| `src-frontend/src/styles/backstageThemes.ts` | 四套色调新值 |
| `src-frontend/src/frontstage/config/colorThemes.ts` | ink 抬升公式 |
| `src-frontend/src/frontstage/config/writingStyles.ts` | 默认 inkColor 对齐 |
| `src-frontend/tailwind.config.js` | `ease-press` |
| `src-frontend/src/components/ui/Button.tsx` | press 曲线；cinema/paper 淡彩 |
| `src-frontend/src/components/ui/Panel.tsx` | double-bezel |
| `src-frontend/src/components/Sidebar.tsx` | 热温冷徽章低对比 |
| `src-frontend/src/components/ui/ai/AiSelectionActions.tsx` | 主按钮不再炭黑实心 |
| `src-frontend/src/pages/AgencyStudio.tsx` | 空态抄共享样式 |
| `src-frontend/src/pages/Tasks.tsx` | 空态抄共享样式 |

---

### Task 1: P0 输入条样板封板

**Files:**
- Modify（工作区应已有，执行时先 `git diff` 确认）:
  - `src-frontend/src/components/ui/ai/AiPromptBar.tsx`
  - `src-frontend/src/frontstage/components/FrontstageBottomBar.tsx`
  - `src-frontend/src/frontstage/styles/frontstage.css`（`.frontstage-input-ghost` `top: 5px; left: 4px`）
  - `src-frontend/src/components/ui/ai/__tests__/AiPromptBar.test.tsx`
  - `src-frontend/src/frontstage/components/__tests__/FrontstageBottomBar.test.tsx`

**Interfaces:**
- `AiPromptBarProps.variant?: 'card' | 'flush'`（默认 `card`）
- 幕前必须 `variant="flush"`
- 发射：空 = 透明 + `--ai-ink-3`；有内容 = `color-mix(in oklch, var(--ai-accent) 18%, transparent)` + `--ai-accent-ink`
- 取消：`size-7 rounded-md`，无 `animate-pulse`、无 `status-danger`

若 `git diff` 显示上述尚未落地，按设计 §9.1 补齐，不要发明第二套样式。

- [ ] **Step 1: 写/确认失败契约已在仓库**

`AiPromptBar.test.tsx` 必须含：

```tsx
it('flush 变体去掉内层边框，发送按钮仍可用', () => {
  const onSend = vi.fn();
  const { container } = render(
    <AiPromptBar variant="flush" value="写一段" onChange={() => {}} onSend={onSend} />
  );
  expect(screen.getByTestId('ai-prompt-bar')).toHaveAttribute('data-variant', 'flush');
  expect(
    container.querySelector('[data-testid="ai-prompt-bar"] > div:last-child')
  ).not.toHaveClass('border-ai-line');
  fireEvent.click(screen.getByTitle('发送'));
  expect(onSend).toHaveBeenCalledTimes(1);
});
```

`FrontstageBottomBar.test.tsx` 必须含：

```tsx
it('输入条走 flush 纸面，取消生成无脉冲红块', () => {
  const { rerender } = render(<FrontstageBottomBar {...defaultProps} inputValue="续写" />);
  expect(screen.getByTestId('ai-prompt-bar')).toHaveAttribute('data-variant', 'flush');
  expect(screen.getByTitle('发送')).toBeInTheDocument();
  rerender(<FrontstageBottomBar {...defaultProps} isGenerating={true} />);
  const cancel = screen.getByTitle('取消生成');
  expect(cancel.className).not.toMatch(/animate-pulse/);
  expect(cancel.className).not.toMatch(/status-danger/);
});
```

再加回归探针（可放在 `AiPromptBar.test.tsx` 末）：

```tsx
it('有内容时发射键不用 --ai-ink 实心填充', () => {
  render(<AiPromptBar value="写一段" onChange={() => {}} onSend={() => {}} />);
  const send = screen.getByTitle('发送');
  expect(send.getAttribute('style') ?? '').not.toMatch(/var\(--ai-ink\)/);
  expect(send.getAttribute('style') ?? '').toMatch(/--ai-accent/);
});
```

- [ ] **Step 2: 跑测试**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiPromptBar.test.tsx src/frontstage/components/__tests__/FrontstageBottomBar.test.tsx`

Expected: 全部 PASS（含新建探针）。若发射键探针 FAIL，把 `AiPromptBar` 默认发射 `style` 改成：

```tsx
style={{
  background: canSend
    ? 'color-mix(in oklch, var(--ai-accent) 18%, transparent)'
    : 'transparent',
  color: canSend ? 'var(--ai-accent-ink)' : 'var(--ai-ink-3)',
}}
```

并确认**没有** `background: canSend ? 'var(--ai-ink)'`。

- [ ] **Step 3: 用户可感知核验**

打开幕前：空输入发射键隐进纸面；输入几字后陶土淡彩；点续写后取消与发射同脚印、不闪红。幽灵提示与输入字对齐。

- [ ] **Step 4: 回归门**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check`

Expected: tsc 0 error；vitest 全绿（允许 +1 探针）；prettier 过。

- [ ] **Step 5: Commit**（仅用户授权后）

```bash
git add src-frontend/src/components/ui/ai/AiPromptBar.tsx \
  src-frontend/src/components/ui/ai/__tests__/AiPromptBar.test.tsx \
  src-frontend/src/frontstage/components/FrontstageBottomBar.tsx \
  src-frontend/src/frontstage/components/__tests__/FrontstageBottomBar.test.tsx \
  src-frontend/src/frontstage/styles/frontstage.css
git commit -m "$(cat <<'EOF'
style: 幕前输入条一层纸面与陶土淡彩发射（视觉进化 P0）

EOF
)"
```

---

### Task 2: P1 霞鹜文楷本地加载

**Files:**
- Create: `src-frontend/public/fonts/lxgwwenkai-regular.woff2`
- Create: `src-frontend/public/fonts/OFL.txt`（从字体包复制 SIL OFL）
- Create: `src-frontend/public/fonts/README.md`（版本钉死说明）
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`（`:root` 的 `--font-serif` 之前插入 `@font-face`）
- Modify: `src-frontend/frontstage.html`（删除 jsDelivr 与 Google Fonts 四条 link）
- Test: `src-frontend/src/styles/__tests__/aiTokens.test.ts`（追加字体探针；或新建 `src-frontend/src/frontstage/styles/__tests__/wenkaiFont.test.ts`）

**为什么不走 CDN：** 设计 §4.6 / §7。`frontstage.html` 现有 `lxgw-wenkai-webfont@1.7.0` 与 `fonts.googleapis.com` 在离线桌面里会静默失败，纸感塌成苹方。

- [ ] **Step 1: 写失败测试**

新建 `src-frontend/src/frontstage/styles/__tests__/wenkaiFont.test.ts`：

```ts
import { describe, it, expect } from 'vitest';
import { existsSync, readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const frontendRoot = resolve(__dirname, '..', '..', '..', '..');

describe('霞鹜文楷本地加载', () => {
  it('public/fonts 含 regular woff2 与 OFL', () => {
    expect(existsSync(resolve(frontendRoot, 'public/fonts/lxgwwenkai-regular.woff2'))).toBe(true);
    expect(existsSync(resolve(frontendRoot, 'public/fonts/OFL.txt'))).toBe(true);
  });

  it('frontstage.css 以 @font-face 声明 LXGW WenKai 且 font-display:swap', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    expect(css).toMatch(/@font-face\s*\{[^}]*font-family:\s*['"]LXGW WenKai['"]/s);
    expect(css).toMatch(/font-display:\s*swap/);
    expect(css).toMatch(/\/fonts\/lxgwwenkai-regular\.woff2/);
  });

  it('frontstage.html 不再请求 jsdelivr 或 fonts.googleapis', () => {
    const html = readFileSync(resolve(frontendRoot, 'frontstage.html'), 'utf-8');
    expect(html).not.toMatch(/jsdelivr/);
    expect(html).not.toMatch(/fonts\.googleapis/);
    expect(html).not.toMatch(/fonts\.gstatic/);
  });
});
```

- [ ] **Step 2: 跑测试确认红**

Run: `cd src-frontend && npx vitest run src/frontstage/styles/__tests__/wenkaiFont.test.ts`

Expected: FAIL（缺文件 / 缺 `@font-face` / HTML 仍有 CDN）。

- [ ] **Step 3: 钉死并拷贝字体（一次性，不把包留在 dependencies）**

```bash
cd /tmp
npm pack lxgw-wenkai-webfont@1.7.0
mkdir -p /tmp/lxgw-pack
tar -xzf lxgw-wenkai-webfont-1.7.0.tgz -C /tmp/lxgw-pack
# 列出 woff2 路径后拷贝 Regular（包内文件名以 tar -tzf 为准，常见为 files/ 或 dist/）
WOFF=$(find /tmp/lxgw-pack/package -iname '*regular*.woff2' | head -1)
test -n "$WOFF"
mkdir -p /Users/yuzaimu/projects/StoryForge/src-frontend/public/fonts
cp "$WOFF" /Users/yuzaimu/projects/StoryForge/src-frontend/public/fonts/lxgwwenkai-regular.woff2
# OFL：包内 LICENSE / OFL.txt 任一
OFL=$(find /tmp/lxgw-pack/package -iname 'OFL.txt' -o -iname 'LICENSE*' | head -1)
cp "$OFL" /Users/yuzaimu/projects/StoryForge/src-frontend/public/fonts/OFL.txt
```

`public/fonts/README.md` 全文：

```md
# LXGW WenKai（霞鹜文楷）

- 来源 npm：`lxgw-wenkai-webfont@1.7.0`（与历史 CDN 同源，改为本地）
- 许可：SIL Open Font License 1.1（见 OFL.txt）
- 本目录只打 Regular woff2。`font-weight: 400 500` 映射到同一文件，避免浏览器合成伪粗。
- 禁止在 HTML 里再引入 jsDelivr / Google Fonts 作为幕前正文来源。
```

体积：woff2 单档预期数 MB。禁止改打完整 TTF（~17MB×N）进安装包。

- [ ] **Step 4: `@font-face` + 删 CDN**

在 `frontstage.css` 文件最顶部（`@tailwind utilities;` 之后、`:root` 之前）插入：

```css
@font-face {
  font-family: 'LXGW WenKai';
  src: url('/fonts/lxgwwenkai-regular.woff2') format('woff2');
  font-weight: 400 500;
  font-style: normal;
  font-display: swap;
}
```

`frontstage.html` `<head>` 删掉这四行（含注释 `Google Fonts`）：

```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://cdn.jsdelivr.net/npm/lxgw-wenkai-webfont@1.7.0/style.css" rel="stylesheet">
<link href="https://fonts.googleapis.com/css2?family=Noto+Serif+SC:wght@400;500;600&display=swap" rel="stylesheet">
```

`--font-serif` 第一位保持 `'LXGW WenKai'`。Noto Serif SC 仅作系统/回退，不再从 Google 拉。

**不要**改 `index.html` 的 Cinzel CDN（幕后、本波不打包 Cinzel）。

- [ ] **Step 5: 跑测试确认绿**

Run: `cd src-frontend && npx vitest run src/frontstage/styles/__tests__/wenkaiFont.test.ts`

Expected: PASS。

- [ ] **Step 6: 用户可感知**

在未安装霞鹜的机器（或临时改系统字体）打开幕前正文：应为楷体纸感，Network 里幕前文档**没有** jsdelivr / fonts.googleapis 请求。

- [ ] **Step 7: 回归门 + Commit**（授权后）

```bash
cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check
git add src-frontend/public/fonts src-frontend/src/frontstage/styles/frontstage.css \
  src-frontend/frontstage.html src-frontend/src/frontstage/styles/__tests__/wenkaiFont.test.ts
git commit -m "$(cat <<'EOF'
feat: 幕前本地加载霞鹜文楷，去掉运行时字体 CDN（视觉进化 P1）

EOF
)"
```

---

### Task 3: P1 数字 `tabular-nums`

**Files:**
- Modify: `src-frontend/src/frontstage/components/FrontstageHeader.tsx`（字数 `status-item`）
- Modify: `src-frontend/src/frontstage/components/ChapterOutline.tsx`（`{item.wordCount} 字`）
- Modify: `src-frontend/src/frontstage/components/FrontstageBottomBar.tsx`（模型 tooltip 里的 TTFB / t/s 若尚未 `tabular-nums`）
- Test: `src-frontend/src/frontstage/components/__tests__/FrontstageHeader.test.tsx`

- [ ] **Step 1: 失败测试**

在 `FrontstageHeader.test.tsx` 的「应该显示章节标题和字数统计」后追加：

```tsx
it('字数使用 tabular-nums', () => {
  render(<FrontstageHeader {...defaultProps} />);
  const el = screen.getByTitle('当前章节字数 / 全文字数');
  expect(el.className).toMatch(/tabular-nums/);
});
```

- [ ] **Step 2: 跑红**

Run: `cd src-frontend && npx vitest run src/frontstage/components/__tests__/FrontstageHeader.test.tsx`

Expected: FAIL（无 `tabular-nums`）。

- [ ] **Step 3: 最小实现**

`FrontstageHeader.tsx` 字数 span 改为：

```tsx
<span className="status-item tabular-nums" title="当前章节字数 / 全文字数">
  {wordCount} 字 / {totalWordCount} 字
</span>
```

`ChapterOutline.tsx` 字数行加上 `tabular-nums`。

BottomBar 里已有耗时 `tabular-nums` 的保持；TTFB 的 `model-tooltip-meta` 若是纯文本，给该 span 加 `tabular-nums`。

- [ ] **Step 4: 跑绿 + 回归 + Commit**（授权后）

```bash
cd src-frontend && npx vitest run src/frontstage/components/__tests__/FrontstageHeader.test.tsx
cd src-frontend && npx tsc --noEmit && npm run format:check
git add src-frontend/src/frontstage/components/FrontstageHeader.tsx \
  src-frontend/src/frontstage/components/__tests__/FrontstageHeader.test.tsx \
  src-frontend/src/frontstage/components/ChapterOutline.tsx \
  src-frontend/src/frontstage/components/FrontstageBottomBar.tsx
git commit -m "$(cat <<'EOF'
style: 幕前字数与度量使用 tabular-nums（视觉进化 P1）

EOF
)"
```

---

### Task 4: P2 幕后色板、阴影、velvet、状态色

**Files:**
- Modify: `src-frontend/src/styles/tokens.css`
- Modify: `src-frontend/src/styles/backstageThemes.ts`
- Modify: `src-frontend/src/styles/__tests__/backstageThemes.test.ts`
- Modify: `src-frontend/index.html` 仅加载屏背景 hex（与新 `--cinema-950` warm 对齐，避免启动闪 OLED 黑）

**钉死新值（warm = tokens.css 默认 = `TOKENS_CSS_CURRENT`）：**

| 变量 | 旧 | 新 |
|---|---|---|
| `--cinema-950` | `#050508` | `#0c0b09` |
| `--cinema-900` | `#0a0a0f` | `#12110e` |
| `--cinema-velvet` | `#7c3aed` | `#5c5470` |
| `--status-success` | `#22c55e` | `#4a9a6a` |
| `--status-success-dim` | `rgba(34, 197, 94, 0.4)` | `rgba(74, 154, 106, 0.4)` |
| `--status-warning` | `#facc15` | `#c4a035` |
| `--status-danger` | `#ef4444` | `#c45c4a` |
| `--status-danger-dim` | `rgba(239, 68, 68, 0.4)` | `rgba(196, 92, 74, 0.4)` |
| `--shadow-panel` | `0 4px 24px rgba(0,0,0,0.4)` | `0 4px 24px color-mix(in oklch, var(--cinema-950) 55%, transparent)` |
| `--shadow-float` | `0 8px 32px rgba(0,0,0,0.5)` | `0 8px 32px color-mix(in oklch, var(--cinema-950) 65%, transparent)` |

`--cinema-850/800/700/600/500/gold*` 暖金套保持，只抬 950/900 离开纯黑。

四套主题 950 一律离开 OLED：

```ts
const STATUS = {
  '--status-success': '#4a9a6a',
  '--status-success-dim': 'rgba(74, 154, 106, 0.4)',
  '--status-warning': '#c4a035',
  '--status-danger': '#c45c4a',
  '--status-danger-dim': 'rgba(196, 92, 74, 0.4)',
};
```

```ts
warm: '--cinema-950': '#0c0b09', '--cinema-900': '#12110e', '--cinema-velvet': '#5c5470'
cool: '--cinema-950': '#0a1016', /* 900 可略抬 */ '--cinema-velvet': '#4a6678'
amber: '--cinema-950': '#120e0a', '--cinema-velvet': '#6a5340'
indigo: '--cinema-950': '#0c0c14', '--cinema-velvet': '#5a5478'
```

cool/amber/indigo 的 900–500 与 gold 保持现文件值，只改表中列出的 950/velvet（cool 的 950 从 `#04080c` 改为 `#0a1016`）。

- [ ] **Step 1: 改测试期望（先红或先一起改）**

`backstageThemes.test.ts` 的 `TOKENS_CSS_CURRENT` 换成上表新值。注释改为「warm 必须与 tokens.css 完全一致」。

追加：

```ts
it('cinema-950 不是 OLED 纯黑', () => {
  for (const theme of Object.values(backstageThemes)) {
    expect(theme.vars['--cinema-950'].toLowerCase()).not.toBe('#050508');
    expect(theme.vars['--cinema-950'].toLowerCase()).not.toBe('#000000');
  }
});

it('warm velvet 饱和度低于旧 AI 紫', () => {
  expect(backstageThemes.warm.vars['--cinema-velvet']).toBe('#5c5470');
});
```

- [ ] **Step 2: 改 tokens.css 与 backstageThemes.ts 与 index.html 加载屏**

`tokens.css` `:root` 块按表替换 cinema-950/900/velvet、STATUS、两处 shadow。

`index.html`：`background:#050508` → `background:#0c0b09`（仅加载屏内联，React 挂载前）。

`--ai-green/red/orange` 已是 `var(--status-*)`，会跟着变，不要另写一套。

- [ ] **Step 3: 跑测试**

Run: `cd src-frontend && npx vitest run src/styles/__tests__/backstageThemes.test.ts src/styles/__tests__/aiTokens.test.ts`

Expected: PASS。若「warm 与 tokens.css 一致」FAIL，用 `rg '--cinema-950'` 对齐三处。

- [ ] **Step 4: 回归 + Commit**（授权后）

```bash
cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check
git add src-frontend/src/styles/tokens.css \
  src-frontend/src/styles/backstageThemes.ts \
  src-frontend/src/styles/__tests__/backstageThemes.test.ts \
  src-frontend/index.html
git commit -m "$(cat <<'EOF'
style: 幕后木炭底、同色相阴影与降饱和状态色（视觉进化 P2）

EOF
)"
```

---

### Task 5: P2 幕前墨色抬升与暖浮层影

**Files:**
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`（`--ink`、`--shadow-float`）
- Modify: `src-frontend/src/frontstage/config/colorThemes.ts`（`ink: fmt(Math.max(ch.l - 6, 18), ...)`，原 `- 13` / floor 12）
- Modify: `src-frontend/src/frontstage/config/writingStyles.ts`（默认 `inkColor: 'oklch(32% 0.015 85)'`）

暖主题 charcoal `oklch(38% …)`，`38 - 6 = 32`，对齐设计 §8.1。

- [ ] **Step 1: 失败测试**

新建 `src-frontend/src/frontstage/config/__tests__/inkLift.test.ts`：

```ts
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { colorThemes } from '../colorThemes';

const frontendRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../../..');

describe('幕前墨色抬升', () => {
  it('frontstage.css 默认 --ink 为 oklch 32%', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    expect(css).toMatch(/--ink:\s*oklch\(32%/);
  });

  it('deriveTheme ink 亮于旧 25% 印刷黑', () => {
    expect(colorThemes.warm.ink).toMatch(/oklch\(32%/);
  });

  it('幕前 --shadow-float 不用纯黑 rgba(0,0,0', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    const block = css.match(/--shadow-float:\s*([^;]+);/)?.[1] ?? '';
    expect(block).not.toMatch(/rgba\(0,\s*0,\s*0/);
  });
});
```

（`colorThemes.warm.ink` 实际字符串以 `fmt` 输出为准；若 fmt 产生 `oklch(32.00% …)` 则断言改成 `/oklch\(\s*32/`。）

- [ ] **Step 2: 跑红**

Run: `cd src-frontend && npx vitest run src/frontstage/config/__tests__/inkLift.test.ts`

Expected: FAIL。

- [ ] **Step 3: 实现**

`frontstage.css`：

```css
--ink: oklch(32% 0.015 85);
--shadow-float: 0 8px 24px oklch(32% 0.02 85 / 0.14);
```

删掉旧 `--shadow-float: 0 8px 24px rgba(0, 0, 0, 0.12);`。

`colorThemes.ts`：

```ts
ink: fmt(Math.max(ch.l - 6, 18), ch.c * 1.1, ch.h),
```

`writingStyles.ts` 默认 `inkColor` 改为 `'oklch(32% 0.015 85)'`。

若 `frontstage.css` 另有 `box-shadow: 0 8px 24px oklch(25% 0.015 85 / 0.08)`（约 L858），改为 `oklch(32% 0.015 85 / 0.08)`。

- [ ] **Step 4: 跑绿 + 回归 + Commit**（授权后）

```bash
cd src-frontend && npx vitest run src/frontstage/config/__tests__/inkLift.test.ts
cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check
git add src-frontend/src/frontstage/styles/frontstage.css \
  src-frontend/src/frontstage/config/colorThemes.ts \
  src-frontend/src/frontstage/config/writingStyles.ts \
  src-frontend/src/frontstage/config/__tests__/inkLift.test.ts
git commit -m "$(cat <<'EOF'
style: 幕前墨色抬升与暖色浮层影（视觉进化 P2）

EOF
)"
```

---

### Task 6: P3 `--transition-press` 与 Button 契约

**Files:**
- Modify: `src-frontend/src/styles/tokens.css`（`:root` 与 `prefers-reduced-motion`）
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`（同样两处，幕前窗口不读 tokens.css）
- Modify: `src-frontend/tailwind.config.js`（`transitionTimingFunction.press`）
- Modify: `src-frontend/src/components/ui/Button.tsx`
- Test: `src-frontend/src/styles/__tests__/aiTokens.test.ts` 或新建 `pressMotion.test.ts`
- Test: Button 若已有测试则改断言；否则新建 `src-frontend/src/components/ui/__tests__/Button.test.tsx`

Press 曲线（设计 §6）：`300ms cubic-bezier(0.32, 0.72, 0, 1)`，位移 `scale-[0.98]`。禁止 `scale-95`。

- [ ] **Step 1: 失败测试**

`src-frontend/src/styles/__tests__/pressMotion.test.ts`：

```ts
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const frontendRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');

describe('press 运动令牌', () => {
  it('tokens.css 与 frontstage.css 都定义 --transition-press 并在 reduced-motion 冻结', () => {
    for (const rel of ['src/styles/tokens.css', 'src/frontstage/styles/frontstage.css']) {
      const css = readFileSync(resolve(frontendRoot, rel), 'utf-8');
      expect(css, rel).toMatch(/--transition-press:\s*0\.3s cubic-bezier\(0\.32,\s*0\.72,\s*0,\s*1\)/);
      expect(css, rel).toMatch(/prefers-reduced-motion[\s\S]*--transition-press:\s*0\.01s linear/);
    }
  });

  it('tailwind 注册 ease-press', () => {
    const tw = readFileSync(resolve(frontendRoot, 'tailwind.config.js'), 'utf-8');
    expect(tw).toMatch(/press:\s*'cubic-bezier\(0\.32,\s*0\.72,\s*0,\s*1\)'/);
  });
});
```

Button 测试：

```tsx
import { render, screen } from '@testing-library/react';
import { Button } from '../Button';

it('press 用 scale-98 而非 scale-95', () => {
  render(<Button>确定</Button>);
  const cls = screen.getByRole('button').className;
  expect(cls).toMatch(/active:scale-\[0\.98\]/);
  expect(cls).not.toMatch(/active:scale-95/);
  expect(cls).toMatch(/ease-press/);
});
```

- [ ] **Step 2: 跑红**

Run: `cd src-frontend && npx vitest run src/styles/__tests__/pressMotion.test.ts src/components/ui/__tests__/Button.test.tsx`

Expected: FAIL。

- [ ] **Step 3: 实现**

`tokens.css` `:root` 在 `--transition-spring` 旁加：

```css
--transition-press: 0.3s cubic-bezier(0.32, 0.72, 0, 1);
```

`prefers-reduced-motion` 块加 `--transition-press: 0.01s linear;`。

`frontstage.css` `:root` 现有 `--transition-fast/normal/slow` 旁同样加 `--transition-press`；若无 reduced-motion 块，仿 `tokens.css` 加一个，至少冻结 `--transition-press` 与 `--transition-spring`。

`tailwind.config.js` `transitionTimingFunction`：

```js
spring: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
press: 'cubic-bezier(0.32, 0.72, 0, 1)',
```

`Button.tsx` `base` 字符串改为：

```ts
const base =
  'inline-flex items-center justify-center font-medium transition-[background-color,color,border-color,transform,opacity] duration-300 ease-press enabled:active:scale-[0.98] focus-visible:outline-none focus-visible:ring-2 disabled:opacity-50 disabled:cursor-not-allowed';
```

把 P0 发射键上的任意 `ease-[cubic-bezier(0.32,0.72,0,1)]` 换成 `ease-press`（`AiPromptBar.tsx` + `FrontstageBottomBar.tsx` 取消键），避免两套写法。

- [ ] **Step 4: 跑绿 + 回归 + Commit**（授权后）

```bash
cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check
git add src-frontend/src/styles/tokens.css src-frontend/src/frontstage/styles/frontstage.css \
  src-frontend/tailwind.config.js src-frontend/src/components/ui/Button.tsx \
  src-frontend/src/styles/__tests__/pressMotion.test.ts \
  src-frontend/src/components/ui/__tests__/Button.test.tsx \
  src-frontend/src/components/ui/ai/AiPromptBar.tsx \
  src-frontend/src/frontstage/components/FrontstageBottomBar.tsx
git commit -m "$(cat <<'EOF'
style: 接入 press 曲线并修正 Button 按压位移（视觉进化 P3）

EOF
)"
```

---

### Task 7: P3 主路径去掉装饰 pulse / ping

**Files:**
- Modify: `src-frontend/src/frontstage/components/FrontstageBottomBar.tsx`
- Modify: `src-frontend/src/components/NovelCreationWizard.tsx`（`animate-ping` 装饰环）
- Modify: `src-frontend/src/pages/Stories.tsx`（`isHighlighted` 的 `animate-pulse` 是空闲高亮，删 pulse、保留 ring）
- 不要改：`IntentionGraphDiagnostics.tsx` 骨架；`KnowledgeGraph` 在 **pending 动作期间** 的 pulse（那是「正在做」不是空闲）

BottomBar 现状锚点：

- `statusClass`：`degraded` 与 `default` 带 `animate-pulse` → 改为静色 `bg-status-warning`（探测中允许，但不要 pulse）。`unhealthy` 保持静色危险，去掉 glow 若过于跳。
- 空模型占位竖条 `animate-pulse` → 静色 `bg-ink-500`。
- 本地生成 `Activity` 的 `animate-pulse` + 外圈 `animate-ping` → 只留陶土 `Activity`，无 ping。

- [ ] **Step 1: 失败测试**

在 `FrontstageBottomBar.test.tsx` 追加：

```tsx
it('模型信号与本地生成指示不含 pulse/ping', () => {
  render(<FrontstageBottomBar {...defaultProps} isGenerating={true} />);
  expect(document.querySelector('.model-signal-bar')?.className ?? '').not.toMatch(
    /animate-pulse/
  );
  expect(document.querySelector('.animate-ping')).toBeNull();
});
```

- [ ] **Step 2: 跑红**

Run: `cd src-frontend && npx vitest run src/frontstage/components/__tests__/FrontstageBottomBar.test.tsx`

Expected: FAIL（空网关仍 pulse，生成仍 ping）。

- [ ] **Step 3: 实现**

`statusClass`：

```ts
const statusClass = (status: ModelHealthSnapshot['status']) => {
  switch (status) {
    case 'healthy':
      return 'bg-status-success';
    case 'degraded':
      return 'bg-status-warning';
    case 'unhealthy':
      return 'bg-status-danger';
    default:
      return 'bg-status-warning';
  }
};
```

空列表占位竖条去掉 `animate-pulse`。

本地生成图标块改为：

```tsx
<Activity className="w-4 h-4 text-terracotta" />
```

删除绝对定位的 `animate-ping` span。

`NovelCreationWizard.tsx` 删除 `animate-ping` 的装饰环 div，保留按钮本身。

`Stories.tsx` 把 `isHighlighted ? 'ring-2 ring-cinema-gold/70 animate-pulse'` 改为 `'ring-1 ring-cinema-gold/40'`（无 pulse）。

- [ ] **Step 4: 跑绿 + 回归 + Commit**（授权后）

```bash
cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check
git add src-frontend/src/frontstage/components/FrontstageBottomBar.tsx \
  src-frontend/src/frontstage/components/__tests__/FrontstageBottomBar.test.tsx \
  src-frontend/src/components/NovelCreationWizard.tsx
git commit -m "$(cat <<'EOF'
style: 去掉幕前信号条与生成指示的装饰脉冲（视觉进化 P3）

EOF
)"
```

---

### Task 8: P4 幕后面板 bezel + 侧栏徽章

**Files:**
- Modify: `src-frontend/src/components/ui/Panel.tsx`
- Modify: `src-frontend/src/components/Sidebar.tsx`（`impactBadgeClass`）
- Test: `src-frontend/src/components/__tests__/Sidebar.ia.test.tsx`
- Test: 若无 Panel 测试则新建 `src-frontend/src/components/ui/__tests__/Panel.test.tsx`

Bezel（设计 §8.3）：外壳 `rounded-panel p-1` + 发丝边；内芯 `rounded-[calc(var(--radius-md)-4px)]` + `bg-cinema-850`。幕前禁止 bezel。

徽章：四色胶囊 → 低对比文字。testid 与文案「热/温/冷/配」不变。

- [ ] **Step 1: 失败测试**

`Sidebar.ia.test.tsx` 追加：

```tsx
it('impact badge 不再使用高饱和色胶囊', () => {
  render(<Sidebar currentView="stories" onNavigate={onNavigate} />);
  const badge = screen.getByTestId('impact-badge-stories');
  expect(badge.className).not.toMatch(/emerald|amber-500|sky-500/);
  expect(badge.className).toMatch(/text-cinema-/);
});
```

`Panel.test.tsx`：

```tsx
import { render, screen } from '@testing-library/react';
import { Panel } from '../Panel';

it('外壳 bezel + 内芯半径级差', () => {
  const { container } = render(<Panel title="设定">内容</Panel>);
  const shell = container.firstChild as HTMLElement;
  expect(shell.className).toMatch(/p-1/);
  expect(shell.className).toMatch(/rounded-panel/);
  expect(container.querySelector('.bg-cinema-850')).not.toBeNull();
});
```

- [ ] **Step 2: 跑红**

Run: `cd src-frontend && npx vitest run src/components/__tests__/Sidebar.ia.test.tsx src/components/ui/__tests__/Panel.test.tsx`

Expected: FAIL。

- [ ] **Step 3: 实现**

`impactBadgeClass`：

```ts
function impactBadgeClass(_impact: NavImpact): string {
  return 'text-cinema-500 border-transparent bg-transparent px-0';
}
```

保留 `impactShort`。「热」仍出现在 `impact-badge-stories`。侧栏分组 `NAV_GROUPS` 的 `space-y` / item `py-2.5` 改为分组之间 `mt-4`（在组 wrapper 上加 `mb-3`），不要改信息架构。

`Panel.tsx` 根节点改为：

```tsx
<div className="rounded-panel border border-borderSubtle bg-cinema-900/40 p-1 shadow-panel">
  <div className="overflow-hidden rounded-[calc(var(--radius-md)-4px)] bg-cinema-850">
    {/* 原 header + content，去掉根上的 bg-cinema-850 border rounded-panel shadow-panel */}
  </div>
</div>
```

折叠动画继续 `ease-spring`（设计：展开用 spring）。

- [ ] **Step 4: 跑绿 + 回归 + Commit**（授权后）

```bash
cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check
git add src-frontend/src/components/ui/Panel.tsx \
  src-frontend/src/components/ui/__tests__/Panel.test.tsx \
  src-frontend/src/components/Sidebar.tsx \
  src-frontend/src/components/__tests__/Sidebar.ia.test.tsx
git commit -m "$(cat <<'EOF'
style: 幕后面板双包边与侧栏徽章降噪（视觉进化 P4）

EOF
)"
```

---

### Task 9: P4 剩余实心 CTA 抄 P0

**Files:**
- Modify: `src-frontend/src/components/ui/Button.tsx`（`paper` / `cinema` / `primary`）
- Modify: `src-frontend/src/components/ui/ai/AiSelectionActions.tsx`（`primary` 常量）
- Modify: `src-frontend/src/pages/settings/GeneralSettings.tsx`（`focus:ring-2` → `ring-1`）
- Modify: `src-frontend/src/components/ui/StudioNavRail.tsx` 仅当仍有 `from-cinema-gold` 实心方标且刺眼时改为金淡彩；**不要**重做 logo。
- Test: 扩展 `Button.test.tsx`；`AiSelectionActions.test.tsx` 断言主按钮 class 不含 `bg-ai-ink`

设计：幕前禁止陶土实心大钮；幕后禁止高饱和金填充主按钮；划词主按钮禁止炭黑实心。

- [ ] **Step 1: 失败测试**

`Button.test.tsx` 追加：

```tsx
it('paper 与 cinema 主按钮是淡彩不是实心高饱和填充', () => {
  const { rerender } = render(<Button variant="paper">写</Button>);
  expect(screen.getByRole('button').className).not.toMatch(/bg-terracotta[^-]/);
  rerender(<Button variant="cinema">做</Button>);
  expect(screen.getByRole('button').className).not.toMatch(/bg-cinema-gold[^-]/);
});
```

`AiSelectionActions.test.tsx` 在 `phase="result"` 渲染后：

```tsx
const keep = screen.getByRole('button', { name: /保留/ });
expect(keep.className).not.toMatch(/bg-ai-ink/);
expect(keep.className).toMatch(/ai-accent/);
```

- [ ] **Step 2: 跑红**

Run: `cd src-frontend && npx vitest run src/components/ui/__tests__/Button.test.tsx src/components/ui/ai/__tests__/AiSelectionActions.test.tsx`

Expected: FAIL。

- [ ] **Step 3: 实现**

`Button.tsx` `variantMap`：

```ts
paper:
  'bg-terracotta/15 text-terracotta-dark hover:bg-terracotta/25 focus-visible:ring-terracotta/40',
cinema:
  'bg-cinema-gold/15 text-cinema-gold hover:bg-cinema-gold/25 focus-visible:ring-cinema-gold/40',
primary:
  'bg-cinema-gold/15 text-cinema-gold hover:bg-cinema-gold/25 focus-visible:ring-cinema-gold/40',
```

`cinema-outline` / `secondary` / `ghost` / `danger` 不动。

`AiSelectionActions.tsx` 的 `primary` 常量改为：

```ts
const primary =
  'inline-flex h-7 shrink-0 items-center gap-1 rounded-md bg-[color-mix(in_oklch,var(--ai-accent)_18%,transparent)] px-2.5 text-[12.5px] font-normal text-ai-accent-ink transition-[opacity,transform] duration-300 ease-press hover:opacity-90 active:scale-[0.98]';
```

`control` 的 `active:scale-[0.96]` 改为 `active:scale-[0.98] duration-300 ease-press`；`rounded-full` 可留（浮条胶囊），不要改成纸面 2px 以免浮层过尖。

设置页刺眼金 glow（设计 §11）：`GeneralSettings.tsx` 四处 `focus:ring-2 focus:ring-cinema-gold/50` 改为 `focus:ring-1 focus:ring-cinema-gold/30`。其它仅 `focus:border-cinema-gold`、无 ring 的输入保持。

- [ ] **Step 4: 跑绿 + 回归 + Commit**（授权后）

```bash
cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check
git add src-frontend/src/components/ui/Button.tsx \
  src-frontend/src/components/ui/__tests__/Button.test.tsx \
  src-frontend/src/components/ui/ai/AiSelectionActions.tsx \
  src-frontend/src/components/ui/ai/__tests__/AiSelectionActions.test.tsx
git commit -m "$(cat <<'EOF'
style: 主按钮与划词采纳改为强调淡彩（视觉进化 P4）

EOF
)"
```

---

### Task 10: P5 空态统一（最小集）

**Files:**
- Create: `src-frontend/src/components/ui/EmptyHint.tsx`
- Create: `src-frontend/src/components/ui/__tests__/EmptyHint.test.tsx`
- Modify: `src-frontend/src/pages/AgencyStudio.tsx`（`暂无活动` 段）
- Modify: `src-frontend/src/pages/Tasks.tsx`（`暂无任务` 段）
- Modify: `src-frontend/src/components/index.ts` 仅当该 barrel 已导出 Panel/Button 时同样导出 `EmptyHint`

YAGNI：不扫全仓「暂无」。只换设计点名的工作室主空态。加载继续用已有 `AiLoading`，本 Task 不替换骨架屏。

- [ ] **Step 1: 失败测试**

```tsx
import { render, screen } from '@testing-library/react';
import { EmptyHint } from '../EmptyHint';

it('渲染说明文字且无 pulse', () => {
  const { container } = render(<EmptyHint>暂无活动</EmptyHint>);
  expect(screen.getByText('暂无活动')).toBeInTheDocument();
  expect(container.firstChild).toHaveClass('text-ai-ink-3');
  expect((container.firstChild as HTMLElement).className).not.toMatch(/animate-pulse/);
});
```

- [ ] **Step 2: 跑红**

Run: `cd src-frontend && npx vitest run src/components/ui/__tests__/EmptyHint.test.tsx`

Expected: FAIL（模块不存在）。

- [ ] **Step 3: 实现**

`EmptyHint.tsx`：

```tsx
import type { ReactNode } from 'react';

export function EmptyHint({ children }: { children: ReactNode }) {
  return (
    <p className="rounded-md border border-dashed border-ai-line px-4 py-6 text-center text-sm text-ai-ink-3">
      {children}
    </p>
  );
}
```

`AgencyStudio.tsx` 将

```tsx
<p className="rounded border border-dashed p-4 text-sm text-ai-ink-3">
  暂无活动--启动创世或续写后，这里会实时显示代理动态。
</p>
```

换成 `<EmptyHint>暂无活动——启动创世或续写后，这里会实时显示代理动态。</EmptyHint>`（破折号用中文破折号，文案可保持原句只换外壳）。

`Tasks.tsx` `暂无任务` 的 `<p className="text-sm">` 换成 `<EmptyHint>暂无任务</EmptyHint>`。

- [ ] **Step 4: 跑绿 + 回归 + Commit**（授权后）

```bash
cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check
python3 scripts/architecture_guard.py
git add src-frontend/src/components/ui/EmptyHint.tsx \
  src-frontend/src/components/ui/__tests__/EmptyHint.test.tsx \
  src-frontend/src/pages/AgencyStudio.tsx \
  src-frontend/src/pages/Tasks.tsx
git commit -m "$(cat <<'EOF'
style: 工作室空态统一为 EmptyHint（视觉进化 P5）

EOF
)"
```

---

### Task 11: 设计文档状态回写（不发版）

**Files:**
- Modify: `docs/plans/2026-08-13-ink-paper-visual-evolution-design.md` §12 阶段表：P0–P5 标已实施（以实际完成的 Task 为准）
- **不要**改 `CHANGELOG.md` / `AGENTS.md` / 版本号，除非用户要求发版。

- [ ] **Step 1:** 把设计 §12 各行状态从「未做 / 工作区已实现」改成「已实施（见 implementation plan Task N）」
- [ ] **Step 2:** Commit（授权后）`docs: 回写墨纸机械视觉进化阶段状态`

---

## 执行顺序与停点

```
Task 1 P0 → Task 2–3 P1 → Task 4–5 P2 → Task 6–7 P3 → Task 8–9 P4 → Task 10 P5 → Task 11
```

每阶段结束后对照设计 §13.2。P1 未看到本地楷体不得开 P2。P0 是后续 CTA 的样板，回归测试必须一直绿。

## 发版（本计划外）

用户要求发版时另做：四源 bump、docs of record、tag、推送、`gh run list` 盯到全绿。视觉进化可与功能版合并，不必单独占一个大版本号——由用户定。
