# P1 AI 原生组件库第一批（生成体验）Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 beautifului.dev 的 5 个 AI 原生组件（LoadingState / ThinkingState / StreamingText / PromptBar / ApprovalCard）适配为受控组件入库 `src-frontend/src/components/ui/ai/`，并逐点替换/接入：幕后 3 处加载指示（GenesisPanel / GuidebookDistillationPanel / NovelCreationWizard）、AgencyStudio 执行轨迹、幕前幽灵续写渲染、幕前底部指令输入条、创建向导选项步骤。为此先建 `--ai-*` 语义令牌桥（幕后 tokens.css / 幕前 frontstage.css 双窗口各自定义，同一组件代码两侧正确着色）。

**Architecture:** 令牌层（`--ai-*` 变量 + tailwind `ai-*` 色与 keyframes）→ 组件层（`components/ui/ai/`，全部受控、无自运行演示逻辑）→ 集成层（逐文件 before/after 替换）。组件只引用 `--ai-*` 语义令牌与 tailwind 注册的 keyframes 工具类，不写死颜色、不引新依赖；图标用既有 `lucide-react`；动画全部手写 CSS keyframes（PromptBar 的 glimm 彩虹扫光降级为一次性 `ai-sweep` 渐变扫过）。

**Tech Stack:** React 18 + Tailwind v3.4（`var()` 色映射）、vitest 4 + Testing Library、jsdom、lucide-react（既有依赖）。

## Global Constraints

- 仓库 /Users/yuzaimu/projects/StoryForge；master 直接工作；中文 conventional commit；不 --no-verify；不推送、不打 tag。
- **不引入新依赖**：禁止 `liveline` / `glimm` / `iconoir-react` / framer-motion 参与本批组件（framer-motion 虽在 package.json 中，本批不用）；图标只用 `lucide-react`。
- 组件全部为**受控组件**：剥离参考实现中的 STAGES/useSequence/AUTO_STEPS/TOKENS/QUESTIONS 等自运行演示逻辑；无内部自动计时步进（AiLoading 的 elapsed 计时器除外，它是显示件）。
- 不改 `FrontstageApp.tsx` 的打字机/race-lock 逻辑（Task 4 只动 RichTextEditor 的渲染包裹）。
- 两个窗口是独立 webview 文档：`--ai-*` 必须在 tokens.css（幕后）与 frontstage.css（幕前）**各自定义**，值不同。
- 准入线：`cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check` 全绿 + 仓库根 `python3 scripts/architecture_guard.py` 通过；vitest 基线 **455 passed / 3 skipped**，只允许增加。
- 设计文档：`docs/plans/2026-08-12-beautifului-ai-native-design.md`（§8 P1 范围）；参考组件源码：`.superpowers/sdd/reference/beautifului/`。
- 参考实现的 Tailwind 令牌（`bg-surface`/`text-ink`/`border-line` 等）一律改写为本计划的 `ai-*` 令牌（`bg-ai-surface`/`text-ai-ink`/`border-ai-line`）；`rounded-control`/`shadow-card` 等站点私有类用本项目工具类/内联样式等价替代。

---

### Task 1: `--ai-*` 语义令牌桥 + AI keyframes（tokens.css + frontstage.css + tailwind.config.js）

**Files:**
- Modify: `src-frontend/src/styles/tokens.css`（幕后值 + reduced-motion 冻结）
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`（幕前值 + reduced-motion 冻结）
- Modify: `src-frontend/tailwind.config.js`（`ai-*` 色映射 + keyframes/animation 注册）
- Test: `src-frontend/src/styles/__tests__/aiTokens.test.ts`

**Interfaces:**
- Consumes: 既有 `--cinema-*` / `--status-*`（tokens.css）、`--ivory` / `--warm-sand` / `--border-cream` / `--parchment-dark` / `--charcoal` / `--olive-gray` / `--stone-gray` / `--terracotta` / `--terracotta-dark`（frontstage.css）
- Produces:
  - CSS 变量（两窗口同名不同值）：`--ai-surface` `--ai-inset` `--ai-field` `--ai-hover` `--ai-hover-2` `--ai-ink` `--ai-ink-2` `--ai-ink-3` `--ai-line` `--ai-line-strong` `--ai-accent` `--ai-accent-ink` `--ai-accent-tint` `--ai-green` `--ai-red` `--ai-orange`（共 16 个）
  - Tailwind 色工具：`bg-ai-surface` / `text-ai-ink` / `text-ai-ink-2` / `text-ai-ink-3` / `border-ai-line` / `border-ai-line-strong` / `bg-ai-accent` / `text-ai-accent-ink` / `bg-ai-accent-tint` / `text-ai-green` / `text-ai-red` / `bg-ai-green` 等（`ai.<key>` 全组）
  - Tailwind 动画工具：`animate-pixel-on` `animate-shimmer-text` `animate-ai-fade-up` `animate-pop-in` `animate-stream-in` `animate-ai-spin` `animate-eq-bounce` `animate-ai-sweep` `animate-ai-blink`（Task 2-6 组件引用）
  - 注意：`animate-ai-fade-up` 是**新工具名**——既有 `animate-fade-up`（fadeUp 0.6s 大幕动画，FrontstageBottomBar 在用）不可覆盖；新 keyframe 名为 `fade-up`，二者不冲突。

- [ ] **Step 1: Write the failing test**

```typescript
// src-frontend/src/styles/__tests__/aiTokens.test.ts
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
// 从 src-frontend/src/styles/__tests__/ 回到 src-frontend/：../../../
const frontendRoot = resolve(__dirname, '..', '..', '..');

const tokensCss = readFileSync(resolve(frontendRoot, 'src/styles/tokens.css'), 'utf-8');
const frontstageCss = readFileSync(
  resolve(frontendRoot, 'src/frontstage/styles/frontstage.css'),
  'utf-8'
);
const tailwindConfig = readFileSync(resolve(frontendRoot, 'tailwind.config.js'), 'utf-8');

/** P1 语义令牌全集（两窗口必须各自定义） */
const AI_VARS = [
  '--ai-surface',
  '--ai-inset',
  '--ai-field',
  '--ai-hover',
  '--ai-hover-2',
  '--ai-ink',
  '--ai-ink-2',
  '--ai-ink-3',
  '--ai-line',
  '--ai-line-strong',
  '--ai-accent',
  '--ai-accent-ink',
  '--ai-accent-tint',
  '--ai-green',
  '--ai-red',
  '--ai-orange',
];

/** tailwind.config.js 中 ai 色组的 key（映射目标即同名变量） */
const AI_COLOR_KEYS = [
  'surface',
  'inset',
  'field',
  'hover',
  'hover-2',
  'ink',
  'ink-2',
  'ink-3',
  'line',
  'line-strong',
  'accent',
  'accent-ink',
  'accent-tint',
  'green',
  'red',
  'orange',
];

/** tailwind.config.js keyframes 注册表（ai-fade-up 工具的 keyframe 名为 fade-up） */
const AI_KEYFRAME_ENTRIES = [
  "'pixel-on': {",
  "'shimmer-text': {",
  "'fade-up': {",
  "'pop-in': {",
  "'stream-in': {",
  "'ai-spin': {",
  "'eq-bounce': {",
  "'ai-sweep': {",
  "'ai-blink': {",
];

const AI_ANIMATION_UTILS = [
  'pixel-on',
  'shimmer-text',
  'ai-fade-up',
  'pop-in',
  'stream-in',
  'ai-spin',
  'eq-bounce',
  'ai-sweep',
  'ai-blink',
];

