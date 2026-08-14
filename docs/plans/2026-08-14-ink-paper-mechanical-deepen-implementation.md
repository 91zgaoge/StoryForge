# 墨纸 / 机械定向进化补齐 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 按 `docs/plans/2026-08-14-ink-paper-mechanical-deepen-design.md` 把 v0.43.0 未做满的缺口补齐：P0 幕前输入无框，P1 Medium 字体 / 选区 / 纸色 / 顶栏淡彩，P2 幕后同色相内芯 / Panel 高光 / 弹簧 500ms / 侧栏去金框。

**Architecture:** 纯前端。不改 IPC / Agency / PersistMode / 路由。令牌仍是双窗各自定义的 `--ai-*` 17 名（只改值）。幕前纸走 `--parchment*` OKLCH；幕后色走 `backstageThemes.ts` + `tokens.css` 三方同 commit。字体第二档 Medium 本地 woff2，禁止 CDN。

**Tech Stack:** React 18 + Tailwind v3.4、vitest 4 + Testing Library、lucide-react、霞鹜文楷 SIL OFL、fontTools 压 woff2。零新 npm 运行时依赖。

**需求来源：** `docs/plans/2026-08-14-ink-paper-mechanical-deepen-design.md`。前序已发版：`2026-08-13-ink-paper-visual-evolution-design.md`（v0.43.0）。

---

## Global Constraints

- 仓库 `/Users/yuzaimu/projects/StoryForge`。在 **master** 上做。不要用 `.worktrees/ink-paper`（那是 v0.43.0 残留）。
- 中文 conventional commit。不 `--no-verify`。**不推送、不打 tag、不 bump 版本**，发版等用户指令。
- **Commit 步骤**：仅当用户在本会话明确说「提交」时执行各 Task 末的 commit。未授权则做完代码+测试停在工作区。
- 改 `Panel` / `Sidebar` / `FrontstageBottomBar` 前必须跑 GitNexus `impact({target, direction:"upstream"})` 并在该 Task 记录风险。HIGH/CRITICAL 先告诉用户再改。只改 className / CSS 变量值，不改组件 API。
- 禁止：岛式顶栏、GSAP、新图标包、运行时字体 CDN、复活 Ghost Chrome、炭黑发射键、输入壳加回 `border`/`rounded-paper`/`bg-paper-50`、焦点 ring、幕前 bezel、卡片悬停 `scale` 过冲、空闲 pulse/ping。
- `--ai-*` 17 变量名不得增删。
- `backstageThemes.test.ts` 的 `TOKENS_CSS_CURRENT` 必须与 `tokens.css` + `backstageThemes.warm.vars` **三方同 commit**。
- 纯前端。误碰 Rust 则必须重跑 `cargo test --lib`。
- 每刀对照设计 §10.2 用户可感知项；只改 className 未看三态不得勾完成。
- 准入线（每 Task 回归）：`cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check`；仓库根 `python3 scripts/architecture_guard.py`。vitest 只允许增加。
- 行号会漂，执行以**锚点代码**定位。
- P0 / P1 / P2 可独立成为一次发版。未对照该刀验收不得开下一刀。

---

## File map

| 文件 | 职责 |
|---|---|
| `src-frontend/src/frontstage/components/FrontstageBottomBar.tsx` | P0 无框；底栏 `--parchment-dark` |
| `src-frontend/src/frontstage/styles/frontstage.css` | P0 幽灵偏移核；P1 `@font-face` Medium、选区、纸 chroma、顶栏按钮 |
| `src-frontend/src/frontstage/components/__tests__/FrontstageBottomBar.test.tsx` | P0 无框探针 |
| `src-frontend/src/frontstage/styles/__tests__/wenkaiFont.test.ts` | P1 Regular≠400 500；Medium 文件 |
| `src-frontend/public/fonts/lxgwwenkai-medium.woff2` | P1 新建 |
| `src-frontend/public/fonts/README.md` | P1 去掉「400 500 映射同一文件」 |
| `src-frontend/src/frontstage/config/colorThemes.ts` | P1 warm 纸起点 |
| `src-frontend/src/frontstage/config/writingStyles.ts` | P1 默认 `paperColor` |
| `src-frontend/src/frontstage/config/__tests__/inkLift.test.ts` | P1 纸 chroma 探针（同文件追加） |
| `src-frontend/src/styles/tokens.css` | P1 paper hex；P2 spring 0.5s + cinema 850–500 |
| `src-frontend/src/frontstage/components/FrontstageHeader.tsx` | P1 不改结构；样式在 CSS |
| `src-frontend/src/frontstage/components/__tests__/FrontstageHeader.test.tsx` | P1 按钮 class 探针 |
| `src-frontend/src/styles/backstageThemes.ts` | P2 warm 850–500 |
| `src-frontend/src/styles/__tests__/backstageThemes.test.ts` | P2 `TOKENS_CSS_CURRENT` |
| `src-frontend/src/styles/__tests__/pressMotion.test.ts` | P2 spring 0.5s |
| `src-frontend/src/components/ui/Panel.tsx` | P2 inset 高光 + duration-500 |
| `src-frontend/src/components/ui/__tests__/Panel.test.tsx` | P2 高光 / 时长 |
| `src-frontend/src/components/Sidebar.tsx` | P2 选中去金框 |
| `src-frontend/src/components/__tests__/Sidebar.ia.test.tsx` | P2 选中无 `border-cinema-gold` |
| `src-frontend/src/components/SceneEditor.tsx` | P2 页签 spring |
| `src-frontend/src/pages/Stories.tsx` | P2 风格页签 spring |
| `src-frontend/src/pages/settings/ModelCard.tsx` | P2 switch 用 spring（颜色过渡） |
| `src-frontend/src/pages/settings/__tests__/PromptsPanel.test.tsx` | P2 空闲无 pulse 回归 |
| `src-frontend/src/components/ui/ai/AiPromptBar.tsx` | **不改**（保持 flush） |

