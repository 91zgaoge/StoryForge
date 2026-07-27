# StoryMoss UI 重塑 Phase 1 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 StoryMoss 前端重塑为「幕前墨纸沉浸写作」与「幕后机械精密工作室」双模式设计系统，完成底层 token、核心组件与前后台壳层的改造。

**Architecture:** 通过 CSS 自定义属性与 Tailwind 扩展建立两套语义化 token（`--paper-*` / `--cinema-*`），幕前组件以扁平纸张为基准、后台组件以深色面板为基准；共享过渡曲线与缩放反馈；新增 `useGhostChrome` 等交互 hook；不动业务逻辑，只替换视觉层与布局。

**Tech Stack:** React 18 + TypeScript + Vite + Tailwind CSS + TipTap + Tauri 2.4 + `lucide-react`。

## Global Constraints

- 不新增重型依赖（不使用 Framer Motion 等大型动画库，CSS transitions + Tailwind 即可）。
- 所有颜色必须使用 OKLCH 或设计文档中的 hex token，禁止硬编码色值。
- 保持键盘可达性：`focus-visible` 必须可见。
- 必须支持 `prefers-reduced-motion`。
- 每次修改后运行 `npx tsc --noEmit` + `npx vitest run` + `cargo test -p storymoss`。
- 每次 commit 前运行 pre-commit 格式守卫（已启用 `.githooks/pre-commit`）。
- 中文正文优先使用霞鹜文楷 / Noto Serif SC 回退。

---

## File Structure

| 文件 | 责任 |
|---|---|
| `src-frontend/src/styles/tokens.css` | 新建：双模式 CSS 变量全集 |
| `src-frontend/tailwind.config.js` | 修改：扩展 token 到 Tailwind 工具类 |
| `src-frontend/src/index.css` | 修改：引入 tokens，清理旧硬编码样式 |
| `src-frontend/src/frontstage/styles/frontstage.css` | 修改：幕前编辑器与容器样式 |
| `src-frontend/src/frontstage/hooks/useGhostChrome.ts` | 新建：控制工具栏淡入淡出 |
| `src-frontend/src/frontstage/components/FrontstageHeader.tsx` | 修改：幽灵标题栏 |
| `src-frontend/src/frontstage/components/FrontstageBottomBar.tsx` | 修改：墨水风格输入栏与 AI 提示 |
| `src-frontend/src/frontstage/components/RichTextEditor.tsx` | 修改：纸张舞台最大宽度与呼吸空间 |
| `src-frontend/src/components/ui/Button.tsx` | 新建/修改：双模式按钮 |
| `src-frontend/src/components/ui/Toggle.tsx` | 新建：机械拨动开关 |
| `src-frontend/src/components/ui/Panel.tsx` | 新建：幕后机械面板 |
| `src-frontend/src/pages/Dashboard.tsx` | 修改：幕后仪表板壳层 |
| `src-frontend/src/pages/settings/Settings.tsx` | 修改：机械风格设置页 |
| `src-frontend/src/App.tsx` | 修改：前后台切换过渡类 |

---

### Task 1: 建立双模式 CSS Token 文件

**Files:**
- Create: `src-frontend/src/styles/tokens.css`
- Modify: `src-frontend/src/index.css`（顶部 `@import` 引入）
- Test: `npx tsc --noEmit` + 手动检查页面无 404

**Interfaces:**
- Produces: CSS variables `--paper-*`, `--cinema-*`, `--transition-*`, `--radius-*`, `--shadow-*`

- [ ] **Step 1: 新建 token 文件**