describe('AI 语义令牌桥（P1 Task1）', () => {
  it('tokens.css（幕后）定义全部 16 个 --ai-* 变量', () => {
    for (const v of AI_VARS) {
      expect(tokensCss, `tokens.css 缺 ${v}`).toContain(`${v}:`);
    }
  });

  it('frontstage.css（幕前）定义全部 16 个 --ai-* 变量', () => {
    for (const v of AI_VARS) {
      expect(frontstageCss, `frontstage.css 缺 ${v}`).toContain(`${v}:`);
    }
  });

  it('tailwind.config.js 将 ai 色组映射到 var(--ai-*)', () => {
    for (const key of AI_COLOR_KEYS) {
      expect(tailwindConfig, `tailwind 缺 ai.${key} 映射`).toContain(`var(--ai-${key})`);
    }
  });

  it('tailwind.config.js 注册全部 AI keyframes 与动画工具', () => {
    for (const entry of AI_KEYFRAME_ENTRIES) {
      expect(tailwindConfig, `tailwind 缺 keyframe ${entry}`).toContain(entry);
    }
    for (const util of AI_ANIMATION_UTILS) {
      expect(tailwindConfig, `tailwind 缺 animation 工具 ${util}`).toContain(`'${util}':`);
    }
  });

  it('两个窗口 CSS 均有 prefers-reduced-motion 动画冻结选择器', () => {
    for (const [name, css] of [
      ['tokens.css', tokensCss],
      ['frontstage.css', frontstageCss],
    ] as const) {
      expect(css, `${name} 缺 .animate-pixel-on 冻结`).toContain('.animate-pixel-on');
      expect(css, `${name} 缺 .animate-ai-sweep 冻结`).toContain('.animate-ai-sweep');
    }
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/styles/__tests__/aiTokens.test.ts`
Expected: FAIL（5 个用例全挂：`--ai-*` 变量与 keyframes 均不存在）

- [ ] **Step 3: Write implementation**

**(a) `src-frontend/src/styles/tokens.css`** — 在 `--border-subtle: rgba(255, 255, 255, 0.06);` 行后插入：

```css

  /* ===== AI 原生组件语义令牌（P1）——幕后值；幕前同名变量在 frontstage.css ===== */
  --ai-surface: var(--cinema-900);
  --ai-inset: var(--cinema-850);
  --ai-field: var(--cinema-800);
  --ai-hover: var(--cinema-700);
  --ai-hover-2: var(--cinema-600);
  --ai-ink: #eceaf4;
  --ai-ink-2: #b8b4cc;
  --ai-ink-3: #7d7890;
  --ai-line: var(--cinema-700);
  --ai-line-strong: var(--cinema-500);
  --ai-accent: var(--cinema-gold);
  --ai-accent-ink: var(--cinema-gold-light);
  --ai-accent-tint: rgba(212, 175, 55, 0.12);
  --ai-green: var(--status-success);
  --ai-red: var(--status-danger);
  --ai-orange: var(--status-warning);
```

文件末尾的 `@media (prefers-reduced-motion: reduce)` 块整体替换为：

```css
@media (prefers-reduced-motion: reduce) {
  :root {
    --transition-fast: 0.01s linear;
    --transition-normal: 0.01s linear;
    --transition-spring: 0.01s linear;
  }
  /* AI 组件动画冻结（AiLoading 计时器等非动画逻辑不受影响） */
  .animate-pixel-on,
  .animate-shimmer-text,
  .animate-ai-fade-up,
  .animate-pop-in,
  .animate-stream-in,
  .animate-ai-spin,
  .animate-eq-bounce,
  .animate-ai-sweep,
  .animate-ai-blink {
    animation: none !important;
  }
}
```

**(b) `src-frontend/src/frontstage/styles/frontstage.css`** — 在 `:root` 内 `--text-on-accent: oklch(100% 0 0);` 行后插入：

```css

  /* ===== AI 原生组件语义令牌（P1）——幕前值；幕后同名变量在 tokens.css ===== */
  --ai-surface: var(--ivory);
  --ai-inset: var(--warm-sand);
  --ai-field: var(--border-cream);
  --ai-hover: var(--parchment-dark);
  --ai-hover-2: var(--warm-sand);
  --ai-ink: var(--charcoal);
  --ai-ink-2: var(--olive-gray);
  --ai-ink-3: var(--stone-gray);
  --ai-line: var(--border-cream);
  --ai-line-strong: var(--stone-gray);
  --ai-accent: var(--terracotta);
  --ai-accent-ink: var(--terracotta-dark);
  --ai-accent-tint: oklch(58% 0.13 45 / 0.12);
  --ai-green: oklch(50% 0.18 145);
  --ai-red: oklch(50% 0.18 25);
  --ai-orange: #f59e0b;
```

（取值依据：hover/hover-2 取 parchment-dark 93.5% / warm-sand 91% 两级递深；green/red 取本文件 ingest 健康徽章同款 oklch(50% 0.18 145/25)；orange 取本文件 pending 徽章同款 #f59e0b；line-strong 取 stone-gray。）

文件末尾追加：

```css

/* ===== AI 组件动画冻结（P1，与 tokens.css 幕后同款） ===== */
@media (prefers-reduced-motion: reduce) {
  .animate-pixel-on,
  .animate-shimmer-text,
  .animate-ai-fade-up,
  .animate-pop-in,
  .animate-stream-in,
  .animate-ai-spin,
  .animate-eq-bounce,
  .animate-ai-sweep,
  .animate-ai-blink {
    animation: none !important;
  }
}
```

**(c) `src-frontend/tailwind.config.js`** — 在 colors 的 `borderSubtle: 'var(--border-subtle)',` 行后插入：

```js
        // AI 原生组件语义令牌（P1）：tokens.css（幕后）/frontstage.css（幕前）各自定义
        ai: {
          surface: 'var(--ai-surface)',
          inset: 'var(--ai-inset)',
          field: 'var(--ai-field)',
          hover: 'var(--ai-hover)',
          'hover-2': 'var(--ai-hover-2)',
          ink: 'var(--ai-ink)',
          'ink-2': 'var(--ai-ink-2)',
          'ink-3': 'var(--ai-ink-3)',
          line: 'var(--ai-line)',
          'line-strong': 'var(--ai-line-strong)',
          accent: 'var(--ai-accent)',
          'accent-ink': 'var(--ai-accent-ink)',
          'accent-tint': 'var(--ai-accent-tint)',
          green: 'var(--ai-green)',
          red: 'var(--ai-red)',
          orange: 'var(--ai-orange)',
        },
```

在 animation 的 `'spin-slow': 'spin 3s linear infinite',` 行后插入：

```js
        // AI 原生组件动画（P1）。ai-fade-up 区别于既有 fade-up（0.6s 大幕入场），勿复用。
        'pixel-on': 'pixel-on 650ms ease-in-out infinite',
        'shimmer-text': 'shimmer-text 1.4s linear infinite',
        'ai-fade-up': 'fade-up 350ms cubic-bezier(0.23, 1, 0.32, 1) both',
        'pop-in': 'pop-in 250ms cubic-bezier(0.23, 1, 0.32, 1) both',
        'stream-in': 'stream-in 420ms cubic-bezier(0.22, 0.61, 0.25, 1) both',
        'ai-spin': 'ai-spin 700ms linear infinite',
        'eq-bounce': 'eq-bounce 900ms ease-in-out infinite',
        'ai-sweep': 'ai-sweep 950ms ease-out both',
        'ai-blink': 'ai-blink 1.1s steps(2, start) infinite',
```

在 keyframes 的 `slideLeft: { ... },` 块后插入：

```js
        'pixel-on': {
          '0%, 100%': { opacity: '0.15' },
          '50%': { opacity: '1' },
        },
        'shimmer-text': {
          '0%': { backgroundPosition: '200% 0' },
          '100%': { backgroundPosition: '-200% 0' },
        },
        'fade-up': {
          '0%': { opacity: '0', transform: 'translateY(6px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        'pop-in': {
          '0%': { opacity: '0', transform: 'scale(0.92)' },
          '100%': { opacity: '1', transform: 'scale(1)' },
        },
        'stream-in': {
          '0%': { opacity: '0', filter: 'blur(6px)' },
          '100%': { opacity: '1', filter: 'blur(0)' },
        },
        'ai-spin': {
          '100%': { transform: 'rotate(360deg)' },
        },
        'eq-bounce': {
          '0%, 100%': { transform: 'scaleY(0.35)' },
          '50%': { transform: 'scaleY(1)' },
        },
        'ai-sweep': {
          '0%': { transform: 'translateX(-120%)', opacity: '0' },
          '12%': { opacity: '0.9' },
          '85%': { opacity: '0.9' },
          '100%': { transform: 'translateX(240%)', opacity: '0' },
        },
        'ai-blink': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0' },
        },
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-frontend && npx vitest run src/styles/__tests__/aiTokens.test.ts && npx tsc --noEmit`
Expected: 5 passed；tsc 干净

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/styles/tokens.css src-frontend/src/frontstage/styles/frontstage.css src-frontend/tailwind.config.js src-frontend/src/styles/__tests__/aiTokens.test.ts
git commit -m "feat: AI 原生组件语义令牌桥与 keyframes（P1 Task1）"
```

---

### Task 2: AiLoading 组件 + 幕后 3 处加载指示替换

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiLoading.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiLoading.test.tsx`
- Modify: `src-frontend/src/components/GenesisPanel.tsx`（L443-457 当前步 spinner；import 区 L1-18）
- Modify: `src-frontend/src/components/guidebook-distillation/GuidebookDistillationPanel.tsx`（L122-125 状态图标、L152-155 进度块文案；import 区 L1-14）
- Modify: `src-frontend/src/components/NovelCreationWizard.tsx`（`renderGenerating` L257-267）

**Interfaces:**
- Consumes: Task 1 的 `animate-pixel-on` / `animate-shimmer-text` / `bg-ai-ink` / `text-ai-ink-3`
- Produces:
  - `export interface AiLoadingProps { label: string; variant?: 'drive' | 'dots' | 'orbit'; startedAt?: number }`
  - `export function AiLoading(props: AiLoadingProps): JSX.Element` — elapsed 计时从 `startedAt`（默认挂载时刻）起算，100ms 粒度，mono tabular figures；`data-testid="ai-loading"` / `ai-loading-elapsed`

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiLoading.test.tsx
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import { AiLoading } from '../AiLoading';

describe('AiLoading', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('渲染 label、9 格点与计时器', () => {
    const { container } = render(<AiLoading label="正在生成世界观" />);
    expect(screen.getByText('正在生成世界观')).toBeInTheDocument();
    expect(container.querySelectorAll('[aria-hidden] > span')).toHaveLength(9);
    expect(screen.getByTestId('ai-loading-elapsed').textContent).toMatch(/s$/);
  });

  it('elapsed 从 startedAt 起算', () => {
    vi.useFakeTimers();
    vi.setSystemTime(1_000_000);
    render(<AiLoading label="x" startedAt={995_000} />);
    expect(screen.getByTestId('ai-loading-elapsed').textContent).toBe('5.0s');
  });

  it('计时随时间推进（100ms 粒度）', () => {
    vi.useFakeTimers();
    render(<AiLoading label="x" />);
    act(() => {
      vi.advanceTimersByTime(2100);
    });
    expect(screen.getByTestId('ai-loading-elapsed').textContent).toBe('2.1s');
  });

  it('超过 60s 显示 m+s；orbit 变体中心格不点亮', () => {
    vi.useFakeTimers();
    vi.setSystemTime(200_000);
    const { container } = render(<AiLoading label="x" variant="orbit" startedAt={125_000} />);
    expect(screen.getByTestId('ai-loading-elapsed').textContent).toBe('1m 15.0s');
    // orbit 模式中心格（index 4）无动画
    const cells = container.querySelectorAll('[aria-hidden] > span');
    expect(cells[4].className).not.toContain('animate-pixel-on');
    expect(cells[0].className).toContain('animate-pixel-on');
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiLoading.test.tsx`
Expected: FAIL（模块 `../AiLoading` 不存在）

- [ ] **Step 3: Write implementation**

```tsx
// src-frontend/src/components/ui/ai/AiLoading.tsx
/**
 * AiLoading — 像素格点加载器（适配自 beautifului LoadingState）
 *
 * 受控组件：label 由调用方提供；elapsed 计时从 startedAt（默认挂载时刻）起算。
 * variant：drive（方块 chevron 波前）/ dots（圆点波前）/ orbit（彗星绕边，中心格不点亮）。
 * 动画 = Task 1 注册的 CSS keyframes（pixel-on / shimmer-text）；
 * prefers-reduced-motion 下动画冻结（tokens.css / frontstage.css 冻结块），计时仍走。
 */
import { useEffect, useState } from 'react';

export interface AiLoadingProps {
  label: string;
  variant?: 'drive' | 'dots' | 'orbit';
  startedAt?: number;
}

/* chevron 波前：格点按列+行距依次点亮，650ms 周期短于扫描，两个波前同时在场 */
const chevron = Array.from({ length: 9 }, (_, i) => {
  const r = Math.floor(i / 3);
  const c = i % 3;
  return (c + Math.abs(r - 1)) * 90;
});

/* orbit：彗星绕格外圈一周，中心格（index 4，不在外圈序列中）保持暗态 */
const ORBIT_ORDER = [0, 1, 2, 5, 8, 7, 6, 3];
const orbit = Array.from({ length: 9 }, (_, i) => {
  const k = ORBIT_ORDER.indexOf(i);
  return k === -1 ? null : k * 110;
});

const PATTERNS = {
  drive: { delays: chevron, dur: 650, round: false },
  dots: { delays: chevron, dur: 650, round: true },
  orbit: { delays: orbit, dur: 950, round: false },
} as const;

function formatElapsed(totalSeconds: number): string {
  if (totalSeconds < 60) return `${totalSeconds.toFixed(1)}s`;
  return `${Math.floor(totalSeconds / 60)}m ${(totalSeconds % 60).toFixed(1)}s`;
}

export function AiLoading({ label, variant = 'drive', startedAt }: AiLoadingProps) {
  const [start, setStart] = useState(() => startedAt ?? Date.now());
  const [now, setNow] = useState(() => Date.now());

  // 新一轮任务传入新 startedAt 时归零重计
  useEffect(() => {
    if (startedAt !== undefined) setStart(startedAt);
  }, [startedAt]);

  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 100);
    return () => clearInterval(t);
  }, []);

  const elapsed = formatElapsed(Math.max(0, (now - start) / 1000));
  const { delays, dur, round } = PATTERNS[variant];

  return (
    <div className="flex w-fit items-center gap-2.5" data-testid="ai-loading">
      <span aria-hidden className="grid grid-cols-[repeat(3,4px)] gap-[1.5px]">
        {delays.map((d, i) => (
          <span
            key={i}
            className={`size-[4px] bg-ai-ink ${round ? 'rounded-full' : 'rounded-[1px]'} ${
              d === null ? '' : 'animate-pixel-on'
            }`}
            style={{
              opacity: d === null ? 0.07 : 0.15,
              animationDelay: d === null ? undefined : `${d}ms`,
              animationDuration: `${dur}ms`,
            }}
          />
        ))}
      </span>
      <span
        className="animate-shimmer-text bg-clip-text text-[13px] font-medium text-transparent"
        style={{
          backgroundImage:
            'linear-gradient(90deg, var(--ai-ink-3) 35%, var(--ai-ink) 50%, var(--ai-ink-3) 65%)',
          backgroundSize: '200% 100%',
        }}
      >
        {label}
      </span>
      <span
        className="font-mono text-[12px] text-ai-ink-3 tabular-nums"
        data-testid="ai-loading-elapsed"
      >
        {elapsed}
      </span>
    </div>
  );
}

export default AiLoading;
```

**(a) `GenesisPanel.tsx`**：import 区（L19 `import { cn } from '@/utils/cn';` 行前）加：

```tsx
import { AiLoading } from '@/components/ui/ai/AiLoading';
```

L443-449 旧片段：

```tsx
              {/* Current Step Live Log */}
              {isCurrent && isRunning && progress && (
                <div className="px-2.5 pb-2.5 pt-0">
                  <div className="flex items-center gap-1.5 mb-1">
                    <Loader2 className="w-2.5 h-2.5 text-cinema-gold animate-spin" />
                    <span className="text-[10px] text-cinema-gold/70">{progress.message}</span>
                  </div>
```

替换为：

```tsx
              {/* Current Step Live Log */}
              {isCurrent && isRunning && progress && (
                <div className="px-2.5 pb-2.5 pt-0">
                  <div className="mb-1">
                    <AiLoading label={progress.message} variant="drive" />
                  </div>
```

（Loader2 import 保留：L193/L373/L534 仍在用。）

**(b) `GuidebookDistillationPanel.tsx`**：import 区（L33 `import { cn } from '@/utils/cn';` 行前）加：

```tsx
import { AiLoading } from '@/components/ui/ai/AiLoading';
```

L122-125 旧片段：

```tsx
          {ACTIVE_STATUSES.includes(status) && (
            <Loader2 className="w-4 h-4 text-cinema-gold animate-spin" />
          )}
          <span className="text-xs text-gray-500">{STATUS_LABELS[status] || status}</span>
```

替换为：

```tsx
          {ACTIVE_STATUSES.includes(status) ? (
            <AiLoading label={STATUS_LABELS[status] || status} variant="dots" />
          ) : (
            <span className="text-xs text-gray-500">{STATUS_LABELS[status] || status}</span>
          )}
```

L152-155 旧片段：

```tsx
          <div className="flex items-center justify-between text-xs text-gray-500 mb-1">
            <span>{currentStep || '正在提炼...'}</span>
            <span className="font-mono">{progress}%</span>
          </div>
```

替换为：

```tsx
          <div className="flex items-center justify-between text-xs text-gray-500 mb-1">
            <AiLoading label={currentStep || '正在提炼…'} variant="drive" />
            <span className="font-mono">{progress}%</span>
          </div>
```

（Loader2 import 保留：L479/L499/L591 仍在用。）

**(c) `NovelCreationWizard.tsx`**：import 区（L13 `import { Button } from '@/components/ui/Button';` 行前）加：

```tsx
import { AiLoading } from '@/components/ui/ai/AiLoading';
```

`renderGenerating`（L257-267）旧片段：

```tsx
  const renderGenerating = (message: string) => (
    <div className="text-center py-12">
      <div className="relative w-20 h-20 mx-auto mb-6">
        <div className="absolute inset-0 border-4 border-cinema-700 rounded-full" />
        <div className="absolute inset-0 border-4 border-cinema-gold rounded-full border-t-transparent animate-spin" />
        <Sparkles className="absolute inset-0 m-auto w-8 h-8 text-cinema-gold" />
      </div>
      <h3 className="text-xl font-semibold text-white mb-2">{message}</h3>
      <p className="text-gray-400">AI正在发挥创意...</p>
    </div>
  );
```

替换为：

```tsx
  const renderGenerating = (message: string) => (
    <div className="text-center py-12">
      <div className="flex justify-center mb-6">
        <AiLoading label={message} variant="orbit" />
      </div>
      <p className="text-gray-400">AI正在发挥创意...</p>
    </div>
  );
```

（Sparkles import 保留：genre 输入与策略确认按钮仍在用。）

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai src/components/guidebook-distillation src/utils/__tests__/genesisSteps.test.ts && npx tsc --noEmit`
Expected: AiLoading 4 passed；GuidebookDistillationPanel 既有测试不回归；tsc 干净

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiLoading.tsx src-frontend/src/components/ui/ai/__tests__/AiLoading.test.tsx src-frontend/src/components/GenesisPanel.tsx src-frontend/src/components/guidebook-distillation/GuidebookDistillationPanel.tsx src-frontend/src/components/NovelCreationWizard.tsx
git commit -m "feat: AiLoading 组件入库并替换幕后三处加载指示（P1 Task2）"
```

---

### Task 3: AiThinking 组件 + AgencyStudio「当前执行轨迹」

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiThinking.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiThinking.test.tsx`
- Modify: `src-frontend/src/pages/AgencyStudio.tsx`（import 区 L1-11；时间线 section L355-369 内嵌轨迹块）

**Interfaces:**
- Consumes: Task 1 的 `animate-shimmer-text` / `animate-ai-fade-up` / `animate-ai-spin` / `bg-ai-line` / `text-ai-green` / `text-ai-red` 等
- Produces:
  - `export interface AiThinkingRow { primary: string; secondary?: string; mono?: boolean; add?: number; del?: number; href?: string }`
  - `export interface AiThinkingProps { title: string; doneTitle?: string; working: boolean; rows: AiThinkingRow[]; defaultExpanded?: boolean }`
  - `export function AiThinking(props: AiThinkingProps): JSX.Element` — 行数据驱动，新增行 fade-up 交错入场（index 封顶 8 档 × 80ms）；标题按钮展开/收起（grid-template-rows 0fr/1fr）；working 时末行显示旋转圈、标题 shimmer；href 行渲染为 `<a target="_blank">` 带下划线；左侧竖线随内容高度生长。`data-testid="ai-thinking"` / `ai-thinking-trace` / `ai-thinking-spinner`
  - 剥离参考实现的 STAGES / useSequence / VARIANTS / Search 来源计数（+7 more）/ Coding 选中态

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiThinking.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiThinking } from '../AiThinking';

const rows = [
  { primary: '主创 生成 概念包', secondary: 'concept' },
  { primary: '管理 生成 世界观', mono: true },
];

describe('AiThinking', () => {
  it('working 时末行显示旋转指示，标题为传入 title', () => {
    render(<AiThinking title="当前执行轨迹" working={true} rows={rows} defaultExpanded />);
    expect(screen.getByText('当前执行轨迹')).toBeInTheDocument();
    expect(screen.getByTestId('ai-thinking-spinner')).toBeInTheDocument();
  });

  it('非 working 显示 doneTitle 且无 spinner', () => {
    render(
      <AiThinking
        title="当前执行轨迹"
        doneTitle="执行轨迹（已结束）"
        working={false}
        rows={rows}
        defaultExpanded
      />
    );
    expect(screen.getByText('执行轨迹（已结束）')).toBeInTheDocument();
    expect(screen.queryByTestId('ai-thinking-spinner')).not.toBeInTheDocument();
  });

  it('默认收起（0fr），点击标题展开（1fr）', () => {
    render(<AiThinking title="轨迹" working={false} rows={rows} />);
    const btn = screen.getByRole('button', { name: /轨迹/ });
    const trace = screen.getByTestId('ai-thinking-trace');
    expect(btn).toHaveAttribute('aria-expanded', 'false');
    expect(trace.style.gridTemplateRows).toBe('0fr');
    fireEvent.click(btn);
    expect(btn).toHaveAttribute('aria-expanded', 'true');
    expect(trace.style.gridTemplateRows).toBe('1fr');
  });

  it('href 行渲染为新窗口链接', () => {
    render(
      <AiThinking
        title="t"
        working={false}
        defaultExpanded
        rows={[{ primary: '设计文档', href: 'https://example.com/spec' }]}
      />
    );
    const link = screen.getByRole('link', { name: /设计文档/ });
    expect(link).toHaveAttribute('href', 'https://example.com/spec');
    expect(link).toHaveAttribute('target', '_blank');
  });

  it('add/del 行显示增删计数', () => {
    render(
      <AiThinking
        title="t"
        working={false}
        defaultExpanded
        rows={[{ primary: 'Edit', secondary: 'a.ts', mono: true, add: 74, del: 41 }]}
      />
    );
    expect(screen.getByText('+74')).toBeInTheDocument();
    expect(screen.getByText('−41')).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiThinking.test.tsx`
Expected: FAIL（模块 `../AiThinking` 不存在）

- [ ] **Step 3: Write implementation**

```tsx
// src-frontend/src/components/ui/ai/AiThinking.tsx
/**
 * AiThinking — 可展开的执行轨迹（适配自 beautifului ThinkingState）
 *
 * 受控组件：rows 由调用方数据驱动，无任何内部演示时序（STAGES/useSequence 已剥离）。
 * 标题按钮展开/收起（grid-template-rows 0fr/1fr）；行以 fade-up 交错入场；
 * 左侧竖线随内容高度生长；working=true 时标题 shimmer、末行显示旋转指示。
 */
import { useLayoutEffect, useRef, useState } from 'react';

export interface AiThinkingRow {
  primary: string;
  secondary?: string;
  mono?: boolean;
  add?: number;
  del?: number;
  href?: string;
}

export interface AiThinkingProps {
  title: string;
  doneTitle?: string;
  working: boolean;
  rows: AiThinkingRow[];
  defaultExpanded?: boolean;
}

export function AiThinking({
  title,
  doneTitle,
  working,
  rows,
  defaultExpanded = false,
}: AiThinkingProps) {
  const [manualExpanded, setManualExpanded] = useState<boolean | null>(null);
  const expanded = manualExpanded ?? defaultExpanded;
  const traceRef = useRef<HTMLDivElement>(null);
  const [lineHeight, setLineHeight] = useState(0);

  // 竖线随内容高度生长（行数增减/展开收起后重测）
  useLayoutEffect(() => {
    if (traceRef.current) setLineHeight(traceRef.current.offsetHeight);
  }, [rows.length, expanded]);

  return (
    <div className="flex w-full flex-col" data-testid="ai-thinking">
      {/* header */}
      <button
        type="button"
        aria-expanded={expanded}
        onClick={() => setManualExpanded(!expanded)}
        className="-mx-1.5 flex w-fit items-center gap-2 rounded px-1.5 py-1 transition-colors duration-100 hover:bg-ai-hover-2"
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill={working ? 'var(--ai-ink-2)' : 'var(--ai-ink-3)'}
          aria-hidden
        >
          <path d="M12 2l2.4 7.2L22 12l-7.6 2.8L12 22l-2.4-7.2L2 12l7.6-2.8z" />
        </svg>
        {working ? (
          <span
            className="animate-shimmer-text bg-clip-text text-[13px] font-medium whitespace-nowrap text-transparent"
            style={{
              backgroundImage:
                'linear-gradient(90deg, var(--ai-ink-3) 35%, var(--ai-ink) 50%, var(--ai-ink-3) 65%)',
              backgroundSize: '200% 100%',
            }}
          >
            {title}
          </span>
        ) : (
          <span className="text-[13px] font-medium whitespace-nowrap text-ai-ink-2">
            {doneTitle ?? title}
          </span>
        )}
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="var(--ai-ink-3)"
          strokeWidth="2.2"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="transition-transform duration-300"
          style={{ transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)' }}
          aria-hidden
        >
          <path d="M6 9l6 6 6-6" />
        </svg>
      </button>

      {/* expandable trace */}
      <div
        data-testid="ai-thinking-trace"
        className="grid transition-[grid-template-rows,opacity] duration-300"
        style={{
          gridTemplateRows: expanded ? '1fr' : '0fr',
          opacity: expanded ? 1 : 0,
          transitionTimingFunction: 'cubic-bezier(0.23, 1, 0.32, 1)',
        }}
      >
        <div className="overflow-hidden">
          <div className="relative mt-1 ml-[5px] pl-4">
            <span
              aria-hidden
              className="absolute left-[3px] w-px bg-ai-line"
              style={{
                top: -8,
                height: lineHeight ? lineHeight - 2 : 0,
                transition: 'height 500ms cubic-bezier(0.23,1,0.32,1)',
              }}
            />
            <div ref={traceRef} className="flex flex-col gap-1 py-1">
              {rows.map((row, i) => {
                const isLast = i === rows.length - 1;
                const content = (
                  <>
                    {working && isLast ? (
                      <span
                        data-testid="ai-thinking-spinner"
                        className="animate-ai-spin size-3 shrink-0 rounded-full border-[1.5px] border-ai-line-strong border-t-ai-ink-2"
                      />
                    ) : (
                      <svg
                        width="14"
                        height="14"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="var(--ai-ink-3)"
                        strokeWidth="2.5"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        className="shrink-0"
                        aria-hidden
                      >
                        <path d="M20 6L9 17l-5-5" />
                      </svg>
                    )}
                    <span
                      className={`min-w-0 truncate text-[12.5px] font-medium text-ai-ink ${
                        row.href
                          ? 'underline decoration-ai-line-strong underline-offset-2'
                          : ''
                      }`}
                    >
                      {row.primary}
                    </span>
                    {row.secondary && (
                      <span
                        className={`shrink-0 text-[11.5px] text-ai-ink-3 ${
                          row.mono ? 'font-mono' : ''
                        }`}
                      >
                        {row.secondary}
                      </span>
                    )}
                    {row.add !== undefined && (
                      <span className="shrink-0 font-mono text-[11px] tabular-nums">
                        <span className="text-ai-green">+{row.add}</span>{' '}
                        <span className="text-ai-red">−{row.del ?? 0}</span>
                      </span>
                    )}
                  </>
                );
                const rowClass =
                  'animate-ai-fade-up flex min-h-7 w-full items-center gap-2 rounded-[6px] px-1.5 py-0.5 text-left';
                // 交错入场：index 封顶 8 档，避免长列表尾部行延迟过大
                const style = { animationDelay: `${Math.min(i, 8) * 80}ms` };

                if (row.href) {
                  return (
                    <a
                      key={`${row.primary}-${i}`}
                      href={row.href}
                      target="_blank"
                      rel="noreferrer"
                      className={`${rowClass} transition-colors duration-150 hover:bg-ai-hover`}
                      style={style}
                    >
                      {content}
                    </a>
                  );
                }
                return (
                  <div key={`${row.primary}-${i}`} className={rowClass} style={style}>
                    {content}
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default AiThinking;
```

**`AgencyStudio.tsx`**：import 区（L10 `import { getRun, listActivities, listBoard, listRuns } from '@/services/api/agency';` 行前）加：

```tsx
import { AiThinking } from '@/components/ui/ai/AiThinking';
```

时间线 section（L355-369）旧片段：

```tsx
      <section>
        <h2 className="mb-2 font-medium">时间线</h2>
        {timeline.length === 0 ? (
          <p className="text-sm text-gray-400">暂无记录</p>
        ) : (
```

替换为（AiThinking 轨迹块在上，既有时间线保留为历史记录）：

```tsx
      <section>
        <h2 className="mb-2 font-medium">时间线</h2>
        {runActivities.length > 0 && (
          <div className="mb-3">
            <AiThinking
              title="当前执行轨迹"
              doneTitle="执行轨迹（已结束）"
              working={run?.status === 'running'}
              rows={runActivities.slice(-12).map(a => ({
                primary: `${roleName(a.role)} ${a.action}`,
                secondary: a.detail || undefined,
              }))}
              defaultExpanded={run?.status === 'running'}
            />
          </div>
        )}
        {timeline.length === 0 ? (
          <p className="text-sm text-gray-400">暂无记录</p>
        ) : (
```

（`runActivities` 为 DB 主源 + live 补充、按时间正序，`slice(-12)` 取最近 12 条；`roleName` / `run` 均为本文件既有。）

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiThinking.test.tsx src/pages/__tests__/AgencyStudio.test.tsx && npx tsc --noEmit`
Expected: AiThinking 5 passed；AgencyStudio 既有测试不回归；tsc 干净

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiThinking.tsx src-frontend/src/components/ui/ai/__tests__/AiThinking.test.tsx src-frontend/src/pages/AgencyStudio.tsx
git commit -m "feat: AiThinking 组件入库并接入代理工作室执行轨迹（P1 Task3）"
```

---

### Task 4: AiStreamingText 组件 + 幕前幽灵续写渲染 + 删除旧 StreamingText 死代码

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiStreamingText.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiStreamingText.test.tsx`
- Modify: `src-frontend/src/frontstage/components/RichTextEditor.tsx`（幽灵段落 L1368-1378；import 区）
- Delete: `src-frontend/src/frontstage/components/StreamingText.tsx`、`src-frontend/src/frontstage/hooks/useStreamingGeneration.ts`（死代码）
- Modify: `src-frontend/src/frontstage/components/index.ts`（移除 L2 导出）

**Interfaces:**
- Consumes: Task 1 的 `animate-stream-in` / `animate-ai-blink` / `bg-ai-ink`；RichTextEditor 既有 props `generatedText?: string`、`isGenerating?: boolean`
- Produces:
  - `export interface AiStreamingTextProps { text: string; done: boolean; className?: string }`
  - `export function segmentStreamText(text: string): string[]` — `Intl.Segmenter('zh', { granularity: 'word' })` 词级切分，环境不支持时逐字符回退（导出供单测）
  - `export function AiStreamingText(props): JSX.Element` — 新到达单位以 stream-in 模糊入场（稳定 key 复用旧节点不重播）；`done=false` 时末尾闪烁光标（`data-testid="ai-streaming-cursor"`）；文本重置（新流）时整体重新入场
  - **未来工作（注释标注）**：行内引用 citations / sources 面板 / follow-ups 操作区——本应用无数据源，P1 不适配

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiStreamingText.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AiStreamingText, segmentStreamText } from '../AiStreamingText';

describe('segmentStreamText', () => {
  it('中文按词切分（非逐字）', () => {
    const tokens = segmentStreamText('他走进雨里');
    expect(tokens.join('')).toBe('他走进雨里');
    expect(tokens.length).toBeGreaterThan(1);
    expect(tokens.length).toBeLessThan(5); // 词级切分，不会拆成 5 个单字
  });

  it('空串返回空数组', () => {
    expect(segmentStreamText('')).toEqual([]);
  });
});

describe('AiStreamingText', () => {
  it('渲染全部已到达文本，未完成时显示闪烁光标', () => {
    render(<AiStreamingText text="他走进雨里" done={false} />);
    expect(screen.getByTestId('ai-streaming-text').textContent).toBe('他走进雨里');
    expect(screen.getByTestId('ai-streaming-cursor')).toBeInTheDocument();
  });

  it('done 后光标消失，文本完整', () => {
    const { rerender } = render(<AiStreamingText text="你好" done={false} />);
    rerender(<AiStreamingText text="你好世界" done={true} />);
    expect(screen.getByTestId('ai-streaming-text').textContent).toBe('你好世界');
    expect(screen.queryByTestId('ai-streaming-cursor')).not.toBeInTheDocument();
  });

  it('增量到达时旧 token 节点复用（动画不重播），新 token 追加', () => {
    const { rerender } = render(<AiStreamingText text="你好" done={false} />);
    const first = screen.getByTestId('ai-streaming-text').querySelector('span');
    expect(first).not.toBeNull();
    rerender(<AiStreamingText text="你好世界" done={false} />);
    const spans = screen.getByTestId('ai-streaming-text').querySelectorAll('span');
    expect(spans[0]).toBe(first); // 同一 DOM 节点 → CSS 动画不重播
    expect(spans.length).toBeGreaterThan(1);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiStreamingText.test.tsx`
Expected: FAIL（模块 `../AiStreamingText` 不存在）

- [ ] **Step 3: Write implementation**

```tsx
// src-frontend/src/components/ui/ai/AiStreamingText.tsx
/**
 * AiStreamingText — 流式文字渲染（适配自 beautifului StreamingText）
 *
 * 受控组件：text 为截至目前已到达的完整文本（调用方逐步增长），
 * 每个新到达的单位以 stream-in 模糊入场动画出现（稳定 key → 旧节点不重播）；
 * done=false 时末尾显示闪烁光标。
 *
 * 中文感知分词：Intl.Segmenter('zh', { granularity: 'word' })，不支持时逐字回退。
 *
 * 未来工作（本应用暂无数据源，P1 未适配）：行内引用 citations、sources 面板、
 * follow-ups 操作区——见 .superpowers/sdd/reference/beautifului/StreamingText.tsx。
 */
import { useRef } from 'react';

export interface AiStreamingTextProps {
  text: string;
  done: boolean;
  className?: string;
}

interface SegmenterLike {
  segment(input: string): Iterable<{ segment: string }>;
}

/** 中文按词切分；无 Intl.Segmenter 的环境回退逐字符（保证多字节字符不被字节级切开） */
export function segmentStreamText(text: string): string[] {
  const Seg = (
    Intl as unknown as {
      Segmenter?: new (locale: string, opts: { granularity: 'word' }) => SegmenterLike;
    }
  ).Segmenter;
  if (Seg) {
    return Array.from(new Seg('zh', { granularity: 'word' }).segment(text), s => s.segment);
  }
  return Array.from(text);
}

export function AiStreamingText({ text, done, className }: AiStreamingTextProps) {
  // 流重置检测：text 不再以既有文本为前缀（新一轮生成）时 epoch +1，
  // key 变化强制全部 token 重新入场
  const epochRef = useRef(0);
  const prevTextRef = useRef('');
  if (prevTextRef.current && !text.startsWith(prevTextRef.current)) {
    epochRef.current += 1;
  }
  prevTextRef.current = text;

  const tokens = segmentStreamText(text);

  return (
    <span className={className} data-testid="ai-streaming-text">
      {tokens.map((token, i) => (
        <span
          key={`${epochRef.current}:${i}`}
          className="animate-stream-in inline [will-change:filter,opacity]"
        >
          {token}
        </span>
      ))}
      {!done && (
        <span
          aria-hidden
          data-testid="ai-streaming-cursor"
          className="animate-ai-blink ml-0.5 inline-block h-3 w-0.5 translate-y-0.5 rounded-full bg-ai-ink"
        />
      )}
    </span>
  );
}

export default AiStreamingText;
```

**(a) `RichTextEditor.tsx`**：import 区（L84 `import { EditorContextMenu } from './EditorContextMenu';` 行后）加：

```tsx
import { AiStreamingText } from '@/components/ui/ai/AiStreamingText';
```

幽灵段落（L1370-1378）旧片段：

```tsx
              {shouldShowGhostParagraph && (
                <p
                  className="ghost-paragraph"
                  data-testid="ghost-paragraph"
                  style={{ userSelect: 'none' }}
                >
                  {generatedText}
                </p>
              )}
```

替换为：

```tsx
              {shouldShowGhostParagraph && (
                <p
                  className="ghost-paragraph"
                  data-testid="ghost-paragraph"
                  style={{ userSelect: 'none' }}
                >
                  <AiStreamingText text={generatedText} done={!isGenerating} />
                </p>
              )}
```

（`generatedText` / `isGenerating` 均为本组件既有 props——L94-95。不改 FrontstageApp.tsx 的打字机/race-lock 逻辑；`.editor-ghost-continuation` / `.ghost-paragraph` CSS 不变，textContent 型断言不受影响。）

**(b) 删除死代码**。先确认零消费者：

Run: `cd src-frontend && grep -rn "frontstage/components/StreamingText\|from './StreamingText'\|from '../hooks/useStreamingGeneration'\|useStreamingGeneration" src --include='*.ts*' | grep -v "frontstage/components/StreamingText.tsx" | grep -v "frontstage/hooks/useStreamingGeneration.ts"`
Expected: 仅剩 `src/frontstage/components/index.ts:2:export { StreamingText } from './StreamingText';` 一行。若有其他输出则停止本步骤，报告引用点。

然后：
1. 删除 `src-frontend/src/frontstage/components/StreamingText.tsx`
2. 删除 `src-frontend/src/frontstage/hooks/useStreamingGeneration.ts`
3. `src-frontend/src/frontstage/components/index.ts` 旧内容：

```ts
// Frontstage Components Export
export { StreamingText } from './StreamingText';
export { SmartHintSystem } from '../ai-perception';
export { AiHintOverlay } from './AiHintOverlay';
export { default as RichTextEditor } from './RichTextEditor';
export { ChapterOutline } from './ChapterOutline';
export { CharacterCardPopup } from './CharacterCardPopup';
```

新内容（仅删第 2 行）：

```ts
// Frontstage Components Export
export { SmartHintSystem } from '../ai-perception';
export { AiHintOverlay } from './AiHintOverlay';
export { default as RichTextEditor } from './RichTextEditor';
export { ChapterOutline } from './ChapterOutline';
export { CharacterCardPopup } from './CharacterCardPopup';
```

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiStreamingText.test.tsx src/frontstage/components/__tests__ src/frontstage/__tests__ && npx tsc --noEmit`
Expected: AiStreamingText 5 passed；RichTextEditor / FrontstageApp 全部既有测试不回归；tsc 干净（StreamingText 删除后无悬空引用）

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiStreamingText.tsx src-frontend/src/components/ui/ai/__tests__/AiStreamingText.test.tsx src-frontend/src/frontstage/components/RichTextEditor.tsx src-frontend/src/frontstage/components/index.ts src-frontend/src/frontstage/components/StreamingText.tsx src-frontend/src/frontstage/hooks/useStreamingGeneration.ts
git commit -m "feat: AiStreamingText 组件入库接入幕前幽灵续写并删除旧流式死代码（P1 Task4）"
```

---

### Task 5: AiPromptBar 组件 + FrontstageBottomBar 指令输入条替换

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiPromptBar.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiPromptBar.test.tsx`
- Modify: `src-frontend/src/frontstage/components/FrontstageBottomBar.tsx`（import 区 L1-19；textarea 自适应 effect L116-129 删除；输入框区 L382-431）

**Interfaces:**
- Consumes: Task 1 的 `bg-ai-surface` / `border-ai-line(-strong)` / `bg-ai-hover(-2)` / `text-ai-ink(-2/-3)` / `animate-ai-sweep`；既有 `lucide-react` 图标（Plus / ArrowUp / ChevronDown / Check）
- Produces:
  - `export interface AiPromptSource { key: string; name: string; desc: string }`
  - `export interface AiPromptCommand { key: string; name: string; desc: string }`（name 含 `/` 前缀，如 `/自动续写`）
  - `export interface AiPromptModel { key: string; name: string; tag?: string }`
  - `export interface AiPromptBarProps`：

```typescript
export interface AiPromptBarProps {
  value: string;
  onChange: (v: string) => void;
  onSend: () => void;
  placeholder?: string; // 默认 '输入任意指令…'
  disabled?: boolean;
  sources?: AiPromptSource[]; // 缺省/空 → @ 菜单与 + 按钮均不渲染
  commands?: AiPromptCommand[]; // 缺省/空 → / 菜单不打开
  models?: AiPromptModel[]; // 缺省 → 模型选择器整体不渲染（P1 幕前不传，P2 再接）
  model?: string;
  onModelChange?: (key: string) => void; // 切换到不同模型时同步触发 ai-sweep 扫光
  /** 菜单关闭时透传底层 textarea keydown（父级幽灵提示 ↑↓/→/Esc 等逻辑）；父级 preventDefault 的键不再走内部发送 */
  onKeyDown?: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
  /** 透传底层 textarea focus */
  onFocus?: () => void;
  /** 替换发送按钮的自定义动作（如生成中的「取消生成」按钮） */
  trailingAction?: React.ReactNode;
}
```

  - `export function AiPromptBar(props: AiPromptBarProps): JSX.Element`
  - 相对设计稿 API 的三个**增补**（`onKeyDown` / `onFocus` / `trailingAction`）：BottomBar 既有行为（幽灵提示历史导航、focus 回调、生成中取消按钮）需要透传，均为可选，不传即为纯净受控输入条
  - 相对参考实现的**删减**：AUTO_STEPS 自动演示、听写按钮（eq-bounce 保留在令牌层）、品牌 SVG、glimm canvas（扫光改为 `ai-sweep` CSS 渐变覆盖层）、附件行（无 attachments prop，整体不落）、inline/expanded 布局重排（textarea 恒占主列，自动增高 28–160px）

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiPromptBar.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiPromptBar } from '../AiPromptBar';

const sources = [
  { key: 'char-lin', name: '林晚', desc: '角色' },
  { key: 'world-canglan', name: '苍澜大陆', desc: '世界观' },
];
const commands = [
  { key: 'auto_write', name: '/自动续写', desc: '从当前位置自动续写' },
  { key: 'auto_revise', name: '/审校', desc: '审校当前章节' },
];
const models = [
  { key: 'a', name: 'Model A', tag: 'Flagship' },
  { key: 'b', name: 'Model B' },
];

describe('AiPromptBar', () => {
  it('受控渲染：显示 value，输入触发 onChange', () => {
    const onChange = vi.fn();
    render(<AiPromptBar value="你好" onChange={onChange} onSend={() => {}} />);
    const ta = screen.getByPlaceholderText('输入任意指令…');
    expect(ta).toHaveValue('你好');
    fireEvent.change(ta, { target: { value: '你好！' } });
    expect(onChange).toHaveBeenCalledWith('你好！');
  });

  it('空输入发送禁用；有内容点击发送触发 onSend', () => {
    const onSend = vi.fn();
    const { rerender } = render(<AiPromptBar value="" onChange={() => {}} onSend={onSend} />);
    expect(screen.getByTitle('发送')).toBeDisabled();
    rerender(<AiPromptBar value="写一段" onChange={() => {}} onSend={onSend} />);
    fireEvent.click(screen.getByTitle('发送'));
    expect(onSend).toHaveBeenCalledTimes(1);
  });

  it('Enter 发送；IME 组合输入中 Enter 不发送', () => {
    const onSend = vi.fn();
    render(<AiPromptBar value="写一段" onChange={() => {}} onSend={onSend} />);
    const ta = screen.getByPlaceholderText('输入任意指令…');
    fireEvent.keyDown(ta, { key: 'Enter' });
    expect(onSend).toHaveBeenCalledTimes(1);
    // 中文输入法组合中（isComposing=true）Enter 仅上屏，不触发发送
    const composing = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true });
    Object.defineProperty(composing, 'isComposing', { value: true });
    ta.dispatchEvent(composing);
    expect(onSend).toHaveBeenCalledTimes(1);
  });

  it('输入 / 打开命令菜单，↓+Enter 选中插入命令文本（去掉 / 前缀）', () => {
    const onChange = vi.fn();
    render(<AiPromptBar value="/" onChange={onChange} onSend={() => {}} commands={commands} />);
    const ta = screen.getByPlaceholderText('输入任意指令…');
    expect(screen.getByTestId('ai-prompt-menu')).toBeInTheDocument();
    fireEvent.keyDown(ta, { key: 'ArrowDown' });
    fireEvent.keyDown(ta, { key: 'Enter' });
    expect(onChange).toHaveBeenCalledWith('审校 ');
  });

  it('输入 @ 且传入 sources 时打开数据源菜单；无 sources 不打开', () => {
    const { rerender } = render(<AiPromptBar value="@" onChange={() => {}} onSend={() => {}} />);
    expect(screen.queryByTestId('ai-prompt-menu')).not.toBeInTheDocument();
    rerender(
      <AiPromptBar value="@" onChange={() => {}} onSend={() => {}} sources={sources} />
    );
    expect(screen.getByTestId('ai-prompt-menu')).toBeInTheDocument();
    expect(screen.getByText('林晚')).toBeInTheDocument();
  });

  it('Esc 关闭菜单；菜单关闭后 onKeyDown 透传父级', () => {
    const onKeyDown = vi.fn();
    render(
      <AiPromptBar
        value="/"
        onChange={() => {}}
        onSend={() => {}}
        commands={commands}
        onKeyDown={onKeyDown}
      />
    );
    const ta = screen.getByPlaceholderText('输入任意指令…');
    fireEvent.keyDown(ta, { key: 'Escape' }); // 第一次：仅关菜单，不透传
    expect(screen.queryByTestId('ai-prompt-menu')).not.toBeInTheDocument();
    expect(onKeyDown).not.toHaveBeenCalled();
    fireEvent.keyDown(ta, { key: 'ArrowUp' }); // 菜单已关：透传
    expect(onKeyDown).toHaveBeenCalledTimes(1);
  });

  it('models 缺省时不渲染模型选择器；传入后切换模型触发 onModelChange 与扫光', () => {
    const onModelChange = vi.fn();
    const { rerender } = render(<AiPromptBar value="" onChange={() => {}} onSend={() => {}} />);
    expect(screen.queryByLabelText('选择模型')).not.toBeInTheDocument();
    rerender(
      <AiPromptBar
        value=""
        onChange={() => {}}
        onSend={() => {}}
        models={models}
        model="a"
        onModelChange={onModelChange}
      />
    );
    fireEvent.click(screen.getByLabelText('选择模型'));
    fireEvent.click(screen.getByText('Model B'));
    expect(onModelChange).toHaveBeenCalledWith('b');
    expect(screen.getByTestId('ai-sweep-overlay')).toBeInTheDocument();
  });

  it('trailingAction 传入时替换发送按钮', () => {
    render(
      <AiPromptBar
        value="x"
        onChange={() => {}}
        onSend={() => {}}
        trailingAction={<button title="取消生成">x</button>}
      />
    );
    expect(screen.getByTitle('取消生成')).toBeInTheDocument();
    expect(screen.queryByTitle('发送')).not.toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiPromptBar.test.tsx`
Expected: FAIL（模块 `../AiPromptBar` 不存在）

- [ ] **Step 3: Write implementation**

```tsx
// src-frontend/src/components/ui/ai/AiPromptBar.tsx
/**
 * AiPromptBar — AI 指令输入条（适配自 beautifului PromptBar）
 *
 * 受控组件：value/onChange/onSend 由调用方持有。
 * 保留：自动增高 textarea；@ 数据源与 / 命令菜单（滑动高亮 + ↑↓/Enter/Tab/Esc
 * 键盘导航 + IME isComposing 守卫）；模型选择菜单（传入 models 才渲染，
 * 切换模型触发 ai-sweep 一次性渐变扫光——glimm canvas 的纯 CSS 替代）。
 * 移除：AUTO_STEPS 自动演示、听写、品牌 SVG、附件（无 attachments prop）、
 * inline/expanded 布局重排。
 */
import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { ArrowUp, Check, ChevronDown, Plus } from 'lucide-react';

export interface AiPromptSource {
  key: string;
  name: string;
  desc: string;
}

export interface AiPromptCommand {
  key: string;
  name: string; // 含 / 前缀，如 /自动续写
  desc: string;
}

export interface AiPromptModel {
  key: string;
  name: string;
  tag?: string;
}

export interface AiPromptBarProps {
  value: string;
  onChange: (v: string) => void;
  onSend: () => void;
  placeholder?: string;
  disabled?: boolean;
  sources?: AiPromptSource[];
  commands?: AiPromptCommand[];
  models?: AiPromptModel[];
  model?: string;
  onModelChange?: (key: string) => void;
  onKeyDown?: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
  onFocus?: () => void;
  trailingAction?: React.ReactNode;
}

/* 正在输入的最后一个 @词 或 /词（\w 之外补 CJK 区间，中文数据源名可续打筛选） */
function parseToken(draft: string): { kind: 'at' | 'slash'; query: string; start: number } | null {
  const match = /(^|\s)([@/])([\w一-鿿-]*)$/.exec(draft);
  if (!match) return null;
  return {
    kind: match[2] === '@' ? 'at' : 'slash',
    query: match[3].toLowerCase(),
    start: match.index + match[1].length,
  };
}

export function AiPromptBar({
  value,
  onChange,
  onSend,
  placeholder = '输入任意指令…',
  disabled = false,
  sources,
  commands,
  models,
  model,
  onModelChange,
  onKeyDown,
  onFocus,
  trailingAction,
}: AiPromptBarProps) {
  const [dismissed, setDismissed] = useState(false);
  const [plusOpen, setPlusOpen] = useState(false);
  const [modelOpen, setModelOpen] = useState(false);
  const [active, setActive] = useState(0);
  const [engaged, setEngaged] = useState(false);
  const [rowBox, setRowBox] = useState<{ top: number; height: number } | null>(null);
  const [modelHovered, setModelHovered] = useState<number | null>(null);
  const [modelBox, setModelBox] = useState<{ top: number; height: number } | null>(null);
  const [sweeping, setSweeping] = useState(false);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const rowRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const modelRowRefs = useRef<(HTMLButtonElement | null)[]>([]);

  const hasSources = (sources?.length ?? 0) > 0;
  const hasCommands = (commands?.length ?? 0) > 0;
  const hasModels = (models?.length ?? 0) > 0;

  const token = dismissed ? null : parseToken(value);
  const menu: 'at' | 'slash' | null = plusOpen
    ? 'at'
    : token?.kind === 'at' && hasSources
      ? 'at'
      : token?.kind === 'slash' && hasCommands
        ? 'slash'
        : null;
  const query = plusOpen ? '' : (token?.query ?? '');

  const rows: { key: string; name: string; desc: string }[] =
    menu === 'at'
      ? (sources ?? []).filter(s => s.name.toLowerCase().includes(query))
      : menu === 'slash'
        ? (commands ?? []).filter(c => c.name.slice(1).toLowerCase().includes(query))
        : [];

  useEffect(() => {
    setActive(0);
    setEngaged(false);
  }, [menu, query]);

  /* 单一滑动高亮块（gliding highlight）跟随 active 行，而非每行各自切换背景 */
  useLayoutEffect(() => {
    const target = rowRefs.current[active];
    if (target) setRowBox({ top: target.offsetTop, height: target.offsetHeight });
  }, [menu, query, active, rows.length]);

  const modelIndex = (models ?? []).findIndex(m => m.key === model);
  useLayoutEffect(() => {
    if (!modelOpen) return;
    const target = modelRowRefs.current[modelHovered ?? modelIndex];
    if (target) setModelBox({ top: target.offsetTop, height: target.offsetHeight });
  }, [modelOpen, modelHovered, modelIndex]);

  useEffect(() => {
    if (!modelOpen) setModelHovered(null);
  }, [modelOpen]);

  /* 自动增高：28px 起，160px 封顶 */
  useLayoutEffect(() => {
    const input = inputRef.current;
    if (!input) return;
    input.style.height = '0px';
    const contentHeight = input.scrollHeight;
    input.style.height = `${Math.min(Math.max(contentHeight, 28), 160)}px`;
    input.style.overflowY = contentHeight > 160 ? 'auto' : 'hidden';
  }, [value]);

  /* ai-sweep 一次性扫光（模型切换时）。jsdom 无 matchMedia，可选链兜底 */
  const fireSweep = () => {
    if (window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) return;
    setSweeping(true);
  };

  const selectModel = (key: string) => {
    onModelChange?.(key);
    setModelOpen(false);
    if (key !== model) fireSweep();
    inputRef.current?.focus();
  };

  /* 选中菜单项：@源 插入 @名字；/命令 插入命令名（去 / 前缀，与直接在输入框
   * 打「自动续写」等指令的既有路由一致——统一由后端意图识别处理） */
  const pick = (row: { key: string; name: string }) => {
    const base = token ? value.slice(0, token.start) : value;
    onChange(menu === 'at' ? `${base}@${row.name} ` : `${base}${row.name.replace(/^\//, '')} `);
    setPlusOpen(false);
    setDismissed(false);
    inputRef.current?.focus();
  };

  const canSend = value.trim().length > 0 && !disabled;
  const send = () => {
    if (!canSend) return;
    onSend();
    setPlusOpen(false);
    setModelOpen(false);
    setDismissed(false);
  };

  const currentModel = (models ?? []).find(m => m.key === model);

  return (
    <div className="relative" data-testid="ai-prompt-bar">
      {/* ── @ / 命令菜单（从输入条上沿向上生长） ── */}
      {menu && (
        <div
          data-testid="ai-prompt-menu"
          onMouseLeave={() => setEngaged(false)}
          className="animate-pop-in absolute inset-x-0 bottom-full z-10 mb-2 rounded-[10px] border border-ai-line bg-ai-surface p-1 shadow-float"
          style={{ transformOrigin: 'bottom center' }}
        >
          <span
            aria-hidden
            className="pointer-events-none absolute inset-x-1 rounded-[6px] bg-ai-hover"
            style={{
              top: rowBox?.top ?? 0,
              height: rowBox?.height ?? 0,
              opacity: rowBox && engaged && rows.length > 0 ? 1 : 0,
              transition:
                'top 220ms cubic-bezier(0.23,1,0.32,1), height 220ms cubic-bezier(0.23,1,0.32,1), opacity 150ms ease',
            }}
          />
          {rows.map((row, i) => (
            <button
              key={row.key}
              type="button"
              ref={el => {
                rowRefs.current[i] = el;
              }}
              onMouseDown={e => e.preventDefault()}
              onMouseEnter={() => {
                setActive(i);
                setEngaged(true);
              }}
              onClick={() => pick(row)}
              className="relative z-10 flex h-9 w-full items-center gap-2.5 rounded-[6px] px-2 text-left"
              data-testid={`ai-prompt-menu-row-${row.key}`}
            >
              <span className="shrink-0 text-[12.5px] font-medium text-ai-ink">{row.name}</span>
              <span className="min-w-0 flex-1 truncate text-[12px] text-ai-ink-3">{row.desc}</span>
            </button>
          ))}
          {rows.length === 0 && (
            <div className="flex h-9 items-center px-2 text-[12px] text-ai-ink-3">
              无匹配「{query}」
            </div>
          )}
          <div className="mt-1 border-t border-ai-line px-2 pt-1.5 pb-1 text-[11px] text-ai-ink-3">
            {menu === 'at' ? '输入以筛选数据源' : '输入以筛选命令'}
          </div>
        </div>
      )}

      {/* ── 模型菜单 ── */}
      {modelOpen && hasModels && (
        <div
          data-testid="ai-model-menu"
          onMouseLeave={() => setModelHovered(null)}
          className="animate-pop-in absolute right-0 bottom-full z-10 mb-2 w-44 rounded-[10px] border border-ai-line bg-ai-surface p-1 shadow-float"
          style={{ transformOrigin: 'bottom right' }}
        >
          <span
            aria-hidden
            className="pointer-events-none absolute inset-x-1 rounded-[6px] bg-ai-hover"
            style={{
              top: modelBox?.top ?? 0,
              height: modelBox?.height ?? 0,
              opacity: modelBox && modelHovered !== null ? 1 : 0,
              transition:
                'top 220ms cubic-bezier(0.23,1,0.32,1), height 220ms cubic-bezier(0.23,1,0.32,1), opacity 150ms ease',
            }}
          />
          {(models ?? []).map((m, i) => (
            <button
              key={m.key}
              type="button"
              ref={el => {
                modelRowRefs.current[i] = el;
              }}
              onMouseDown={e => e.preventDefault()}
              onMouseEnter={() => setModelHovered(i)}
              onClick={() => selectModel(m.key)}
              className="relative z-10 flex h-8 w-full items-center gap-2 rounded-[6px] px-2 text-left"
            >
              <span className="min-w-0 flex-1 truncate text-[12.5px] font-medium text-ai-ink">
                {m.name}
              </span>
              {m.tag && <span className="shrink-0 text-[11px] text-ai-ink-3">{m.tag}</span>}
              <span className={`shrink-0 text-ai-ink ${m.key === model ? '' : 'invisible'}`}>
                <Check className="size-3.5" />
              </span>
            </button>
          ))}
        </div>
      )}

      {/* ── 输入条本体 ── */}
      <div className="relative isolate flex items-end gap-1 overflow-hidden rounded-[10px] border border-ai-line bg-ai-surface p-1.5 transition-colors duration-150 focus-within:border-ai-line-strong">
        {/* ai-sweep 扫光覆盖层（模型切换时播放一次；950ms 与 keyframes 同步） */}
        {sweeping && (
          <span
            aria-hidden
            data-testid="ai-sweep-overlay"
            onAnimationEnd={() => setSweeping(false)}
            className="animate-ai-sweep pointer-events-none absolute inset-y-0 left-0 -z-10 w-1/2"
            style={{
              background:
                'linear-gradient(105deg, transparent 0%, var(--ai-accent-tint) 30%, var(--ai-accent) 50%, var(--ai-accent-tint) 70%, transparent 100%)',
            }}
          />
        )}

        {hasSources && (
          <button
            type="button"
            aria-label="添加数据源"
            aria-expanded={plusOpen}
            onClick={() => {
              setModelOpen(false);
              setPlusOpen(c => !c);
              inputRef.current?.focus();
            }}
            className={`flex size-7 shrink-0 items-center justify-center rounded-[8px] transition-colors duration-150 hover:bg-ai-hover hover:text-ai-ink ${
              plusOpen ? 'bg-ai-hover text-ai-ink' : 'text-ai-ink-3'
            }`}
          >
            <Plus className="size-4" />
          </button>
        )}

        <textarea
          ref={inputRef}
          rows={1}
          value={value}
          onChange={e => {
            onChange(e.target.value);
            setDismissed(false);
            setPlusOpen(false);
          }}
          onKeyDown={e => {
            // 1) 菜单打开时的键盘导航（↑↓ 移动 / Enter·Tab 选中）
            if (menu && rows.length > 0) {
              if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
                e.preventDefault();
                setEngaged(true);
                setActive(
                  c => (c + (e.key === 'ArrowDown' ? 1 : rows.length - 1)) % rows.length
                );
                return;
              }
              if ((e.key === 'Enter' && !e.shiftKey) || e.key === 'Tab') {
                e.preventDefault();
                pick(rows[active]);
                return;
              }
            }
            // 2) Esc：菜单/模型菜单打开时仅关闭，不透传父级
            if (e.key === 'Escape' && (menu || modelOpen)) {
              setDismissed(true);
              setPlusOpen(false);
              setModelOpen(false);
              return;
            }
            // 3) 透传父级（幽灵提示 ↑↓/→/Esc 等）；父级 preventDefault 的键不再走内部发送
            onKeyDown?.(e);
            if (e.defaultPrevented) return;
            // 4) Enter 发送（IME 组合输入中除外——中文输入法上屏 Enter 不触发）
            if (e.key === 'Enter' && !e.shiftKey && !e.nativeEvent.isComposing) {
              e.preventDefault();
              send();
            }
          }}
          onFocus={onFocus}
          placeholder={placeholder}
          aria-label="AI 指令输入"
          disabled={disabled}
          className="min-h-7 min-w-0 flex-1 resize-none bg-transparent px-1 py-[5px] text-[13px] leading-[18px] text-ai-ink outline-none [overflow-wrap:anywhere] placeholder:text-ai-ink-3 disabled:opacity-50"
        />

        {hasModels && (
          <button
            type="button"
            aria-expanded={modelOpen}
            aria-label="选择模型"
            onClick={() => {
              setPlusOpen(false);
              setModelOpen(c => !c);
            }}
            className="flex h-7 shrink-0 items-center gap-1 rounded-[8px] px-1.5 text-[12px] font-medium text-ai-ink-2 transition-colors duration-150 hover:bg-ai-hover hover:text-ai-ink"
          >
            {currentModel?.name ?? '选择模型'}
            <ChevronDown className="size-3 text-ai-ink-3" />
          </button>
        )}

        {trailingAction ?? (
          <button
            type="button"
            title="发送"
            aria-label="发送"
            disabled={!canSend}
            onClick={send}
            className="flex size-7 shrink-0 items-center justify-center rounded-[8px] transition-[background-color,color,transform] duration-200 enabled:active:scale-[0.94] disabled:cursor-not-allowed"
            style={{
              background: canSend ? 'var(--ai-ink)' : 'var(--ai-line-strong)',
              color: canSend ? 'var(--ai-surface)' : 'var(--ai-ink-2)',
            }}
          >
            <ArrowUp className="size-4" />
          </button>
        )}
      </div>
    </div>
  );
}

export default AiPromptBar;
```

**(b) `FrontstageBottomBar.tsx`** 三处改动：

1. import 区：`import { Send, X, Activity, ... } from 'lucide-react'` 中**移除 `Send`**（仅旧发送按钮在用），并在 `import { StatusIcon } from './StatusIcon';` 行后加：

```tsx
import { AiPromptBar } from '@/components/ui/ai/AiPromptBar';
```

文件顶部（`abbreviateApiBase` 前）加命令常量：

```tsx
/** 与 RichTextEditor slash 输入一致的真实命令集（handleSlashSubmit）：
 *  自动续写/审校 走专属通道，其余统一由后端意图识别路由（smart_execute）。
 *  选中后作为纯文本插入输入框，提交路径与手打指令完全一致。 */
const PROMPT_COMMANDS = [
  { key: 'auto_write', name: '/自动续写', desc: '从当前位置自动续写' },
  { key: 'auto_revise', name: '/审校', desc: '审校当前章节' },
  { key: 'revise', name: '/AI修稿', desc: '按指令修改正文' },
  { key: 'review', name: '/AI审稿', desc: '审阅当前章节并给出意见' },
  { key: 'finalize', name: '/定稿', desc: '将当前章节定稿' },
];
```

2. 删除 textarea 自适应高度 effect（L116-129，AiPromptBar 内部自管理高度）——旧片段：

```tsx
  // v0.30.27: textarea 自适应高度，根据输入值 + 幽灵提示动态调整。
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const MAX_TEXTAREA_HEIGHT = 200;

  useEffect(() => {
    const el = textareaRef.current;
    if (!el) return;

    el.style.height = 'auto';
    const scrollHeight = el.scrollHeight;
    const newHeight = Math.min(scrollHeight, MAX_TEXTAREA_HEIGHT);
    el.style.height = `${newHeight}px`;
    el.style.overflowY = scrollHeight > MAX_TEXTAREA_HEIGHT ? 'auto' : 'hidden';
  }, [inputValue, ghostHint, loglineHint]);
```

整段删除。同时 L1 的 `import React, { useEffect, useRef, useState } from 'react';` 改为 `import React, { useState } from 'react';`（`useEffect`/`useRef` 在本文件仅此一处使用）。

3. 输入框区（L382-431）旧片段：

```tsx
            <textarea
              ref={textareaRef}
              className={[
                'relative z-10 w-full bg-transparent border-0 outline-none resize-none',
                'text-ink-900 placeholder-ink-500 font-body text-sm leading-normal',
                'min-h-[24px] max-h-[200px] overflow-y-hidden',
                'disabled:opacity-50 disabled:cursor-not-allowed',
              ].join(' ')}
              placeholder={ghostHint ? '' : '输入任意指令…'}
              value={inputValue}
              onChange={e => onInputChange(e.target.value)}
              onKeyDown={onInputKeyDown}
              onFocus={onInputFocus}
              disabled={isGenerating}
              rows={1}
            />
          </div>

          {isGenerating ? (
            <button
              className={[
                'w-8 h-8 rounded-md flex items-center justify-center p-0 flex-shrink-0',
                'bg-status-danger/15 text-status-danger',
                'hover:bg-status-danger/25',
                'transition-colors duration-150',
                'animate-pulse',
              ].join(' ')}
              onClick={onCancelGeneration}
              title="取消生成"
              aria-label="取消生成"
            >
              <X className="w-4 h-4" />
            </button>
          ) : (
            <button
              className={[
                'w-8 h-8 rounded-md flex items-center justify-center p-0 flex-shrink-0',
                'bg-terracotta/10 text-terracotta',
                'hover:bg-terracotta/20',
                'transition-colors duration-150',
                'disabled:bg-paper-200 disabled:text-ink-500/50 disabled:cursor-not-allowed',
              ].join(' ')}
              onClick={onInputSubmit}
              disabled={!inputValue.trim()}
              title="发送"
              aria-label="发送"
            >
              <Send className="w-4 h-4" />
            </button>
          )}
```

替换为：

```tsx
            <AiPromptBar
              value={inputValue}
              onChange={onInputChange}
              onSend={onInputSubmit}
              placeholder={ghostHint ? '' : '输入任意指令…'}
              disabled={isGenerating}
              commands={PROMPT_COMMANDS}
              onKeyDown={onInputKeyDown}
              onFocus={onInputFocus}
              trailingAction={
                isGenerating ? (
                  <button
                    className={[
                      'w-7 h-7 rounded-[8px] flex items-center justify-center p-0 flex-shrink-0',
                      'bg-status-danger/15 text-status-danger',
                      'hover:bg-status-danger/25',
                      'transition-colors duration-150',
                      'animate-pulse',
                    ].join(' ')}
                    onClick={onCancelGeneration}
                    title="取消生成"
                    aria-label="取消生成"
                  >
                    <X className="w-4 h-4" />
                  </button>
                ) : undefined
              }
            />
          </div>
```

说明：
- 幽灵提示覆盖层（L356-381 两个 `frontstage-input-ghost` span）**保留不动**，仍在同一 relative 容器内覆盖于输入区之上。
- Enter 发送路径：父级 `onInputKeyDown`（FrontstageApp `handleInputKeyDown`）对 Enter preventDefault + 提交，AiPromptBar 内部检测到 `defaultPrevented` 后跳过自身 `onSend`，不会重复提交；无父级处理时由 AiPromptBar 自身 Enter→`onSend`。
- @ 数据源（角色/世界观名）在 BottomBar 现有 props/hooks 中**不易获得**——P1 不传 `sources`（@ 菜单与 + 按钮自动不渲染），留待 P2。
- 模型选择器 P1 **不传 `models`**（整体不渲染），留待 P2。

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiPromptBar.test.tsx src/frontstage/components/__tests__/FrontstageBottomBar.test.tsx && npx tsc --noEmit`
Expected: AiPromptBar 8 passed；FrontstageBottomBar 既有 17 项测试不回归（placeholder/title/ghost hint/取消按钮断言均保持）；tsc 干净

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiPromptBar.tsx src-frontend/src/components/ui/ai/__tests__/AiPromptBar.test.tsx src-frontend/src/frontstage/components/FrontstageBottomBar.tsx
git commit -m "feat: AiPromptBar 组件入库并替换幕前底部指令输入条（P1 Task5）"
```

---

### Task 6: AiApprovalCard 组件 + NovelCreationWizard 四个选项步骤替换

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiApprovalCard.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiApprovalCard.test.tsx`
- Modify: `src-frontend/src/components/NovelCreationWizard.tsx`（import 区；`renderStrategySelection` L269-365、`renderWorldSelection` L367-421、`renderCharacterSelection` L423-478、`renderStyleSelection` L480-520）

**Interfaces:**
- Consumes: Task 1 的 `bg-ai-surface` / `border-ai-line` / `bg-ai-hover` / `bg-ai-ink` / `bg-ai-green` / `animate-pop-in` / `animate-ai-fade-up`；lucide-react（ArrowUp / Check / ChevronLeft / ChevronRight / X）
- Produces:
  - `export interface AiApprovalOption { key: string; label: string; description?: string }`
  - `export interface AiApprovalQuestion { key: string; title: string; type: 'radio' | 'check'; options: AiApprovalOption[]; allowCustom?: boolean }`
  - `export interface AiApprovalCardProps { questions: AiApprovalQuestion[]; onSubmit: (answers: Record<string, string[]>) => void; onDismiss?: () => void; submitLabel?: string /* 默认「提交」 */ }`
  - `export function AiApprovalCard(props): JSX.Element` — 一次一题；ring-dot 分页器 + 上一题/下一题；radio 选中 480ms 后自动前进（最后一题自动提交）；check 多选不自动前进；`allowCustom` 提供「自定义回答…」输入（radio 题输入自定义时清空已选）；提交后绿色对勾 + 「已提交」（`animate-pop-in`）。answers 以 `question.key -> option.key[]` 上报，自定义回答以文本为数组元素
  - 剥离参考实现的 QUESTIONS 常量与 open/reset 演示状态（「Start over」/「Open approval」不落）

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiApprovalCard.test.tsx
import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { AiApprovalCard } from '../AiApprovalCard';

const questions = [
  {
    key: 'world',
    title: '选择世界观',
    type: 'radio' as const,
    options: [
      { key: '0', label: '苍澜大陆', description: '修真文明与蒸汽机械并存' },
      { key: '1', label: '雾都伦城', description: '维多利亚悬疑' },
    ],
  },
  {
    key: 'tags',
    title: '选择故事元素',
    type: 'check' as const,
    options: [
      { key: 'a', label: '悬疑' },
      { key: 'b', label: '成长' },
    ],
    allowCustom: true,
  },
];

describe('AiApprovalCard', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('一次只显示一个问题，分页按钮可前后切换', () => {
    render(<AiApprovalCard questions={questions} onSubmit={() => {}} />);
    expect(screen.getByText('选择世界观')).toBeInTheDocument();
    expect(screen.queryByText('选择故事元素')).not.toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('下一题'));
    expect(screen.getByText('选择故事元素')).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('上一题'));
    expect(screen.getByText('选择世界观')).toBeInTheDocument();
  });

  it('radio 选中 480ms 后自动前进；最后一题提交显示「已提交」并按 key 上报', () => {
    vi.useFakeTimers();
    const onSubmit = vi.fn();
    render(<AiApprovalCard questions={questions} onSubmit={onSubmit} />);
    fireEvent.click(screen.getByText('苍澜大陆'));
    act(() => {
      vi.advanceTimersByTime(500);
    });
    expect(screen.getByText('选择故事元素')).toBeInTheDocument(); // 自动前进
    fireEvent.click(screen.getByText('悬疑')); // check 不自动前进
    act(() => {
      vi.advanceTimersByTime(500);
    });
    expect(screen.queryByText('已提交')).not.toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('提交'));
    expect(screen.getByText('已提交')).toBeInTheDocument();
    expect(onSubmit).toHaveBeenCalledWith({ world: ['0'], tags: ['a'] });
  });

  it('check 多选可再点取消；radio 输入自定义回答时清空已选', () => {
    render(<AiApprovalCard questions={[questions[1]]} onSubmit={() => {}} />);
    fireEvent.click(screen.getByText('悬疑'));
    fireEvent.click(screen.getByText('成长'));
    fireEvent.click(screen.getByText('悬疑')); // 取消
    expect(screen.getByLabelText('提交')).toBeEnabled(); // 仍有「成长」
  });

  it('allowCustom 自定义回答以文本作为答案提交', () => {
    const onSubmit = vi.fn();
    render(<AiApprovalCard questions={[questions[1]]} onSubmit={onSubmit} />);
    fireEvent.change(screen.getByLabelText('自定义回答'), { target: { value: '我自己的答案' } });
    fireEvent.click(screen.getByLabelText('提交'));
    expect(onSubmit).toHaveBeenCalledWith({ tags: ['我自己的答案'] });
  });

  it('onDismiss 传入时渲染关闭按钮并回调', () => {
    const onDismiss = vi.fn();
    render(<AiApprovalCard questions={questions} onSubmit={() => {}} onDismiss={onDismiss} />);
    fireEvent.click(screen.getByLabelText('关闭'));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiApprovalCard.test.tsx`
Expected: FAIL（模块 `../AiApprovalCard` 不存在）

- [ ] **Step 3: Write implementation**

```tsx
// src-frontend/src/components/ui/ai/AiApprovalCard.tsx
/**
 * AiApprovalCard — 人工审批/选项卡（适配自 beautifului ApprovalCard）
 *
 * 受控组件：questions 由调用方提供。一次一个问题；ring-dot 分页器 +
 * 上一题/下一题；radio 单选 480ms 后自动前进（最后一题自动提交）；
 * allowCustom 时提供「自定义回答…」输入；提交后显示绿色对勾「已提交」。
 * answers 以 question.key -> option.key[] 上报（自定义回答以文本为数组元素）。
 */
import { useState } from 'react';
import { ArrowUp, Check, ChevronLeft, ChevronRight, X } from 'lucide-react';

export interface AiApprovalOption {
  key: string;
  label: string;
  description?: string;
}

export interface AiApprovalQuestion {
  key: string;
  title: string;
  type: 'radio' | 'check';
  options: AiApprovalOption[];
  allowCustom?: boolean;
}

export interface AiApprovalCardProps {
  questions: AiApprovalQuestion[];
  onSubmit: (answers: Record<string, string[]>) => void;
  onDismiss?: () => void;
  submitLabel?: string;
}

export function AiApprovalCard({
  questions,
  onSubmit,
  onDismiss,
  submitLabel = '提交',
}: AiApprovalCardProps) {
  const [qi, setQi] = useState(0);
  const [answers, setAnswers] = useState<Record<string, string[]>>({});
  const [custom, setCustom] = useState<Record<string, string>>({});
  const [sent, setSent] = useState(false);
  const question = questions[qi];
  const last = qi === questions.length - 1;
  const selected = answers[question.key] ?? [];
  const hasAnswer = selected.length > 0 || Boolean(custom[question.key]?.trim());

  /* 汇总答案：每题优先 option key 列表，无选中则取自定义文本 */
  const buildAnswers = (extra?: Record<string, string[]>) => {
    const merged = { ...answers, ...extra };
    const out: Record<string, string[]> = {};
    for (const q of questions) {
      const picked = merged[q.key] ?? [];
      const text = custom[q.key]?.trim();
      if (picked.length > 0) out[q.key] = picked;
      else if (text) out[q.key] = [text];
    }
    return out;
  };

  const submit = (finalAnswers?: Record<string, string[]>) => {
    setSent(true);
    onSubmit(finalAnswers ?? buildAnswers());
  };

  const toggle = (optionKey: string) => {
    if (question.type === 'radio') {
      const next = { ...answers, [question.key]: [optionKey] };
      setAnswers(next);
      setCustom(c => ({ ...c, [question.key]: '' }));
      // 单选自动前进；最后一题自动提交（extra 透传避免读到旧 state）
      window.setTimeout(() => {
        if (last) submit(buildAnswers({ [question.key]: [optionKey] }));
        else setQi(c => Math.min(questions.length - 1, c + 1));
      }, 480);
    } else {
      setAnswers(c => {
        const picked = c[question.key] ?? [];
        return {
          ...c,
          [question.key]: picked.includes(optionKey)
            ? picked.filter(k => k !== optionKey)
            : [...picked, optionKey],
        };
      });
    }
  };

  return (
    <div
      className="w-full overflow-hidden rounded-[12px] border border-ai-line bg-ai-surface shadow-float"
      data-testid="ai-approval-card"
    >
      {sent ? (
        <div className="flex h-36 flex-col items-center justify-center gap-2">
          <span className="animate-pop-in flex size-6 items-center justify-center rounded-full bg-ai-green text-white">
            <Check className="size-3.5" strokeWidth={3} />
          </span>
          <span className="animate-ai-fade-up text-[13px] font-medium text-ai-ink">已提交</span>
        </div>
      ) : (
        <div key={question.key} className="animate-ai-fade-up p-4">
          <div className="flex items-start justify-between gap-3">
            <span className="text-[13px] font-medium text-ai-ink">{question.title}</span>
            {onDismiss && (
              <button
                type="button"
                aria-label="关闭"
                onClick={onDismiss}
                className="flex size-5 shrink-0 items-center justify-center rounded-[5px] text-ai-ink-3 transition-colors duration-100 hover:bg-ai-hover hover:text-ai-ink"
              >
                <X className="size-3.5" />
              </button>
            )}
          </div>
          <div className="mt-2 flex flex-col gap-0.5">
            {question.options.map(option => {
              const on = selected.includes(option.key);
              return (
                <button
                  key={option.key}
                  type="button"
                  aria-pressed={on}
                  onClick={() => toggle(option.key)}
                  className="-mx-1.5 flex items-center gap-2 rounded-[8px] px-1.5 py-1.5 text-left transition-colors duration-100 hover:bg-ai-hover"
                >
                  <span
                    className={`flex size-4 shrink-0 items-center justify-center transition-colors duration-200 ${
                      question.type === 'radio' ? 'rounded-full' : 'rounded-[5px]'
                    } ${
                      on
                        ? 'bg-ai-ink text-ai-surface'
                        : 'shadow-[inset_0_0_0_1.5px_var(--ai-line-strong)] text-transparent'
                    }`}
                  >
                    {question.type === 'radio' ? (
                      <span
                        className="size-1.5 rounded-full bg-ai-surface transition-transform duration-200"
                        style={{ transform: on ? 'scale(1)' : 'scale(0)' }}
                      />
                    ) : (
                      <Check className="size-3" strokeWidth={3} />
                    )}
                  </span>
                  <span className="min-w-0">
                    <span
                      className={`block text-[13px] transition-colors duration-200 ${
                        on ? 'text-ai-ink' : 'text-ai-ink-2'
                      }`}
                    >
                      {option.label}
                    </span>
                    {option.description && (
                      <span className="block truncate text-[11.5px] text-ai-ink-3">
                        {option.description}
                      </span>
                    )}
                  </span>
                </button>
              );
            })}
            {question.allowCustom && (
              <label className="-mx-1.5 flex items-center gap-2 rounded-[8px] px-1.5 py-1 transition-colors duration-100 focus-within:bg-ai-hover hover:bg-ai-hover">
                <span aria-hidden className="size-4 shrink-0" />
                <input
                  value={custom[question.key] ?? ''}
                  onChange={e => {
                    setCustom(c => ({ ...c, [question.key]: e.target.value }));
                    if (question.type === 'radio') {
                      setAnswers(c => ({ ...c, [question.key]: [] }));
                    }
                  }}
                  placeholder="自定义回答…"
                  aria-label="自定义回答"
                  className="min-w-0 flex-1 bg-transparent text-[13px] text-ai-ink outline-none placeholder:text-ai-ink-3"
                />
              </label>
            )}
          </div>
        </div>
      )}

      {/* footer — ring-dot 分页器 + 提交箭头 */}
      <div className="flex items-center justify-between border-t border-ai-line px-4 py-2">
        <span className="flex items-center gap-2">
          <button
            type="button"
            aria-label="上一题"
            disabled={qi === 0 || sent}
            onClick={() => setQi(c => Math.max(0, c - 1))}
            className="flex size-6 items-center justify-center rounded-[5px] text-ai-ink-3 transition-colors duration-100 enabled:hover:bg-ai-hover enabled:hover:text-ai-ink-2 disabled:opacity-35"
          >
            <ChevronLeft className="size-3.5" />
          </button>
          <span className="flex items-center gap-1">
            {questions.map((q, i) => (
              <button
                key={q.key}
                type="button"
                aria-label={`第 ${i + 1} 题`}
                aria-current={i === qi && !sent ? 'step' : undefined}
                disabled={sent}
                onClick={() => setQi(i)}
                className="rounded-full transition-all duration-300 disabled:cursor-default"
                style={
                  i === qi && !sent
                    ? { width: 9, height: 9, border: '2.5px solid var(--ai-ink)' }
                    : sent || i < qi
                      ? { width: 7, height: 7, background: 'var(--ai-ink-3)' }
                      : { width: 7, height: 7, border: '1.5px solid var(--ai-ink-3)' }
                }
              />
            ))}
          </span>
          <button
            type="button"
            aria-label="下一题"
            disabled={last || sent}
            onClick={() => setQi(c => Math.min(questions.length - 1, c + 1))}
            className="flex size-6 items-center justify-center rounded-[5px] text-ai-ink-3 transition-colors duration-100 enabled:hover:bg-ai-hover enabled:hover:text-ai-ink-2 disabled:opacity-35"
          >
            <ChevronRight className="size-3.5" />
          </button>
        </span>
        {!sent && (
          <button
            type="button"
            aria-label={last ? submitLabel : '继续'}
            disabled={!hasAnswer}
            onClick={() => (last ? submit() : setQi(c => c + 1))}
            className="-mr-0.5 flex size-7 items-center justify-center rounded-[8px] transition-[background-color,color,transform] duration-200 enabled:active:scale-[0.96] disabled:cursor-not-allowed"
            style={{
              background: hasAnswer ? 'var(--ai-ink)' : 'var(--ai-field)',
              color: hasAnswer ? 'var(--ai-surface)' : 'var(--ai-ink-3)',
            }}
          >
            <ArrowUp className="size-3.5" strokeWidth={2.5} />
          </button>
        )}
      </div>
    </div>
  );
}

export default AiApprovalCard;
```

**(b) `NovelCreationWizard.tsx`**：import 区（L13 `import { Button } from '@/components/ui/Button';` 行前）加：

```tsx
import { AiApprovalCard } from '@/components/ui/ai/AiApprovalCard';
```

四个步骤现状均为**单选**（点击卡片即生成下一步），故全部映射为单题 `type: 'radio'`——选中后 480ms 自动提交，直接触发既有 handler（行为与现状一致，仅多了 480ms 确认动画）。

1. `renderStrategySelection`：保留推荐理由 Card 与说明文案，替换底部按钮行。旧片段（L349-363）：

```tsx
      <div className="flex justify-between">
        <Button variant="ghost" onClick={handleBack}>
          <ChevronLeft className="w-4 h-4 mr-1" />
          上一步
        </Button>
        <Button
          variant="primary"
          onClick={handleConfirmStrategy}
          disabled={!selectedStrategy || isGenerating}
          isLoading={isGenerating}
        >
          <Sparkles className="w-4 h-4 mr-2" />
          确认策略，生成世界观
        </Button>
      </div>
    </div>
  );
```

替换为：

```tsx
      <div className="flex justify-between">
        <Button variant="ghost" onClick={handleBack}>
          <ChevronLeft className="w-4 h-4 mr-1" />
          上一步
        </Button>
      </div>

      {selectedStrategy && !isGenerating && (
        <AiApprovalCard
          questions={[
            {
              key: 'strategy',
              title: '确认采用 AI 推荐的创作策略？',
              type: 'radio',
              options: [
                {
                  key: 'accept',
                  label: '采用推荐策略，生成世界观',
                  description: selectedStrategy.rationale || '未提供推荐理由',
                },
              ],
            },
          ]}
          onSubmit={() => handleConfirmStrategy()}
        />
      )}
    </div>
  );
```

2. `renderWorldSelection`：替换选项网格与副标题。旧片段（L369-421）：

```tsx
      <div className="text-center">
        <h2 className="text-2xl font-bold text-white mb-2">选择世界观</h2>
        <p className="text-gray-400">双击可编辑，点击选择</p>
      </div>

      <div className="grid gap-4">
        {worldOptions.map((world, index) => (
          <Card
            key={world.id}
            hover
            className={`cursor-pointer transition-all ${
              selectedWorld === index ? 'ring-2 ring-cinema-gold' : ''
            }`}
            onClick={() => handleSelectWorld(index)}
          >
            <CardContent className="p-5">
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 rounded-xl bg-cinema-gold/10 flex items-center justify-center flex-shrink-0">
                  <Globe className="w-6 h-6 text-cinema-gold" />
                </div>
                <div className="flex-1">
                  <h3 className="font-semibold text-white mb-2">{world.concept}</h3>
                  <div className="space-y-2">
                    <div>
                      <span className="text-xs text-gray-500">核心规则：</span>
                      <div className="flex flex-wrap gap-1 mt-1">
                        {world.rules.map((rule, i) => (
                          <span
                            key={i}
                            className="px-2 py-0.5 text-xs bg-cinema-800 rounded text-gray-300"
                          >
                            {rule.name}
                          </span>
                        ))}
                      </div>
                    </div>
                    <p className="text-sm text-gray-400 line-clamp-2">{world.history}</p>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <div className="flex justify-between">
        <Button variant="ghost" onClick={handleBack}>
          <ChevronLeft className="w-4 h-4 mr-1" />
          上一步
        </Button>
      </div>
    </div>
  );
```

替换为：

```tsx
      <div className="text-center">
        <h2 className="text-2xl font-bold text-white mb-2">选择世界观</h2>
        <p className="text-gray-400">选择后自动开始生成角色谱</p>
      </div>

      <AiApprovalCard
        questions={[
          {
            key: 'world',
            title: '选择世界观',
            type: 'radio',
            options: worldOptions.map((world, index) => ({
              key: String(index),
              label: world.concept,
              description: world.history.slice(0, 60),
            })),
          },
        ]}
        onSubmit={answers => handleSelectWorld(Number(answers.world[0]))}
      />

      <div className="flex justify-between">
        <Button variant="ghost" onClick={handleBack}>
          <ChevronLeft className="w-4 h-4 mr-1" />
          上一步
        </Button>
      </div>
    </div>
  );
```

3. `renderCharacterSelection`：替换选项网格。旧片段（L430-469）：

```tsx
      <div className="grid gap-4">
        {characterSets.map((characterSet, index) => (
          <Card
            key={index}
            hover
            className={`cursor-pointer transition-all ${
              selectedCharacters === index ? 'ring-2 ring-cinema-gold' : ''
            }`}
            onClick={() => handleSelectCharacters(index)}
          >
            <CardContent className="p-5">
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 rounded-xl bg-cinema-gold/10 flex items-center justify-center flex-shrink-0">
                  <Users className="w-6 h-6 text-cinema-gold" />
                </div>
                <div className="flex-1">
                  <div className="flex flex-wrap gap-2 mb-3">
                    {characterSet.map(char => (
                      <span
                        key={char.id}
                        className="px-2.5 py-1 rounded-lg bg-cinema-800 text-gray-300 text-sm"
                      >
                        {char.name}
                      </span>
                    ))}
                  </div>
                  <div className="space-y-1">
                    {characterSet.map(char => (
                      <p key={char.id} className="text-sm text-gray-400">
                        <span className="text-gray-300">{char.name}：</span>
                        {char.personality} · {char.goals}
                      </p>
                    ))}
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
```

替换为：

```tsx
      <AiApprovalCard
        questions={[
          {
            key: 'characters',
            title: '选择一组核心角色配置',
            type: 'radio',
            options: characterSets.map((characterSet, index) => ({
              key: String(index),
              label: characterSet.map(c => c.name).join('、'),
              description: characterSet
                .map(c => `${c.name}：${c.personality} · ${c.goals}`)
                .join('；')
                .slice(0, 80),
            })),
          },
        ]}
        onSubmit={answers => handleSelectCharacters(Number(answers.characters[0]))}
      />
```

4. `renderStyleSelection`：替换选项网格。旧片段（L487-511）：

```tsx
      <div className="grid gap-4">
        {styleOptions.map((style, index) => (
          <Card
            key={style.id}
            hover
            className={`cursor-pointer transition-all ${
              selectedStyle === index ? 'ring-2 ring-cinema-gold' : ''
            }`}
            onClick={() => handleSelectStyle(index)}
          >
            <CardContent className="p-5">
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 rounded-xl bg-cinema-gold/10 flex items-center justify-center flex-shrink-0">
                  <PenTool className="w-6 h-6 text-cinema-gold" />
                </div>
                <div className="flex-1">
                  <h3 className="font-semibold text-white mb-1">{style.name}</h3>
                  <p className="text-sm text-gray-400 mb-2">{style.description}</p>
                  <p className="text-xs text-gray-500 italic line-clamp-2">"{style.sample_text}"</p>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
```

替换为：

```tsx
      <AiApprovalCard
        questions={[
          {
            key: 'style',
            title: '选择适合你故事的文字风格',
            type: 'radio',
            options: styleOptions.map((style, index) => ({
              key: String(index),
              label: style.name,
              description: `${style.description}（示例：${style.sample_text.slice(0, 40)}…）`,
            })),
          },
        ]}
        onSubmit={answers => handleSelectStyle(Number(answers.style[0]))}
      />
```

既有 handler（`handleSelectWorld` / `handleSelectCharacters` / `handleSelectStyle` / `handleConfirmStrategy`）全部保留原签名与防重入逻辑不动。`Card`/`CardContent`/`Globe`/`Users`/`PenTool` import 保留（`renderCompleted` 仍在用 Card 以外的图标；若 tsc/eslint 报 `Card`/`CardContent` 未使用，则从 L14 的 import 中移除——`renderCompleted` 用的是裸 div，不依赖 Card）。

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiApprovalCard.test.tsx src/utils/__tests__/applyWizardToStory.test.ts && npx tsc --noEmit`
Expected: AiApprovalCard 5 passed；wizard 相关既有测试不回归；tsc 干净

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiApprovalCard.tsx src-frontend/src/components/ui/ai/__tests__/AiApprovalCard.test.tsx src-frontend/src/components/NovelCreationWizard.tsx
git commit -m "feat: AiApprovalCard 组件入库并替换创建向导四个选项步骤（P1 Task6）"
```

---

### Task 7: 全量回归门 + 文档同步

**Files:**
- Modify: `CHANGELOG.md`（顶部加 Unreleased 段）
- Modify: `PROJECT_STATUS.md`（「最近完成功能」加 P1 条目）
- Modify: `AGENTS.md`（编码风格节加 AI 组件约定）
- Modify: `src-frontend/src/components/index.ts`（barrel 导出 5 个组件）

**Interfaces:**
- Consumes: Task 2-6 全部产出
- Produces: barrel 导出 `AiLoading` / `AiThinking` / `AiStreamingText` / `AiPromptBar` / `AiApprovalCard`（+ 各自 Props/Row/Question 类型用 `export type`）

- [ ] **Step 1: barrel 导出**

`src-frontend/src/components/index.ts` 在 L4 `export { DataLoader } from './DataLoader';` 行前插入：

```ts
// P1 - AI Native Components（生成体验）
export { AiLoading } from './ui/ai/AiLoading';
export { AiThinking } from './ui/ai/AiThinking';
export { AiStreamingText } from './ui/ai/AiStreamingText';
export { AiPromptBar } from './ui/ai/AiPromptBar';
export { AiApprovalCard } from './ui/ai/AiApprovalCard';
export type { AiLoadingProps } from './ui/ai/AiLoading';
export type { AiThinkingProps, AiThinkingRow } from './ui/ai/AiThinking';
export type { AiStreamingTextProps } from './ui/ai/AiStreamingText';
export type {
  AiPromptBarProps,
  AiPromptSource,
  AiPromptCommand,
  AiPromptModel,
} from './ui/ai/AiPromptBar';
export type {
  AiApprovalCardProps,
  AiApprovalQuestion,
  AiApprovalOption,
} from './ui/ai/AiApprovalCard';
```

- [ ] **Step 2: 全量回归门**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | tail -3 && npm run format:check`
Expected: tsc 干净；vitest **≥486 passed / 3 skipped**（基线 455 + Task1 5 + Task2 4 + Task3 5 + Task4 5 + Task5 8 + Task6 5 = 487 预期下限 486，以实际输出为准并记录进 CHANGELOG；只允许比基线多）；format 通过

Run: `python3 scripts/architecture_guard.py`
Expected: 退出码 0（无前端改动涉 Rust 模块，应为通过）

- [ ] **Step 3: 文档同步（版本号不动，发版另行进行）**

**(a) `CHANGELOG.md`** — 在 `# Changelog` 头与 `## v0.38.2（2026-08-12）` 之间插入：

```markdown
## Unreleased（P1 AI 原生组件库 · 生成体验）

### 功能：beautifului AI 原生组件第一批（设计文档 P1 范围）

将 beautifului.dev 的 5 个生成体验组件适配为受控组件入库 `src-frontend/src/components/ui/ai/`，并逐点接入幕后/幕前落点。全部组件只引用 `--ai-*` 语义令牌（幕后 tokens.css / 幕前 frontstage.css 双窗口各自定义，同一组件代码两侧正确着色），不引新依赖（图标 lucide-react，动画手写 CSS keyframes）。

- **令牌桥（Task1）**：新增 16 个 `--ai-*` 变量（surface/inset/field/hover/hover-2/ink×3/line×2/accent×3/green/red/orange），幕后取 cinema/status 系、幕前取 ivory/terracotta/oklch 徽章色系；tailwind.config.js 注册 `ai-*` 色组与 9 个 keyframes/动画工具（pixel-on/shimmer-text/ai-fade-up/pop-in/stream-in/ai-spin/eq-bounce/ai-sweep/ai-blink）；两窗口 CSS 均加 prefers-reduced-motion 动画冻结。
- **AiLoading（Task2）**：像素格点加载器（drive/dots/orbit + shimmer 标签 + startedAt 起算的 mono 计时），替换 GenesisPanel 当前步 spinner、GuidebookDistillationPanel 状态图标与进度块文案、NovelCreationWizard renderGenerating。
- **AiThinking（Task3）**：数据驱动的可展开执行轨迹（grid 0fr/1fr、行交错 fade-up、生长竖线、working 末行 spinner），接入 AgencyStudio 时间线顶部「当前执行轨迹」（runActivities 最近 12 条），原时间线保留为历史。
- **AiStreamingText（Task4）**：中文词级分词（Intl.Segmenter，逐字回退）的流式渲染，新单位 stream-in 模糊入场 + 闪烁光标，包裹幕前幽灵续写段落；删除死代码 frontstage `StreamingText.tsx` + `useStreamingGeneration.ts`。
- **AiPromptBar（Task5）**：受控指令输入条（自动增高、/ 命令菜单滑动高亮 + 键盘导航 + IME 守卫、可选模型选择器 + ai-sweep CSS 扫光），替换 FrontstageBottomBar 主输入区；命令集 = RichTextEditor slash 真实命令（自动续写/审校/AI修稿/AI审稿/定稿）；@ 数据源与模型选择器 P1 不接（P2）。
- **AiApprovalCard（Task6）**：一题一页审批卡（ring-dot 分页、radio 480ms 自动前进、自定义回答、已提交态），替换创建向导策略确认/世界观/角色谱/文风四个选项步骤，既有 handler 不变。

### 测试

- src-frontend `npx vitest run`：**<以 Task7 Step2 实际输出填写> passed / 3 skipped**（基线 455 + 本批新增 32）。
```

**(b) `PROJECT_STATUS.md`** — 在 `## ✅ 最近完成功能` 下、`### v0.38.2` 条目前插入：

```markdown
### Unreleased - beautifului AI 原生组件 P1（生成体验五件套）（2026-08-12）

- **令牌桥**：`--ai-*` 语义令牌 16 个双窗口各自定义（幕后 tokens.css/幕前 frontstage.css），tailwind 注册 ai 色组 + 9 个动画工具，reduced-motion 冻结。
- **五组件入库** `components/ui/ai/`：AiLoading（幕后 3 处加载指示）、AiThinking（AgencyStudio 当前执行轨迹）、AiStreamingText（幕前幽灵续写，中文词级分词）、AiPromptBar（幕前底部指令条 + / 命令菜单）、AiApprovalCard（创建向导四选项步骤）。
- **清理**：删除幕前死代码 `StreamingText.tsx` + `useStreamingGeneration.ts`。
- **验证**：`npx tsc --noEmit` / `npx vitest run`（<实际数> passed / 3 skipped）/ `format:check` / `architecture_guard.py` 全绿。版本号未动，发版另行进行。
```

**(c) `AGENTS.md`** — 在 `## 编码风格` 节的 TypeScript 条目后追加一行：

```markdown
- **AI 原生组件**: `src-frontend/src/components/ui/ai/`（AiLoading/AiThinking/AiStreamingText/AiPromptBar/AiApprovalCard），只引用 `--ai-*` 语义令牌（幕后 tokens.css / 幕前 frontstage.css 各自定义），不写死颜色；动画用 tailwind.config.js 注册的 ai keyframes 工具类；受控组件，禁止引入自运行演示逻辑。
```

- [ ] **Step 4: Commit**

```bash
git add src-frontend/src/components/index.ts CHANGELOG.md PROJECT_STATUS.md AGENTS.md
git commit -m "docs: P1 AI 原生组件库 barrel 导出与文档同步（P1 Task7）"
```

---

## Self-Review 结论

- **Spec coverage（对照设计文档 §8 P1 + 任务书）**：
  - 令牌桥：`--ai-*` 16 变量双窗口定义 + tailwind 映射 + 8 个任务书指定 keyframes（pixel-on/shimmer-text/fade-up/pop-in/stream-in/ai-spin/eq-bounce/ai-sweep）= Task1 ✅；另增 `ai-blink`（AiStreamingText 闪烁光标需要，任务书「blinking cursor」）——增补项，已在 Task1/4 标注。
  - AiLoading / AiThinking / AiStreamingText / AiPromptBar / AiApprovalCard 五组件 API 与任务书逐项一致 ✅（AiPromptBar 增补 `onKeyDown`/`onFocus`/`trailingAction` 三个可选透传 prop——BottomBar 既有幽灵提示导航/focus 回调/取消按钮所必需，已在 Task5 Interfaces 中明示）。
  - 落点：Task2 三处（GenesisPanel L443-457、GuidebookDistillationPanel L122-125+L152-155、Wizard renderGenerating L257-267）、Task3 AgencyStudio L355-369、Task4 RichTextEditor L1368-1378 + 死代码删除（index.ts:2 导出移除）、Task5 FrontstageBottomBar L382-431（含 L116-129 自适应 effect 移除）、Task6 Wizard 四步骤、Task7 回归门 + 文档 ✅。
  - 任务书明确不做项已照办：无 liveline/glimm/iconoir-react/framer-motion；glimm → `ai-sweep` CSS；PromptBar 附件整体不落；BottomBar @ 数据源 P1 不传（不易获得，注记 P2）；模型选择器 P1 不传 models（注记 P2）；StreamingText 的 citations/sources/follow-ups 不落（组件注释标注未来工作）；版本号不动。
- **Placeholder scan**：全文无 TBD/「类似」/「适当处理」；每个代码步含完整代码或精确 before/after 片段。唯一待定值：CHANGELOG/PROJECT_STATUS 中的最终测试计数 `<以 Task7 Step2 实际输出填写>`——这是设计上必须由执行者填入的真实命令输出，非占位符。
- **Type consistency**：
  - `AiLoadingProps.variant` 联合类型 `'drive'|'dots'|'orbit'` 在 Task2 组件/测试/三处集成一致（drive/dots/orbit 各用一处）。
  - `AiThinkingRow` 字段与 Task3 集成映射（primary/secondary）及测试（mono/add/del/href）一致；`−41` 使用 U+2212，与组件实现 `−{row.del ?? 0}` 一致。
  - `AiStreamingTextProps{ text, done, className }` 与 Task4 集成（`text={generatedText} done={!isGenerating}`，RichTextEditor 既有 props L94-95）一致。
  - `AiPromptBarProps` 与 Task5 测试及 BottomBar 集成（value/onChange/onSend/placeholder/disabled/commands/onKeyDown/onFocus/trailingAction）一致；`send` 内部不重复调用父级已 preventDefault 的 Enter。
  - `AiApprovalCardProps.onSubmit(Record<string, string[]>)` 与 Task6 四个 `Number(answers.<key>[0])` 映射一致；`submitLabel` 默认「提交」与测试 `getByLabelText('提交')` 一致。
  - Task1 测试断言的 keyframe 条目写法（`'fade-up': {`）与 tailwind.config.js 插入代码逐字符一致；`ai-fade-up` 工具名未覆盖既有 `animate-fade-up`（FrontstageBottomBar L439 在用，零回归）。