不要改：`landing/`、Rust、IPC、`--ai-*` 变量名、`AiPromptBar` 默认 `card`。

---

### Task 1: P0 幕前输入无框

**Files:**
- Modify: `src-frontend/src/frontstage/components/__tests__/FrontstageBottomBar.test.tsx`
- Modify: `src-frontend/src/frontstage/components/FrontstageBottomBar.tsx`（锚点：`bg-paper-100/90 backdrop-blur-sm border-t border-paper-300` 与内层 `bg-paper-50 border border-paper-300 rounded-paper`）
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`（仅当幽灵错位；锚点 `.frontstage-input-ghost { top: 5px; left: 4px; }`）

**GitNexus:** `impact({target: "FrontstageBottomBar", direction: "upstream"})`。预期调用方是幕前壳。只改 className，不改 props。若 HIGH，先报告用户再继续。

- [ ] **Step 1: 写失败测试**

在 `FrontstageBottomBar.test.tsx` 现有 `'输入条走 flush 纸面，取消生成无脉冲红块'` **之后**追加：

```tsx
  it('输入区无框：外壳无边框圆角独立底，底栏无顶边与毛玻璃', () => {
    const { container } = render(<FrontstageBottomBar {...defaultProps} inputValue="续写" />);
    const bar = container.firstChild as HTMLElement;
    expect(bar.className).not.toMatch(/border-t/);
    expect(bar.className).not.toMatch(/backdrop-blur/);
    expect(bar.className).not.toMatch(/bg-paper-100/);
    expect(bar.className).toMatch(/parchment-dark/);

    const shell = screen.getByTestId('ai-prompt-bar').parentElement as HTMLElement;
    expect(shell.className).not.toMatch(/\bborder\b/);
    expect(shell.className).not.toMatch(/rounded-paper/);
    expect(shell.className).not.toMatch(/bg-paper-50/);
    expect(shell.className).not.toMatch(/focus-within:border/);
  });
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cd src-frontend && npx vitest run src/frontstage/components/__tests__/FrontstageBottomBar.test.tsx`

Expected: FAIL。失败信息含 `border-t` 或 `bg-paper-50` 仍匹配。

- [ ] **Step 3: 最小实现**

底栏外层 class 改为：

```tsx
    <div
      className={[
        'fixed bottom-0 left-0 right-0 z-40',
        'flex flex-col items-center px-4 py-3',
        'bg-[var(--parchment-dark)]',
      ].join(' ')}
    >
```

输入壳（`AiPromptBar` 的父 div）改为只保留排版，去掉材质：

```tsx
        <div className="flex items-end gap-2 px-2.5 py-1.5">
```

不要改 `AiPromptBar` 的 `variant="flush"`、发射/取消、Enter/IME。不要给壳加回 `border` / `focus-within`。

幽灵：`textarea` 仍是 `px-1 py-[5px]`，`.frontstage-input-ghost` 保持 `top: 5px; left: 4px`。若人工对照 logline 后缀与输入基线错位，只改这两个数字，并在同测试文件加：

```tsx
  it('幽灵提示 CSS 仍声明绝对偏移', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    expect(css).toMatch(/\.frontstage-input-ghost\s*\{[^}]*top:\s*\d+px/s);
    expect(css).toMatch(/\.frontstage-input-ghost\s*\{[^}]*left:\s*\d+px/s);
  });