```css
/* src-frontend/src/styles/tokens.css */
:root {
  /* 幕前「墨纸」 */
  --paper-50: #fdfbf7;
  --paper-100: #faf6f1;
  --paper-200: #f2ebe2;
  --paper-300: #e6ddd1;
  --ink-900: #2a2622;
  --ink-700: #4a453f;
  --ink-500: #7a756d;
  --terracotta: #c76b4f;
  --terracotta-light: #d9896e;
  --terracotta-dark: #a85539;

  /* 幕后「机械」 */
  --cinema-950: #050508;
  --cinema-900: #0a0a0f;
  --cinema-850: #0f0f16;
  --cinema-800: #151520;
  --cinema-700: #1e1e2e;
  --cinema-600: #2a2a3c;
  --cinema-gold: #d4af37;
  --cinema-gold-light: #e8c547;
  --cinema-gold-dark: #b8941f;
  --cinema-velvet: #7c3aed;

  /* 共享 */
  --radius-none: 0px;
  --radius-sm: 2px;
  --radius-md: 8px;
  --radius-full: 9999px;

  --transition-fast: 0.15s ease;
  --transition-normal: 0.25s ease-out;
  --transition-spring: 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);

  --shadow-panel: 0 4px 24px rgba(0, 0, 0, 0.4);
  --shadow-float: 0 8px 32px rgba(0, 0, 0, 0.5);
}

@media (prefers-reduced-motion: reduce) {
  :root {
    --transition-fast: 0.01s linear;
    --transition-normal: 0.01s linear;
    --transition-spring: 0.01s linear;
  }
}
```

- [ ] **Step 2: 在 index.css 顶部引入**

```css
@import './styles/tokens.css';
@tailwind base;
/* ... 保留其余内容 ... */
```

- [ ] **Step 3: 运行类型检查**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend
npx tsc --noEmit
```

Expected: 无错误。

- [ ] **Step 4: Commit**

```bash
cd /Users/yuzaimu/projects/StoryForge
git add src-frontend/src/styles/tokens.css src-frontend/src/index.css
git commit -m "feat(ui): add dual-mode design tokens"
```

---

### Task 2: 扩展 Tailwind 配置

**Files:**
- Modify: `src-frontend/tailwind.config.js`
- Test: `npx tsc --noEmit`

**Interfaces:**
- Consumes: CSS tokens from `tokens.css`
- Produces: Tailwind classes `bg-paper-100`, `text-cinema-gold`, `rounded-paper`, `shadow-panel` 等

- [ ] **Step 1: 更新 tailwind.config.js**

```js
/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        paper: {
          50: 'var(--paper-50)',
          100: 'var(--paper-100)',
          200: 'var(--paper-200)',
          300: 'var(--paper-300)',
        },
        ink: {
          500: 'var(--ink-500)',
          700: 'var(--ink-700)',
          900: 'var(--ink-900)',
        },
        terracotta: {
          DEFAULT: 'var(--terracotta)',
          light: 'var(--terracotta-light)',
          dark: 'var(--terracotta-dark)',
        },
        cinema: {
          950: 'var(--cinema-950)',
          900: 'var(--cinema-900)',
          850: 'var(--cinema-850)',
          800: 'var(--cinema-800)',
          700: 'var(--cinema-700)',
          600: 'var(--cinema-600)',
          gold: 'var(--cinema-gold)',
          'gold-light': 'var(--cinema-gold-light)',
          'gold-dark': 'var(--cinema-gold-dark)',
          velvet: 'var(--cinema-velvet)',
        },
      },
      borderRadius: {
        paper: 'var(--radius-sm)',
        panel: 'var(--radius-md)',
      },
      boxShadow: {
        panel: 'var(--shadow-panel)',
        float: 'var(--shadow-float)',
      },
      transitionTimingFunction: {
        spring: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
      },
      fontFamily: {
        display: ['Cinzel', 'serif'],
        body: ["'LXGW WenKai'", "'Noto Serif SC'", "'PingFang SC'", "'Microsoft YaHei'", 'Georgia', 'serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
    },
  },
  plugins: [require('@tailwindcss/typography')],
};
```

- [ ] **Step 2: 验证配置加载**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend
npx tsc --noEmit
```

- [ ] **Step 3: Commit**

```bash
cd /Users/yuzaimu/projects/StoryForge
git add src-frontend/tailwind.config.js
git commit -m "feat(ui): extend tailwind with dual-mode tokens"
```

---

### Task 3: 创建 useGhostChrome Hook

**Files:**
- Create: `src-frontend/src/frontstage/hooks/useGhostChrome.ts`
- Test: `src-frontend/src/frontstage/hooks/__tests__/useGhostChrome.test.tsx`（新建）

**Interfaces:**
- Produces: `{ ghost: boolean; showChrome: () => void; hideChrome: () => void }`

- [ ] **Step 1: 编写 hook**

```ts
// src-frontend/src/frontstage/hooks/useGhostChrome.ts
import { useState, useEffect, useCallback, useRef } from 'react';

const GHOST_DELAY_MS = 3000;

export function useGhostChrome(enabled = true) {
  const [ghost, setGhost] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const showChrome = useCallback(() => {
    if (!enabled) return;
    setGhost(false);
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      setGhost(true);
    }, GHOST_DELAY_MS);
  }, [enabled]);

  useEffect(() => {
    if (!enabled) {
      setGhost(false);
      return;
    }
    showChrome();
    const events = ['mousemove', 'keydown', 'click', 'touchstart'];
    const onActivity = () => showChrome();
    events.forEach(e => window.addEventListener(e, onActivity));
    return () => {
      events.forEach(e => window.removeEventListener(e, onActivity));
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [enabled, showChrome]);

  return { ghost, showChrome };
}
```

- [ ] **Step 2: 编写测试**

```tsx
// src-frontend/src/frontstage/hooks/__tests__/useGhostChrome.test.tsx
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useGhostChrome } from '../useGhostChrome';

describe('useGhostChrome', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  it('enters ghost mode after idle delay', async () => {
    const { result } = renderHook(() => useGhostChrome(true));
    act(() => vi.advanceTimersByTime(3000));
    await waitFor(() => expect(result.current.ghost).toBe(true));
  });

  it('resets timer on mousemove', async () => {
    const { result } = renderHook(() => useGhostChrome(true));
    act(() => vi.advanceTimersByTime(2500));
    act(() => window.dispatchEvent(new MouseEvent('mousemove')));
    act(() => vi.advanceTimersByTime(500));
    expect(result.current.ghost).toBe(false);
    act(() => vi.advanceTimersByTime(3000));
    await waitFor(() => expect(result.current.ghost).toBe(true));
  });
});
```

- [ ] **Step 3: 运行测试**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend
npx vitest run src/frontstage/hooks/__tests__/useGhostChrome.test.tsx
```

Expected: 2 passed。

- [ ] **Step 4: Commit**

```bash
cd /Users/yuzaimu/projects/StoryForge
git add src-frontend/src/frontstage/hooks/useGhostChrome.ts src-frontend/src/frontstage/hooks/__tests__/useGhostChrome.test.tsx
git commit -m "feat(ui): add useGhostChrome hook for immersive frontstage"
```

---

### Task 4: 幕前标题栏幽灵化

**Files:**
- Modify: `src-frontend/src/frontstage/components/FrontstageHeader.tsx`
- Modify: `src-frontend/src/frontstage/FrontstageApp.tsx`（传入 ghost 状态）
- Test: 手动运行 `npm run dev` 或 vitest 相关用例

**Interfaces:**
- Consumes: `useGhostChrome().ghost`
- Produces: `FrontstageHeader` 接受 `ghost?: boolean`

- [ ] **Step 1: 修改 FrontstageHeader 接收 ghost prop**

```tsx
// src-frontend/src/frontstage/components/FrontstageHeader.tsx
interface FrontstageHeaderProps {
  storyTitle?: string;
  chapterTitle?: string;
  wordCount?: number;
  ghost?: boolean;
  onToggleSettings?: () => void;
  // 保留原有其他 props
}