```

（若偏移未改，不必加此测试。）

- [ ] **Step 4: 跑测试确认通过**

Run: `cd src-frontend && npx vitest run src/frontstage/components/__tests__/FrontstageBottomBar.test.tsx && npx tsc --noEmit && npm run format:check`

Expected: PASS。对照设计 §5.3：一层纸、无描边、发射淡彩仍在、取消不叫。

- [ ] **Step 5: 回归 + 根目录守卫**

Run: `cd src-frontend && npx vitest run && python3 ../scripts/architecture_guard.py`

Expected: vitest 全绿（count ≥ 基线）；architecture_guard 零违规。

- [ ] **Step 6: Commit（仅用户说「提交」时）**

```bash
git add src-frontend/src/frontstage/components/FrontstageBottomBar.tsx \
  src-frontend/src/frontstage/components/__tests__/FrontstageBottomBar.test.tsx \
  src-frontend/src/frontstage/styles/frontstage.css
git commit -m "$(cat <<'EOF'
style: 幕前输入条去掉卡片外壳，字写在底栏纸面上

EOF
)"
```

---

### Task 2: P1 霞鹜文楷 Medium 分文件

**Files:**
- Modify: `src-frontend/src/frontstage/styles/__tests__/wenkaiFont.test.ts`
- Create: `src-frontend/public/fonts/lxgwwenkai-medium.woff2`
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`（文件顶 `@font-face`）
- Modify: `src-frontend/public/fonts/README.md`

- [ ] **Step 1: 改测试使 Regular 映射失败**

把 `wenkaiFont.test.ts` 的 `'frontstage.css 以 @font-face 声明…'` 整段换成：

```ts
  it('frontstage.css 分别声明 Regular 400 与 Medium 500', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    expect(css).not.toMatch(/font-weight:\s*400 500/);
    expect(css).toMatch(/lxgwwenkai-regular\.woff2/);
    expect(css).toMatch(/lxgwwenkai-medium\.woff2/);
    expect(css).toMatch(/font-weight:\s*400;/);
    expect(css).toMatch(/font-weight:\s*500;/);
    expect(css).toMatch(/font-display:\s*swap/);
  });

  it('public/fonts 含 medium woff2 且体积不超过 regular 的 1.15 倍', () => {
    const regular = readFileSync(resolve(frontendRoot, 'public/fonts/lxgwwenkai-regular.woff2'));
    const mediumPath = resolve(frontendRoot, 'public/fonts/lxgwwenkai-medium.woff2');
    expect(existsSync(mediumPath)).toBe(true);
    const medium = readFileSync(mediumPath);
    expect(medium.subarray(0, 4).toString('ascii')).toBe('wOF2');
    expect(medium.byteLength).toBeGreaterThan(1_000_000);
    expect(medium.byteLength).toBeLessThanOrEqual(Math.ceil(regular.byteLength * 1.15));
  });
```

保留原 `'public/fonts 含 regular woff2 与 OFL'` 与 `'frontstage.html 不再请求 jsdelivr…'`。

- [ ] **Step 2: 跑测试确认失败**

Run: `cd src-frontend && npx vitest run src/frontstage/styles/__tests__/wenkaiFont.test.ts`

Expected: FAIL（`400 500` 仍在，且 medium 文件不存在）。

- [ ] **Step 3: 从官方 TTF 压 Medium woff2**

与 Regular 同一标签 `v1.250`（见现 `public/fonts/README.md`）。在仓库根执行：

```bash
mkdir -p /tmp/lxgw-medium
curl -L --fail -o /tmp/lxgw-medium/LXGWWenKai-Medium.ttf \
  https://github.com/lxgw/LxgwWenKai/releases/download/v1.250/LXGWWenKai-Medium.ttf
python3 - <<'PY'
from fontTools.ttLib import TTFont
src = "/tmp/lxgw-medium/LXGWWenKai-Medium.ttf"
dst = "/Users/yuzaimu/projects/StoryForge/src-frontend/public/fonts/lxgwwenkai-medium.woff2"
font = TTFont(src)
font.flavor = "woff2"
font.save(dst)
print("wrote", dst)
PY
ls -l src-frontend/public/fonts/*.woff2
```

若 `curl` 404：打开 https://github.com/lxgw/LxgwWenKai/releases/tag/v1.250 核对 Medium TTF 文件名后改 URL，不要换标签。

若 medium.byteLength > regular × 1.15：用与压 Regular 时相同的 glyph 子集再压（读 Regular 的 cmap，过滤 Medium）。禁止把完整 TTF 打进 `public/fonts/`。

- [ ] **Step 4: 改 `@font-face`**

`frontstage.css` 文件顶替换为两个 face（`font-display: swap` 都要）：

```css
@font-face {
  font-family: 'LXGW WenKai';
  src: url('/fonts/lxgwwenkai-regular.woff2') format('woff2');
  font-weight: 400;
  font-style: normal;
  font-display: swap;
}

@font-face {
  font-family: 'LXGW WenKai';
  src: url('/fonts/lxgwwenkai-medium.woff2') format('woff2');
  font-weight: 500;
  font-style: normal;
  font-display: swap;
}
```

`README.md` 全文改为：

```md
# LXGW WenKai（霞鹜文楷）

- 来源：官方仓库 `lxgw/LxgwWenKai@v1.250` 的 TTF，用 fontTools 压成 woff2
- 许可：SIL Open Font License 1.1（见 OFL.txt）
- Regular：`lxgwwenkai-regular.woff2`（font-weight 400）
- Medium：`lxgwwenkai-medium.woff2`（font-weight 500）
- 禁止把 400 与 500 映射到同一文件。禁止在 HTML 里再引入 jsDelivr / Google Fonts 作为幕前正文来源。
```

- [ ] **Step 5: 跑测试确认通过**

Run: `cd src-frontend && npx vitest run src/frontstage/styles/__tests__/wenkaiFont.test.ts && npx tsc --noEmit && npm run format:check`

Expected: PASS。

- [ ] **Step 6: Commit（仅用户说「提交」时）**

```bash
git add src-frontend/public/fonts/lxgwwenkai-medium.woff2 \
  src-frontend/public/fonts/README.md \
  src-frontend/src/frontstage/styles/frontstage.css \
  src-frontend/src/frontstage/styles/__tests__/wenkaiFont.test.ts
git commit -m "$(cat <<'EOF'
style: 霞鹜文楷 Medium 单独打包，不再把 500 映射到 Regular

EOF
)"
```

---

### Task 3: P1 纸 chroma 半步 + 选区 22%

**Files:**
- Modify: `src-frontend/src/frontstage/config/__tests__/inkLift.test.ts`
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`（`--parchment`、`::selection`、`.ProseMirror ::selection`）
- Modify: `src-frontend/src/frontstage/config/colorThemes.ts`（warm 纸起点 `'oklch(96.5% 0.008 95)'`）
- Modify: `src-frontend/src/frontstage/config/writingStyles.ts`（`paperColor: 'oklch(96.5% 0.008 95)'`）
- Modify: `src-frontend/src/styles/tokens.css`（`--paper-50/100/200/300`）

只改 **暖赭默认纸**。不要改 `colorThemes.cool/amber/indigo` 的 parchment 参数。

- [ ] **Step 1: 写失败测试**

在 `inkLift.test.ts` 末尾追加：

```ts
  it('暖赭默认纸 chroma 为 0.012', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    expect(css).toMatch(/--parchment:\s*oklch\(96\.5%\s+0\.012\s+95\)/);
    expect(colorThemes.warm.parchment).toMatch(/oklch\(96\.5%\s+0\.012\s+95\)/);
  });

  it('选区陶土写进 background 的 22% mix，不用 opacity', () => {
    const css = readFileSync(
      resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
      'utf-8'
    );
    const sel = css.match(/::selection\s*\{[^}]+\}/s)?.[0] ?? '';
    const pm = css.match(/\.ProseMirror ::selection\s*\{[^}]+\}/s)?.[0] ?? '';
    expect(sel).toMatch(/color-mix\(in oklch,\s*var\(--terracotta\)\s+22%/);
    expect(pm).toMatch(/color-mix\(in oklch,\s*var\(--terracotta\)\s+22%/);
    expect(sel).not.toMatch(/opacity:/);
    expect(pm).not.toMatch(/opacity:/);
  });
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cd src-frontend && npx vitest run src/frontstage/config/__tests__/inkLift.test.ts`

Expected: FAIL（仍是 `0.008` 或 `opacity:`）。

- [ ] **Step 3: 最小实现**

`frontstage.css` `:root`：

```css
  --parchment: oklch(96.5% 0.012 95);
  --parchment-dark: oklch(93.5% 0.014 95);
  --warm-sand: oklch(91% 0.018 95);
  --border-cream: oklch(94% 0.013 95);
```

（后三行对齐 `deriveTheme`：L-3 / c×1.2、L-5.5 / c×1.5、L-2.5 / c×1.1，避免 JS 应用主题前第一帧接缝。）

`colorThemes.ts` warm 第四参：`'oklch(96.5% 0.012 95)'`。

`writingStyles.ts` 里那一处 `paperColor: 'oklch(96.5% 0.008 95)'` → `'oklch(96.5% 0.012 95)'`。

`tokens.css` 纸 hex 半步（供残留 `paper-*` 工具类，底栏已走 `--parchment-dark`）：

```css
  --paper-50: #fdf8f0;
  --paper-100: #f8f0e6;
  --paper-200: #efe4d4;
  --paper-300: #e4d5c2;