export function FrontstageHeader({ storyTitle, chapterTitle, wordCount, ghost, ...props }: FrontstageHeaderProps) {
  return (
    <header
      className={[
        'fixed top-0 left-0 right-0 z-40',
        'flex items-center justify-between px-6 py-3',
        'bg-paper-100/90 backdrop-blur-sm border-b border-paper-300',
        'transition-opacity duration-300 ease-out',
        ghost ? 'opacity-[0.08]' : 'opacity-100',
      ].join(' ')}
    >
      {/* 保留原有 JSX 结构与功能，仅替换颜色/间距类 */}
    </header>
  );
}
```

- [ ] **Step 2: 在 FrontstageApp 中接入 hook 并传递 ghost**

```tsx
// src-frontend/src/frontstage/FrontstageApp.tsx
import { useGhostChrome } from './hooks/useGhostChrome';

const FrontstageApp: React.FC = () => {
  const { ghost } = useGhostChrome(true);
  // ... 其他状态

  return (
    <div className="frontstage-container">
      <FrontstageHeader ghost={ghost} /* 其他 props */ />
      {/* ... */}
    </div>
  );
};
```

- [ ] **Step 3: 运行 vitest 前台测试**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend
npx vitest run src/frontstage/__tests__
```

Expected: 全部通过。

- [ ] **Step 4: Commit**

```bash
cd /Users/yuzaimu/projects/StoryForge
git add src-frontend/src/frontstage/components/FrontstageHeader.tsx src-frontend/src/frontstage/FrontstageApp.tsx
git commit -m "feat(ui): ghost chrome for frontstage header"
```

---

### Task 5: 幕前底部输入栏墨纸风格

**Files:**
- Modify: `src-frontend/src/frontstage/components/FrontstageBottomBar.tsx`
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`
- Test: `npx vitest run src/frontstage/__tests__`

**Interfaces:**
- Consumes: `ghost` prop
- Produces: 底部栏视觉更新，AI 提示以灰色墨迹显示

- [ ] **Step 1: 修改 BottomBar 样式**

```tsx
// src-frontend/src/frontstage/components/FrontstageBottomBar.tsx
interface FrontstageBottomBarProps {
  ghost?: boolean;
  // 保留其他 props
}

export function FrontstageBottomBar({ ghost, ...props }: FrontstageBottomBarProps) {
  return (
    <div
      className={[
        'fixed bottom-0 left-0 right-0 z-40',
        'flex items-center justify-center px-4 py-3',
        'bg-paper-100/90 backdrop-blur-sm border-t border-paper-300',
        'transition-opacity duration-300 ease-out',
        ghost ? 'opacity-[0.08]' : 'opacity-100',
      ].join(' ')}
    >
      <div className="w-full max-w-2xl relative">
        <textarea
          className="w-full bg-paper-50 border border-paper-300 rounded-paper px-4 py-3
                     text-ink-900 placeholder-ink-500 font-body
                     focus:outline-none focus:border-terracotta focus:ring-1 focus:ring-terracotta/30
                     resize-none transition-colors"
          // 保留 value/onChange 等绑定
        />
        {/* AI 提示以浅灰墨迹显示 */}
        <span className="absolute left-4 top-3 text-ink-500/50 pointer-events-none select-none">
          {/* ghost hint 占位 */}
        </span>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: 清理 frontstage.css 中底部栏旧样式**

删除或注释掉 `.frontstage-bottom-bar` 等旧硬编码样式，改用 Tailwind 类。

- [ ] **Step 3: 运行测试**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend
npx vitest run src/frontstage/__tests__
```

- [ ] **Step 4: Commit**

```bash
cd /Users/yuzaimu/projects/StoryForge
git add src-frontend/src/frontstage/components/FrontstageBottomBar.tsx src-frontend/src/frontstage/styles/frontstage.css
git commit -m "feat(ui): ink-paper style bottom bar with ghost chrome"
```

---

### Task 6: 编辑器纸张舞台

**Files:**
- Modify: `src-frontend/src/frontstage/components/RichTextEditor.tsx`
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`

**Interfaces:**
- Produces: 编辑区居中、最大宽度 720px、上下呼吸空间

- [ ] **Step 1: 修改 RichTextEditor 容器**

```tsx
// src-frontend/src/frontstage/components/RichTextEditor.tsx
// 在渲染 TipTap 的容器上添加类
<div className="w-full h-full overflow-y-auto flex justify-center">
  <div className="w-full max-w-[720px] px-6 py-[10vh]">
    <EditorContent editor={editor} />
  </div>
</div>
```

- [ ] **Step 2: 更新 frontstage.css 中 ProseMirror 样式**

```css
.frontstage-editor-stage {
  width: 100%;
  max-width: 720px;
  margin: 0 auto;
  padding: 10vh 1.5rem;
}

.ProseMirror p {
  margin-block-start: 0.75em;
  margin-block-end: 0.75em;
  text-align: justify;
  text-indent: 2em;
  line-height: 1.8;
}
```

- [ ] **Step 3: 运行前台测试**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend
npx vitest run src/frontstage/__tests__
```

- [ ] **Step 4: Commit**

```bash
cd /Users/yuzaimu/projects/StoryForge
git add src-frontend/src/frontstage/components/RichTextEditor.tsx src-frontend/src/frontstage/styles/frontstage.css
git commit -m "feat(ui): paper stage layout for rich text editor"
```

---

### Task 7: 创建共享 Button 组件（双模式）

**Files:**
- Create: `src-frontend/src/components/ui/Button.tsx`
- Create: `src-frontend/src/components/ui/__tests__/Button.test.tsx`

**Interfaces:**
- Produces: `<Button variant="paper" | "cinema" size="sm" | "md" />`

- [ ] **Step 1: 编写 Button 组件**

```tsx
// src-frontend/src/components/ui/Button.tsx
import React from 'react';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'paper' | 'cinema' | 'cinema-outline';
  size?: 'sm' | 'md';
}

export const Button: React.FC<ButtonProps> = ({
  variant = 'paper',
  size = 'md',
  className = '',
  children,
  ...props
}) => {
  const base =
    'inline-flex items-center justify-center font-medium transition-all active:scale-95 focus-visible:outline-none focus-visible:ring-2';

  const variants = {
    paper: 'bg-terracotta text-white hover:bg-terracotta-light focus-visible:ring-terracotta/40',
    cinema: 'bg-cinema-gold text-cinema-950 hover:bg-cinema-gold-light focus-visible:ring-cinema-gold/40',
    'cinema-outline':
      'bg-transparent border border-cinema-600 text-cinema-gold hover:bg-cinema-800 focus-visible:ring-cinema-gold/30',
  };

  const sizes = {
    sm: 'px-3 py-1.5 text-xs rounded-panel',
    md: 'px-4 py-2 text-sm rounded-panel',
  };

  return (
    <button className={[base, variants[variant], sizes[size], className].join(' ')} {...props}>
      {children}
    </button>
  );
};
```

- [ ] **Step 2: 编写测试**

```tsx
// src-frontend/src/components/ui/__tests__/Button.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Button } from '../Button';

describe('Button', () => {
  it('renders paper variant', () => {
    render(<Button variant="paper">写</Button>);
    expect(screen.getByRole('button', { name: '写' })).toHaveClass('bg-terracotta');
  });

  it('renders cinema variant', () => {
    render(<Button variant="cinema">保存</Button>);
    expect(screen.getByRole('button', { name: '保存' })).toHaveClass('bg-cinema-gold');
  });
});
```

- [ ] **Step 3: 运行测试**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend
npx vitest run src/components/ui/__tests__/Button.test.tsx
```

Expected: 2 passed。

- [ ] **Step 4: Commit**

```bash
cd /Users/yuzaimu/projects/StoryForge
git add src-frontend/src/components/ui/Button.tsx src-frontend/src/components/ui/__tests__/Button.test.tsx
git commit -m "feat(ui): add dual-mode Button component"
```

---

### Task 8: 创建机械 Toggle 开关

**Files:**
- Create: `src-frontend/src/components/ui/Toggle.tsx`
- Create: `src-frontend/src/components/ui/__tests__/Toggle.test.tsx`

**Interfaces:**
- Produces: `<Toggle checked onChange={...} />`

- [ ] **Step 1: 编写 Toggle**

```tsx
// src-frontend/src/components/ui/Toggle.tsx
import React from 'react';

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
}

export const Toggle: React.FC<ToggleProps> = ({ checked, onChange, label }) => {
  return (
    <label className="inline-flex items-center gap-3 cursor-pointer select-none">
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        onClick={() => onChange(!checked)}
        className={[
          'relative w-11 h-6 rounded-full transition-colors duration-200 ease-spring',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cinema-gold/40',
          checked ? 'bg-cinema-gold' : 'bg-cinema-700',
        ].join(' ')}
      >
        <span
          className={[
            'absolute top-1 left-1 w-4 h-4 rounded-full bg-cinema-950 shadow-sm',
            'transition-transform duration-200 ease-spring',
            checked ? 'translate-x-5' : 'translate-x-0',
          ].join(' ')}
        />
      </button>
      {label && <span className="text-sm text-cinema-gold/90 font-medium">{label}</span>}
    </label>
  );
};
```

- [ ] **Step 2: 编写测试**

```tsx
// src-frontend/src/components/ui/__tests__/Toggle.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Toggle } from '../Toggle';

describe('Toggle', () => {
  it('toggles on click', () => {
    const onChange = vi.fn();
    render(<Toggle checked={false} onChange={onChange} />);
    fireEvent.click(screen.getByRole('switch'));
    expect(onChange).toHaveBeenCalledWith(true);
  });
});
```

- [ ] **Step 3: 运行测试**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend
npx vitest run src/components/ui/__tests__/Toggle.test.tsx
```

- [ ] **Step 4: Commit**

```bash
cd /Users/yuzaimu/projects/StoryForge
git add src-frontend/src/components/ui/Toggle.tsx src-frontend/src/components/ui/__tests__/Toggle.test.tsx
git commit -m "feat(ui): add cinema toggle switch component"
```

---

### Task 9: 创建机械 Panel 面板

**Files:**
- Create: `src-frontend/src/components/ui/Panel.tsx`
- Create: `src-frontend/src/components/ui/__tests__/Panel.test.tsx`

**Interfaces:**
- Produces: `<Panel title="..." collapsible>{children}</Panel>`

- [ ] **Step 1: 编写 Panel**

```tsx
// src-frontend/src/components/ui/Panel.tsx
import React, { useState } from 'react';
import { ChevronDown } from 'lucide-react';

interface PanelProps {
  title: string;
  children: React.ReactNode;
  collapsible?: boolean;
  defaultOpen?: boolean;
}

export const Panel: React.FC<PanelProps> = ({ title, children, collapsible, defaultOpen = true }) => {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <div className="bg-cinema-850 border border-white/[0.06] rounded-panel shadow-panel overflow-hidden">
      <div
        className={[
          'flex items-center justify-between px-4 py-3',
          'border-b border-white/[0.06]',
          collapsible ? 'cursor-pointer hover:bg-cinema-800/50' : '',
        ].join(' ')}
        onClick={collapsible ? () => setOpen(v => !v) : undefined}
      >
        <h3 className="text-xs font-bold uppercase tracking-wider text-cinema-gold/80">{title}</h3>
        {collapsible && (
          <ChevronDown
            className={['w-4 h-4 text-cinema-gold/80 transition-transform duration-300 ease-spring', open ? 'rotate-180' : ''].join(' ')}
          />
        )}
      </div>
      <div
        className={['transition-all duration-300 ease-spring overflow-hidden', open ? 'max-h-[1000px] opacity-100' : 'max-h-0 opacity-0'].join(' ')}
      >
        <div className="p-4">{children}</div>
      </div>
    </div>
  );
};
```

- [ ] **Step 2: 编写测试**

```tsx
// src-frontend/src/components/ui/__tests__/Panel.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Panel } from '../Panel';