```

选区两处替换为（删掉 `opacity`）：

```css
::selection {
  background: color-mix(in oklch, var(--terracotta) 22%, transparent);
  color: var(--ink);
}

.ProseMirror ::selection {
  background: color-mix(in oklch, var(--terracotta) 22%, transparent);
  color: var(--ink);
}
```

- [ ] **Step 4: 跑测试确认通过**

Run: `cd src-frontend && npx vitest run src/frontstage/config/__tests__/inkLift.test.ts src/styles/__tests__/aiTokens.test.ts && npx tsc --noEmit`

Expected: PASS。`aiTokens` 17 变量仍全绿。

- [ ] **Step 5: Commit（仅用户说「提交」时）**

```bash
git add src-frontend/src/frontstage/styles/frontstage.css \
  src-frontend/src/frontstage/config/colorThemes.ts \
  src-frontend/src/frontstage/config/writingStyles.ts \
  src-frontend/src/frontstage/config/__tests__/inkLift.test.ts \
  src-frontend/src/styles/tokens.css
git commit -m "$(cat <<'EOF'
style: 暖赭纸 chroma 半步，选区改为陶土 22% 透明

EOF
)"
```

---

### Task 4: P1 顶栏按钮抄发射键 press / 淡彩

**Files:**
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`（锚点 `.settings-btn` / `.zen-mode-btn` / `.wensi-mode-toggle`）
- Modify: `src-frontend/src/frontstage/components/__tests__/FrontstageHeader.test.tsx`

不改 Header 的 JSX 结构、不改点击行为。不抄发射键「空闲近隐」。不要 `rounded-full` 实心圆 + `scale(1.1)`。

- [ ] **Step 1: 写失败测试**

在 `FrontstageHeader.test.tsx` 追加：

```tsx
  it('设置/禅/文思按钮走 press 曲线，无 hover scale 1.1', () => {
    const css = readFileSync(
      resolve(__dirname, '../../styles/frontstage.css'),
      'utf-8'
    );
    for (const sel of ['.settings-btn', '.zen-mode-btn', '.wensi-mode-toggle']) {
      const block = css.match(new RegExp(sel.replace('.', '\\.') + '\\s*\\{[^}]+\\}', 's'))?.[0] ?? '';
      expect(block, sel).toMatch(/--transition-press/);
      expect(block, sel).toMatch(/border-radius:\s*6px/);
    }
    expect(css).not.toMatch(/\.settings-btn:hover\s*\{[^}]*scale\(1\.1\)/s);
    expect(css).not.toMatch(/\.zen-mode-btn:hover\s*\{[^}]*scale\(1\.1\)/s);
    expect(css).not.toMatch(/\.wensi-mode-toggle:hover\s*\{[^}]*scale\(1\.1\)/s);
  });
```

文件顶补：`import { readFileSync } from 'node:fs';` 与 `import { resolve } from 'node:path';`（若该测试文件还没有）。`__dirname` 在 vitest ESM 里若不可用，改读：

```ts
import { fileURLToPath } from 'node:url';
const cssPath = resolve(dirname(fileURLToPath(import.meta.url)), '../../styles/frontstage.css');
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cd src-frontend && npx vitest run src/frontstage/components/__tests__/FrontstageHeader.test.tsx`

Expected: FAIL（仍是 `border-radius: 50%` 与 `scale(1.1)`）。

- [ ] **Step 3: 最小实现**

三个按钮共用同一运动契约。把 `.wensi-mode-toggle` / `.zen-mode-btn` / `.settings-btn` 的 `border-radius: 50%` 改为 `6px`；`transition` 改为：

```css
  transition:
    background-color var(--transition-press),
    color var(--transition-press),
    transform var(--transition-press);
```

三个 `:hover` 去掉 `transform: scale(1.1)`。改为陶土淡彩（设置/禅）：

```css
.settings-btn:hover,
.zen-mode-btn:hover {
  background-color: color-mix(in oklch, var(--terracotta) 18%, transparent);
  color: var(--terracotta-dark);
}
```

文思 hover 保持原色语义（active 陶土、passive 金），只去 scale。补 active press：

```css
.settings-btn:active,
.zen-mode-btn:active,
.wensi-mode-toggle:active {
  transform: scale(0.98);
}

@media (prefers-reduced-motion: reduce) {
  .settings-btn:active,
  .zen-mode-btn:active,
  .wensi-mode-toggle:active {
    transform: none;
  }
}
```

空闲颜色保持 `var(--stone-gray)`，不要降 opacity 到近隐。svg `stroke-width` 可改为 `1.75`（设计不变量）。