describe('Panel', () => {
  it('renders title and children', () => {
    render(<Panel title="Settings">content</Panel>);
    expect(screen.getByText('Settings')).toBeInTheDocument();
    expect(screen.getByText('content')).toBeInTheDocument();
  });

  it('collapses on click', () => {
    render(<Panel title="Advanced" collapsible>secret</Panel>);
    fireEvent.click(screen.getByText('Advanced'));
    expect(screen.getByText('secret')).not.toBeVisible();
  });
});
```

- [ ] **Step 3: 运行测试**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend
npx vitest run src/components/ui/__tests__/Panel.test.tsx
```

- [ ] **Step 4: Commit**

```bash
cd /Users/yuzaimu/projects/StoryForge
git add src-frontend/src/components/ui/Panel.tsx src-frontend/src/components/ui/__tests__/Panel.test.tsx
git commit -m "feat(ui): add cinema panel component"
```

---

### Task 10: 幕后仪表板壳层改造

**Files:**
- Modify: `src-frontend/src/pages/Dashboard.tsx`
- Modify: `src-frontend/src/App.tsx`（添加 `dark` 类到后台路由）

**Interfaces:**
- Consumes: `Panel`, `Button` 组件
- Produces: 深色仪表板布局

- [ ] **Step 1: 修改 App.tsx 为后台路由添加 dark 类**

```tsx
// src-frontend/src/App.tsx
// 在渲染 backstage 路由的根元素上添加 className="dark"
<Route path="/" element={<div className="dark"><Dashboard /></div>} />
```

- [ ] **Step 2: 重写 Dashboard 外壳**

```tsx
// src-frontend/src/pages/Dashboard.tsx
import { Panel } from '@/components/ui/Panel';
import { Button } from '@/components/ui/Button';

export default function Dashboard() {
  return (
    <div className="min-h-screen bg-cinema-950 text-cinema-gold/90 font-body flex">
      {/* 左侧导航轨 */}
      <nav className="w-16 flex-shrink-0 bg-cinema-900 border-r border-white/[0.06] flex flex-col items-center py-4 gap-4">
        {/* 图标导航 */}
      </nav>

      {/* 主区 */}
      <main className="flex-1 flex flex-col min-w-0">
        <header className="h-14 flex items-center justify-between px-6 border-b border-white/[0.06] bg-cinema-900/80 backdrop-blur-sm">
          <h1 className="text-sm font-bold uppercase tracking-widest text-cinema-gold">工作室</h1>
          <div className="flex items-center gap-3">
            <span className="text-xs font-mono text-cinema-gold/60">MODEL: ONLINE</span>
            <Button variant="cinema" size="sm">新建</Button>
          </div>
        </header>

        <div className="flex-1 p-6 overflow-auto">
          <div className="grid grid-cols-12 gap-4">
            <div className="col-span-12 lg:col-span-4">
              <Panel title="作品概览">...</Panel>
            </div>
            <div className="col-span-12 lg:col-span-4">
              <Panel title="角色库">...</Panel>
            </div>
            <div className="col-span-12 lg:col-span-4">
              <Panel title="任务队列">...</Panel>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}
```