- [ ] **Step 4: 跑测试确认通过**

Run: `cd src-frontend && npx vitest run src/frontstage/components/__tests__/FrontstageHeader.test.tsx && npx tsc --noEmit && npm run format:check`

Expected: PASS。既有「设置按钮打开幕后」测试仍绿。

- [ ] **Step 5: P1 全量回归**

Run: `cd src-frontend && npx vitest run && npx tsc --noEmit && npm run format:check && python3 ../scripts/architecture_guard.py`

Expected: 全绿。对照设计 §10.2 P1：楷体 Medium 文件在包内、纸微暖、划词淡陶土、顶栏无实心圆弹跳。

- [ ] **Step 6: Commit（仅用户说「提交」时）**

```bash
git add src-frontend/src/frontstage/styles/frontstage.css \
  src-frontend/src/frontstage/components/__tests__/FrontstageHeader.test.tsx
git commit -m "$(cat <<'EOF'
style: 幕前顶栏按钮改为 press 淡彩，去掉 hover 放大

EOF
)"
```

---

### Task 5: P2 暖金内芯同色相

**Files:**
- Modify: `src-frontend/src/styles/__tests__/backstageThemes.test.ts`（`TOKENS_CSS_CURRENT`）
- Modify: `src-frontend/src/styles/backstageThemes.ts`（`warm.vars`）
- Modify: `src-frontend/src/styles/tokens.css`（默认 cinema 850–500）

cool / amber / indigo **不要**改。`BACKSTAGE_THEME_VARS` 名单不要改。

- [ ] **Step 1: 先改测试期望（有意的视觉变化）**

`TOKENS_CSS_CURRENT` 中这五行改为设计 §7.1：

```ts
  '--cinema-850': '#161310',
  '--cinema-800': '#1c1916',
  '--cinema-700': '#26211c',
  '--cinema-600': '#322c25',
  '--cinema-500': '#423a31',
```

950 / 900 / gold / velvet / status **不动**。

- [ ] **Step 2: 跑测试确认失败**

Run: `cd src-frontend && npx vitest run src/styles/__tests__/backstageThemes.test.ts`

Expected: FAIL（`warm 主题全量 16 值与 tokens.css 完全一致` 不相等）。

- [ ] **Step 3: 最小实现**

`backstageThemes.ts` `warm.vars` 同步上表。`tokens.css` `:root` 同步上表。三处 hex 必须字节级相同。

- [ ] **Step 4: 跑测试确认通过**

Run: `cd src-frontend && npx vitest run src/styles/__tests__/backstageThemes.test.ts src/styles/__tests__/aiTokens.test.ts`

Expected: PASS。

- [ ] **Step 5: Commit（仅用户说「提交」时）**

```bash
git add src-frontend/src/styles/backstageThemes.ts \
  src-frontend/src/styles/tokens.css \
  src-frontend/src/styles/__tests__/backstageThemes.test.ts
git commit -m "$(cat <<'EOF'
style: 暖金幕后内芯改为与外壳同一木炭色相

EOF
)"
```

---

### Task 6: P2 Panel 高光 + 弹簧 500ms + 页签/开关

**Files:**
- Modify: `src-frontend/src/styles/__tests__/pressMotion.test.ts`
- Modify: `src-frontend/src/components/ui/__tests__/Panel.test.tsx`
- Modify: `src-frontend/src/styles/tokens.css`（`--transition-spring`）
- Modify: `src-frontend/src/components/ui/Panel.tsx`
- Modify: `src-frontend/src/components/SceneEditor.tsx`（锚点 `STAGE_TABS` 的 `transition-colors`）
- Modify: `src-frontend/src/pages/Stories.tsx`（锚点风格配置 `flex gap-1 mb-4 p-1 bg-cinema-800`）
- Modify: `src-frontend/src/pages/settings/ModelCard.tsx`（`role="switch"` 按钮）

**GitNexus:** `impact({target: "Panel", direction: "upstream"})`。只改 className / 内联 style 字符串，不改 props。HIGH 先报用户。不要改 `Button` API。

- [ ] **Step 1: 写失败测试**

`pressMotion.test.ts` 追加（不要改现有 press 的 `0.3s` 断言）：

```ts
  it('tokens.css 的 --transition-spring 为 0.5s 并在 reduced-motion 冻结', () => {
    const css = readFileSync(resolve(frontendRoot, 'src/styles/tokens.css'), 'utf-8');
    expect(css).toMatch(
      /--transition-spring:\s*0\.5s cubic-bezier\(0\.34,\s*1\.56,\s*0\.64,\s*1\)/
    );
    expect(css).toMatch(/prefers-reduced-motion[\s\S]*--transition-spring:\s*0\.01s linear/);
  });
```

`Panel.test.tsx` 在 `'外壳 bezel + 内芯半径级差'` 之后追加：

```tsx
  it('内芯有 inset 顶边高光，外壳不加第二圈 ring', () => {
    const src = readFileSync(
      resolve(dirname(fileURLToPath(import.meta.url)), '../Panel.tsx'),
      'utf-8'
    );
    expect(src).toMatch(/inset 0 1px 0/);
    expect(src).not.toMatch(/ring-1|ring-2|0 0 0 1px/);
    expect(src).toMatch(/duration-500/);
  });
```

该测试文件顶补 `readFileSync` / `resolve` / `dirname` / `fileURLToPath`（与 `pressMotion.test.ts` 相同导入）。

在 `pressMotion.test.ts` 再追加页签接线探针：

```ts
  it('SceneEditor 与 Stories 页签走 duration-500 ease-spring', () => {
    const scene = readFileSync(resolve(frontendRoot, 'src/components/SceneEditor.tsx'), 'utf-8');
    const stories = readFileSync(resolve(frontendRoot, 'src/pages/Stories.tsx'), 'utf-8');
    expect(scene).toMatch(/duration-500 ease-spring/);
    expect(stories).toMatch(/duration-500 ease-spring/);
  });
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cd src-frontend && npx vitest run src/styles/__tests__/pressMotion.test.ts src/components/ui/__tests__/Panel.test.tsx`

Expected: FAIL（spring 仍是 `0.3s`，Panel 无 `inset 0 1px`）。

- [ ] **Step 3: 最小实现**

`tokens.css`：

```css
  --transition-spring: 0.5s cubic-bezier(0.34, 1.56, 0.64, 1);
```

reduced-motion 块里的 `--transition-spring: 0.01s linear` **不要删**。不要在 `frontstage.css` 新增该变量。

`Panel.tsx`：

- 外壳 class **保留** `border border-borderSubtle`（这就是发丝）。不要加 `ring-*` 或 `0 0 0 1px`。
- 内芯 div（现 `overflow-hidden rounded-[calc(var(--radius-md)-4px)] bg-cinema-850`）加上：

```tsx
        style={{
          boxShadow: 'inset 0 1px 0 color-mix(in oklch, white 6%, transparent)',
        }}
```

- 两处 `duration-300 ease-spring` 改为 `duration-500 ease-spring`（chevron + 内容展开）。

`SceneEditor.tsx` 页签按钮 class 把 `transition-colors` 换成 `transition-[background-color,color] duration-500 ease-spring`。不要给页签加 `hover:scale`。

`Stories.tsx` 风格配置那两个 tab 按钮同样把 `transition-colors` 换成 `transition-[background-color,color] duration-500 ease-spring`。

`ModelCard.tsx` `role="switch"` 的 `transition-colors` 换成 `transition-[background-color,color] duration-500 ease-spring`。不要新做滑块组件。

卡片悬停：本 Task **不要**给 Card 加 `scale`。

- [ ] **Step 4: 跑测试确认通过**

Run: `cd src-frontend && npx vitest run src/styles/__tests__/pressMotion.test.ts src/components/ui/__tests__/Panel.test.tsx src/pages/settings/__tests__/ModelCard.enabled.test.tsx && npx tsc --noEmit`

Expected: PASS。

- [ ] **Step 5: Commit（仅用户说「提交」时）**

```bash
git add src-frontend/src/styles/tokens.css \
  src-frontend/src/styles/__tests__/pressMotion.test.ts \
  src-frontend/src/components/ui/Panel.tsx \
  src-frontend/src/components/ui/__tests__/Panel.test.tsx \
  src-frontend/src/components/SceneEditor.tsx \
  src-frontend/src/pages/Stories.tsx \
  src-frontend/src/pages/settings/ModelCard.tsx
git commit -m "$(cat <<'EOF'
style: 幕后面板补 inset 高光，弹簧改为 500ms

EOF
)"
```

---

### Task 7: P2 侧栏选中去金框 + 空闲 pulse 回归

**Files:**
- Modify: `src-frontend/src/components/__tests__/Sidebar.ia.test.tsx`
- Modify: `src-frontend/src/components/Sidebar.tsx`（锚点 `bg-cinema-gold/10 text-cinema-gold border border-cinema-gold/20`）
- Modify: `src-frontend/src/pages/settings/__tests__/PromptsPanel.test.tsx`

**GitNexus:** `impact({target: "Sidebar", direction: "upstream"})`。只改选中项 className。不要改 `NAV_GROUPS` / 路由 / 徽章文案。