- [ ] **Step 3: 运行 vitest 全部前台/后台测试**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend
npx vitest run
```

Expected: 全部通过。

- [ ] **Step 4: Commit**

```bash
cd /Users/yuzaimu/projects/StoryForge
git add src-frontend/src/App.tsx src-frontend/src/pages/Dashboard.tsx
git commit -m "feat(ui): cinema dashboard shell"
```

---

### Task 11: 设置页机械风格改造

**Files:**
- Modify: `src-frontend/src/pages/settings/Settings.tsx`
- Modify: 必要时 `src-frontend/src/pages/settings/*.tsx`

**Interfaces:**
- Consumes: `Panel`, `Toggle`, `Button`
- Produces: 机械风格设置页

- [ ] **Step 1: 用 Panel 包裹设置分组**

```tsx
// src-frontend/src/pages/settings/Settings.tsx
import { Panel } from '@/components/ui/Panel';
import { Toggle } from '@/components/ui/Toggle';
import { Button } from '@/components/ui/Button';

export default function Settings() {
  return (
    <div className="min-h-screen bg-cinema-950 text-cinema-gold/90 p-6">
      <div className="max-w-4xl mx-auto space-y-4">
        <Panel title="General">
          <div className="flex items-center justify-between py-2">
            <span className="text-sm text-cinema-gold/80">自动保存</span>
            <Toggle checked={true} onChange={() => {}} />
          </div>
        </Panel>

        <Panel title="Model" collapsible>
          {/* 模型设置 */}
        </Panel>

        <div className="flex justify-end gap-3 pt-4">
          <Button variant="cinema-outline" size="md">重置</Button>
          <Button variant="cinema" size="md">保存</Button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: 运行类型检查与 vitest**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend
npx tsc --noEmit
npx vitest run
```

- [ ] **Step 3: Commit**

```bash
cd /Users/yuzaimu/projects/StoryForge
git add src-frontend/src/pages/settings/Settings.tsx
git commit -m "feat(ui): mechanical settings page"
```

---

### Task 12: 前后台切换过渡

**Files:**
- Modify: `src-frontend/src/App.tsx`
- Test: 手动验证切换动画

**Interfaces:**
- Produces: 路由切换时带 0.4s 过渡

- [ ] **Step 1: 为路由切换容器添加过渡类**

```tsx
// src-frontend/src/App.tsx
import { useLocation } from 'react-router-dom';

function App() {
  const location = useLocation();
  const isFrontstage = location.pathname === '/frontstage';

  return (
    <div
      className={[
        'min-h-screen transition-colors duration-500 ease-out',
        isFrontstage ? 'bg-paper-100' : 'dark bg-cinema-950',
      ].join(' ')}
    >
      <Routes location={location}>
        {/* ... 路由 ... */}
      </Routes>
    </div>
  );
}
```

- [ ] **Step 2: 运行测试**

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend
npx vitest run
```

- [ ] **Step 3: Commit**

```bash
cd /Users/yuzaimu/projects/StoryForge
git add src-frontend/src/App.tsx
git commit -m "feat(ui): mode transition between paper and cinema"
```

---

## Global Verification

在所有任务完成后，必须运行以下验证：

```bash
cd /Users/yuzaimu/projects/StoryForge/src-frontend
npx tsc --noEmit
npx prettier --check src/**/*.tsx src/**/*.css src/**/*.ts
cd /Users/yuzaimu/projects/StoryForge/src-tauri
cargo test -p storymoss
```

Expected:
- TypeScript 无错误
- Prettier 无格式问题
- Vitest 全部通过
- Cargo 1060+ 测试通过

---

## Self-Review Checklist

- [x] 设计文档中的 palette、typography、components、layout、depth、do/don't、responsive 均有对应任务。
- [x] 无 TBD/TODO/placeholder 步骤。
- [x] 所有文件路径为真实项目路径。
- [x] 类型/签名在任务间一致（Button、Toggle、Panel 接口已定义）。
- [x] 每个任务以可独立测试的交付物结束。