规格写「删 PromptsPanel 空态 FileText pulse」。对照代码：该文件唯一 `animate-pulse` 在 **loading**（「正在加载提示词注册表…」），属于设计 §8.2 允许的「正在干活」。**不要删 loading pulse。** 本 Task 用回归测试锁：非 loading 的标题图标不得 pulse。

不要动：`IntentionGraphDiagnostics` 骨架、`ModelCard` 探测 `running`、`KnowledgeGraph` pending、`Stories` 创世当前步、幕前生成进度条。

- [ ] **Step 1: 写失败测试**

`Sidebar.ia.test.tsx` 追加：

```tsx
  it('选中项只有金淡彩，没有金边框', () => {
    render(<Sidebar currentView="stories" onNavigate={onNavigate} />);
    const item = screen.getByText('故事').closest('button') as HTMLElement;
    expect(item.className).toMatch(/bg-cinema-gold\/10/);
    expect(item.className).not.toMatch(/border-cinema-gold/);
  });
```

`PromptsPanel.test.tsx` 在 `'加载并展示提示词列表'` 的 `waitFor` 之后追加一个独立 `it`：

```tsx
  it('列表加载完成后标题图标不带 animate-pulse', async () => {
    const { container } = render(<PromptsPanel />);
    await waitFor(() => {
      expect(screen.getByText('核心写作提示词')).toBeInTheDocument();
    });
    const pulsing = container.querySelectorAll('.animate-pulse');
    expect(pulsing.length).toBe(0);
  });
```

不要删 loading 态的 pulse（`list_prompt_entries` 未返回前允许存在）。

- [ ] **Step 2: 跑测试确认失败**

Run: `cd src-frontend && npx vitest run src/components/__tests__/Sidebar.ia.test.tsx src/pages/settings/__tests__/PromptsPanel.test.tsx`

Expected: Sidebar 选中项测试 FAIL（仍含 `border-cinema-gold/20`）。PromptsPanel 新测试在现有 mock 下应已 PASS（加载完成后本来就没有 pulse）；若 FAIL，先看是不是 `waitFor` 不够，不要为此去删 loading pulse。

- [ ] **Step 3: 最小实现**

`Sidebar.tsx` 选中 class 从：

```tsx
                          isActive &&
                            'bg-cinema-gold/10 text-cinema-gold border border-cinema-gold/20',
```

改为：

```tsx
                          isActive && 'bg-cinema-gold/10 text-cinema-gold',
```

不要改 `impactBadgeClass`。不要改非选中 `text-gray-400`。

- [ ] **Step 4: 跑测试确认通过**

Run: `cd src-frontend && npx vitest run src/components/__tests__/Sidebar.ia.test.tsx src/pages/settings/__tests__/PromptsPanel.test.tsx && npx tsc --noEmit`

Expected: PASS。

- [ ] **Step 5: P2 全量回归**

Run: `cd src-frontend && npx vitest run && npx tsc --noEmit && npm run format:check && python3 ../scripts/architecture_guard.py`

Expected: 全绿。对照设计 §10.2 P2：暖金内芯不发紫、Panel 浅槽+顶高光、侧栏选中无金框、折叠展开 500ms。

- [ ] **Step 6: Commit（仅用户说「提交」时）**

```bash
git add src-frontend/src/components/Sidebar.tsx \
  src-frontend/src/components/__tests__/Sidebar.ia.test.tsx \
  src-frontend/src/pages/settings/__tests__/PromptsPanel.test.tsx
git commit -m "$(cat <<'EOF'
style: 幕后侧栏选中去掉金边框

EOF
)"
```

---

## 设计对照（执行完自检，不要跳过）

| 规格条款 | Task |
|---|---|
| §5 输入无框、去顶边、去毛玻璃、无 focus 边 | Task 1 |
| §6.1 Medium 分文件 | Task 2 |
| §6.3 选区 22% + §8.1 纸 chroma | Task 3 |
| §6.2 顶栏 press/淡彩，不近隐 | Task 4 |
| §7.1 warm 850–500 hex | Task 5 |
| §7.2 inset 高光、不加第二圈 ring | Task 6 |
| §7.4 spring 500ms + 页签/开关 | Task 6 |
| §7.3 侧栏去金框 | Task 7 |
| §8.2 空闲 pulse；loading 保留 | Task 7（澄清：PromptsPanel 仅 loading 有 pulse，不删） |
| 不变量 §3、拒绝 §4 | Global Constraints |
| `landing/` / IPC / `--ai-*` 名 | 不改 |

---

## 完成后

不要 bump 版本、不要 push、不要宣称全界面截图回归已过。P0/P1/P2 三刀均可在用户要求发版时单独 tag。真机目视：幕前底栏无卡片、未装霞鹜的机器仍是楷体且 500 不是伪斜、暖金幕后内芯不发紫。
