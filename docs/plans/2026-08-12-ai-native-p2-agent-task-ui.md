# P2 AI 原生组件库第二批（代理与任务）Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 beautifului.dev 的 5 个 AI 原生组件（ContextCards / ToolChips / RecommendationCard / TaskRows / SelectionActions）适配为受控组件入库 `src-frontend/src/components/ui/ai/`，并逐点替换/接入：PromptCoverageBar 上下文槽位勾叉清单、Tasks 状态筛选条 + Skills 分类筛选条、Tasks 级联改写逐段接受/拒绝卡片、Tasks 任务行外壳、RichTextEditor 划词 AI 操作浮条（新增挂载）。

**Architecture:** 复用 P1 令牌层（`--ai-*` 变量 + tailwind `ai-*` 色与 keyframes，本批不新增令牌；仅 Task 5 在幕前 frontstage.css 补一个既有工具类所需的 `--shadow-float` 变量）→ 组件层（`components/ui/ai/`，全部受控、无自运行演示逻辑）→ 集成层（逐文件 before/after 替换）。组件只引用 `--ai-*` 语义令牌与 tailwind 注册的 keyframes 工具类，不写死颜色、不引新依赖；图标用既有 `lucide-react`（SelectionActions 的 10 个 iconoir 图标按本计划映射表替换）。

**Tech Stack:** React 18 + Tailwind v3.4（`var()` 色映射）、vitest 4 + Testing Library、jsdom、lucide-react（既有依赖）。

## Global Constraints

- 仓库 /Users/yuzaimu/projects/StoryForge；master 直接工作；中文 conventional commit；不 --no-verify；不推送、不打 tag。
- **不引入新依赖**：禁止 `liveline` / `glimm` / `iconoir-react` / framer-motion；图标只用 `lucide-react`。
- 组件全部为**受控组件**：剥离参考实现中的 CHUNKS/ROWS/DIFFS/OPTIONS/TICKS/LEAD/PICKED/REWRITE 等演示数据与 useSequence/useTick 自运行步进逻辑；展开/收起、选中项等纯视图状态允许组件内部持有（同 P1 AiThinking 的 manualExpanded 先例）。
- **不改 P1 已入库组件**：AiLoading/AiThinking/AiStreamingText/AiPromptBar/AiApprovalCard 代码一行不动。SelectionActions 需要的 StreamText 是组件内嵌私有实现（复用 `segmentStreamText` 导出函数 + `animate-stream-in` 类），不改 `AiStreamingText.tsx`。
- **纯前端阶段**：不改任何 Rust 后端代码；`cargo test` 基线 1326 passed / 2 ignored 不变，本批无需重跑。
- 两个窗口是独立 webview 文档：P2 组件多数落在幕后（tokens.css 提供 `--ai-*`），AiSelectionActions 落在幕前（frontstage.css 提供 `--ai-*`）；同一组件代码两侧均能正确着色。
- 移植规则（勘察结论，逐条执行）：内联 `animation:` 引用裸 keyframes 名 → 改 `animate-*` 类 + 内联 `animationDelay`（否则 tailwind 不输出）；删全部 `dark:*` 变体；`var(--line)` 等直引改 `--ai-*`；`green-tint`/`red-tint`/`orange-tint` 无令牌 → `bg-ai-*/10` 或 color-mix；`rounded-card/control/chip`、`shadow-card/btn/hairline/overlay` 无对应 → `rounded-[Npx]`/`border-ai-line`/`shadow-float`；`primitive-card-pad/footer/bar` → tailwind padding 数值；cubic-bezier 缓动保留内联；演示数据全部提为 props。
- 组件风格约定（同 P1）：文件头块注释（适配自 beautifului XXX + 受控约定 + 剥离了什么）、命名导出 + Props 导出 + 文件尾 default 导出、`ai-*` 令牌类、lucide 图标、每组件配 `__tests__` 测试、登记 `components/index.ts`（分组注释格式参照该文件既有 P1 段 L5-24）。
- 准入线：`cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check` 全绿 + 仓库根 `python3 scripts/architecture_guard.py` 通过；vitest 基线 **487 passed / 3 skipped**（P1 完成后实测，见 CHANGELOG Unreleased 段），只允许增加。
- 设计文档：`docs/plans/2026-08-12-beautifului-ai-native-design.md`（§8 P2 范围）；参考组件源码：`.superpowers/sdd/reference/beautifului/`；勘察结论：`.superpowers/sdd/p2-recon-summary.md`。
- Tasks.tsx 被 Task 2/3/4 三个 Task 触及：Task 2 只动 L661-680 筛选条，Task 3 只动 CascadeRewriteDetail（L254-401）内部段落卡，Task 4 只动 TaskRow（L74-252）行外壳与其调用点。三者改动区域互不重叠；Task 3 不得改 CascadeRewriteDetail 的 props 签名（Task 4 的展开区原样复用 TaskDetail → CascadeRewriteDetail 链路）。

---

### Task 1: AiContextCards 组件 + PromptCoverageBar 上下文槽位清单替换（+ 可选 AgencyStudio 黑板卡）

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiContextCards.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiContextCards.test.tsx`
- Modify: `src-frontend/src/components/PromptCoverageBar.tsx`（槽位徽章清单 L67-84；import 区 L6）
- Modify（可选 Step 6）: `src-frontend/src/pages/AgencyStudio.tsx`（黑板 BoardItem L337-349）

**Interfaces:**
- Consumes: P1 令牌 `bg-ai-surface` / `bg-ai-inset` / `text-ai-ink` / `text-ai-ink-2` / `text-ai-ink-3` / `border-ai-line` / `animate-fade-in`（既有 fadeIn 0.4s）/ `animate-ai-fade-up` / `animate-pop-in`；`cn`（`@/utils/cn`）
- Produces:
  - `export interface AiContextCardSource { label: string; badge: string; tone?: 'green' | 'red' | 'orange' | 'accent' | 'neutral' }`
  - `export interface AiContextCardItem { key: string; title: string; meta?: string; body?: string; source?: AiContextCardSource }`
  - `export interface AiContextCardsProps { title: string; count?: number; items: AiContextCardItem[]; className?: string }`
  - `export function AiContextCards(props: AiContextCardsProps): JSX.Element`；`data-testid="ai-context-cards"`
- 剥离：参考实现 CHUNKS 演示数据、chipsShown 700ms 定时器（source chip 入场改为纯 CSS `animate-pop-in` + 内联 animationDelay 错峰）；内联 SVG 图标改 lucide `AlignLeft` / `ArrowUpRight`；`max-w-95` 演示宽度限制删除（宽度由宿主决定）。

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiContextCards.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AiContextCards, type AiContextCardItem } from '../AiContextCards';

const items: AiContextCardItem[] = [
  {
    key: 'a',
    title: '合同红线',
    meta: '290 字',
    body: '冷链接入前必须完成资质核验。',
    source: { label: 'worldbuilding.md', badge: 'MD', tone: 'green' },
  },
  { key: 'b', title: '角色', source: { label: '未注入', badge: '✗', tone: 'neutral' } },
];

describe('AiContextCards', () => {
  it('渲染标题与计数徽章', () => {
    render(<AiContextCards title="上下文槽位" count={7} items={items} />);
    expect(screen.getByText('上下文槽位')).toBeInTheDocument();
    expect(screen.getByText('7')).toBeInTheDocument();
  });

  it('渲染每张卡的标题 / meta / body', () => {
    render(<AiContextCards title="t" items={items} />);
    expect(screen.getByText('合同红线')).toBeInTheDocument();
    expect(screen.getByText('290 字')).toBeInTheDocument();
    expect(screen.getByText('冷链接入前必须完成资质核验。')).toBeInTheDocument();
    expect(screen.getByText('角色')).toBeInTheDocument();
  });

  it('无 count 时不渲染计数徽章；无 body 的卡不渲染正文段', () => {
    render(<AiContextCards title="t" items={items} />);
    // items[1] 无 body：只有一段正文
    expect(screen.getAllByText(/冷链接入/)).toHaveLength(1);
  });

  it('source chip 渲染 badge/label 且错峰 animationDelay 递增', () => {
    render(<AiContextCards title="t" items={items} />);
    const chipA = screen.getByText('worldbuilding.md').closest('span')!;
    const chipB = screen.getByText('未注入').closest('span')!;
    expect(screen.getByText('MD')).toBeInTheDocument();
    expect(chipA.style.animationDelay).toBe('400ms');
    expect(chipB.style.animationDelay).toBe('480ms');
  });

  it('tone 映射到 --ai-* 变量（neutral → --ai-ink-3）', () => {
    render(<AiContextCards title="t" items={items} />);
    const badge = screen.getByText('✗');
    expect(badge.style.background).toContain('var(--ai-ink-3)');
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiContextCards.test.tsx`
Expected: FAIL（组件不存在，import 报错）

- [ ] **Step 3: Write implementation**

**(a) `src-frontend/src/components/ui/ai/AiContextCards.tsx`** — 新建：

```tsx
/**
 * AiContextCards — 检索上下文卡片列表（适配自 beautifului ContextCards）
 *
 * 受控约定：title/count/items 全部由调用方提供，组件不含演示数据；
 * 剥离参考实现：CHUNKS 演示数据、chipsShown 700ms 定时器
 * （source chip 入场改为纯 CSS animate-pop-in + 内联 animationDelay 错峰）、
 * max-w-95 演示宽度限制；内联 SVG 图标改 lucide-react。
 * 站点私有类已替换：rounded-card → rounded-[12px]、shadow-card → border-ai-line、
 * primitive-card-bar → px-3 py-2、shadow-btn → border-ai-line。
 */
import { AlignLeft, ArrowUpRight } from 'lucide-react';
import { cn } from '@/utils/cn';

export interface AiContextCardSource {
  label: string;
  badge: string;
  tone?: 'green' | 'red' | 'orange' | 'accent' | 'neutral';
}

export interface AiContextCardItem {
  key: string;
  title: string;
  meta?: string;
  body?: string;
  source?: AiContextCardSource;
}

export interface AiContextCardsProps {
  title: string;
  count?: number;
  items: AiContextCardItem[];
  className?: string;
}

const TONE_BG: Record<NonNullable<AiContextCardSource['tone']>, string> = {
  green: 'var(--ai-green)',
  red: 'var(--ai-red)',
  orange: 'var(--ai-orange)',
  accent: 'var(--ai-accent)',
  neutral: 'var(--ai-ink-3)',
};

export function AiContextCards({ title, count, items, className }: AiContextCardsProps) {
  return (
    <div className={cn('flex w-full flex-col gap-2', className)} data-testid="ai-context-cards">
      <div className="animate-fade-in flex items-center gap-2 px-0.5">
        <span className="text-[13px] font-semibold text-ai-ink">{title}</span>
        {typeof count === 'number' && (
          <span className="inline-flex h-5 items-center rounded-md border border-ai-line bg-ai-inset px-1.5 text-[11.5px] font-medium text-ai-ink-2 tabular-nums">
            {count}
          </span>
        )}
      </div>

      {items.map((item, i) => (
        <div
          key={item.key}
          className="animate-ai-fade-up overflow-hidden rounded-[12px] border border-ai-line bg-ai-surface"
          style={{ animationDelay: `${i * 100}ms` }}
        >
          <div className="flex items-center gap-2.5 border-b border-ai-line px-3 py-2">
            <span className="flex min-w-0 items-center gap-1.5 text-[13px] font-medium text-ai-ink">
              <AlignLeft size={11} strokeWidth={2.5} aria-hidden className="shrink-0" />
              <span className="truncate">{item.title}</span>
            </span>
            {item.meta && (
              <span className="ml-auto shrink-0 text-[12px] text-ai-ink-3 tabular-nums">
                {item.meta}
              </span>
            )}
          </div>
          {item.body && (
            <p className="px-3 pt-2 pb-1 text-[12.5px] leading-relaxed text-ai-ink-2">
              {item.body}
            </p>
          )}
          {item.source && (
            <div className={cn('px-3 pb-3', item.body ? 'pt-1' : 'pt-2')}>
              <span
                className="animate-pop-in inline-flex h-6 items-center gap-1.5 rounded-full border border-ai-line bg-ai-inset px-2 text-[12px] font-medium text-ai-ink-2 transition-colors duration-300 hover:bg-ai-hover"
                style={{ animationDelay: `${400 + i * 80}ms` }}
              >
                <span
                  className="flex size-3.5 items-center justify-center rounded-[4px] text-[7px] font-bold text-white"
                  style={{ background: TONE_BG[item.source.tone ?? 'neutral'] }}
                >
                  {item.source.badge}
                </span>
                {item.source.label}
                <ArrowUpRight size={9} strokeWidth={2.5} aria-hidden />
              </span>
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

export default AiContextCards;
```

**(b) `PromptCoverageBar.tsx`**：import 区（L6 `import { cn } from '@/utils/cn';` 行后）加：

```tsx
import { AiContextCards } from '@/components/ui/ai/AiContextCards';
```

槽位徽章清单（现状 L67-84）：

```tsx
      <div className="flex flex-wrap gap-1.5">
        {SLOT_LABELS.map(slot => {
          const on = isFilled(details, slot.key);
          return (
            <span
              key={slot.key}
              className={cn(
                'text-[10px] px-1.5 py-0.5 rounded border',
                on
                  ? 'bg-emerald-500/15 text-emerald-400 border-emerald-500/30'
                  : 'bg-cinema-800/50 text-gray-600 border-cinema-700'
              )}
            >
              {slot.label}
            </span>
          );
        })}
      </div>
```

替换为（数据零改造：SLOT_LABELS / isFilled 原样复用，on/off 映射为 source chip 的 badge/tone）：

```tsx
      <AiContextCards
        title="上下文槽位"
        count={filled}
        items={SLOT_LABELS.map(slot => {
          const on = isFilled(details, slot.key);
          return {
            key: slot.key,
            title: slot.label,
            source: {
              label: on ? '已注入 prompt' : '未注入',
              badge: on ? '✓' : '✗',
              tone: on ? ('green' as const) : ('neutral' as const),
            },
          };
        })}
      />
```

替换后 `cn` import 若不再使用则一并删除（本文件其余位置不用 cn——替换前仅槽位徽章一处使用）。既有测试 `src/components/__tests__/PromptCoverageBar.test.tsx` 断言的 `合同红线`/`KG摘要` 文本由卡片标题继续提供，`2/10` 由未改动的头部提供，应不回归。

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiContextCards.test.tsx src/components/__tests__/PromptCoverageBar.test.tsx && npx tsc --noEmit`
Expected: AiContextCards 5 passed；PromptCoverageBar 3 passed 不回归；tsc 干净

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiContextCards.tsx src-frontend/src/components/ui/ai/__tests__/AiContextCards.test.tsx src-frontend/src/components/PromptCoverageBar.tsx
git commit -m "feat: AiContextCards 组件入库并替换 PromptCoverageBar 槽位清单（P2 Task1）"
```

- [ ] **Step 6（可选，时间允许时做，独立 commit）: AgencyStudio 黑板 BoardItem 接入**

`AgencyStudio.tsx` 黑板区（现状 L332-352）：每个 zone 的 `rounded border p-3` 容器内，将标题行 + 条目列表（L335-349）：

```tsx
              <div key={z.key} className="rounded border p-3">
                <div className="mb-2 text-sm font-medium text-gray-500">{z.name}</div>
                {byZone(z.key).length === 0 && <p className="text-xs text-gray-400">（空）</p>}
                <div className="space-y-2">
                  {byZone(z.key).map(item => (
                    <div key={item.id} className="rounded bg-gray-50 p-2 text-sm">
                      <div className="flex items-center justify-between gap-2">
                        <span className="font-medium">{item.key}</span>
                        <span className="text-xs text-gray-400">
                          v{item.version} · {item.status}
                        </span>
                      </div>
                      <div className="truncate text-xs text-gray-500">{item.summary}</div>
                    </div>
                  ))}
                </div>
              </div>
```

替换为：

```tsx
              <div key={z.key} className="rounded border p-3">
                {byZone(z.key).length === 0 ? (
                  <>
                    <div className="mb-2 text-sm font-medium text-gray-500">{z.name}</div>
                    <p className="text-xs text-gray-400">（空）</p>
                  </>
                ) : (
                  <AiContextCards
                    title={z.name}
                    count={byZone(z.key).length}
                    items={byZone(z.key).map(item => ({
                      key: item.id,
                      title: item.key,
                      meta: `v${item.version} · ${item.status}`,
                      body: item.summary,
                    }))}
                  />
                )}
              </div>
```

import 区加 `import { AiContextCards } from '@/components/ui/ai/AiContextCards';`。注意 AgencyStudio 是浅色后台页（`text-gray-*` 直写），AiContextCards 走 `--ai-*` 幕后令牌（tokens.css 深色系）会与周边浅色卡片形成对比——这是本批可接受的风格切口，后续 P3 再统一后台页令牌。

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | tail -3`
Commit: `git add src-frontend/src/pages/AgencyStudio.tsx && git commit -m "feat: AgencyStudio 黑板条目接入 AiContextCards（P2 Task1 可选）"`

---

### Task 2: AiToolChips 组件 + Tasks 状态筛选条 + Skills 分类筛选条

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiToolChips.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiToolChips.test.tsx`
- Modify: `src-frontend/src/pages/Tasks.tsx`（筛选条 L661-680）
- Modify: `src-frontend/src/pages/Skills.tsx`（分类条 L277-288；categories 定义 L32、selectedCategory L61 不动）

**Interfaces:**
- Consumes: P1 令牌与 `animate-pop-in`；`cn`
- Produces:
  - `export interface AiToolChipItem { key: string; label: string; count?: number; mono?: boolean }`
  - `export interface AiToolChipsProps { items: AiToolChipItem[]; activeKey: string; onSelect: (key: string) => void; ariaLabel?: string; className?: string }`
  - `export function AiToolChips(props: AiToolChipsProps): JSX.Element`；`data-testid="ai-tool-chips"`；`role="radiogroup"` / 每 chip `role="radio"` + `aria-checked`
- 说明（重要）：参考实现 ToolChips.tsx 的本体是「工具调用行 + 展开明细 + diff chips」的运行轨迹演示，与本批两个集成点（单选筛选条）语义不符。经勘察确认，本组件**提取其 chip 视觉语法**（rounded-full chip、hover 态、pop-in 交错入场、mono/tabular-nums 细节、active 实心 `bg-ink` 反白）做受控单选 chips 组；工具调用行/展开明细/diff 语法不落（ AgencyStudio 活动流已由 P1 AiThinking 覆盖）。

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiToolChips.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiToolChips } from '../AiToolChips';

const items = [
  { key: 'all', label: '全部' },
  { key: 'running', label: '执行中', count: 3 },
  { key: 'failed', label: '失败', mono: true },
];

describe('AiToolChips', () => {
  it('渲染全部 chips，radiogroup 可访问名生效', () => {
    render(<AiToolChips ariaLabel="任务状态筛选" items={items} activeKey="all" onSelect={() => {}} />);
    expect(screen.getByRole('radiogroup', { name: '任务状态筛选' })).toBeInTheDocument();
    expect(screen.getAllByRole('radio')).toHaveLength(3);
  });

  it('activeKey 对应 chip aria-checked=true，其余 false', () => {
    render(<AiToolChips items={items} activeKey="running" onSelect={() => {}} />);
    expect(screen.getByRole('radio', { name: /执行中/ })).toHaveAttribute('aria-checked', 'true');
    expect(screen.getByRole('radio', { name: '全部' })).toHaveAttribute('aria-checked', 'false');
  });

  it('点击调用 onSelect(key)', () => {
    const onSelect = vi.fn();
    render(<AiToolChips items={items} activeKey="all" onSelect={onSelect} />);
    fireEvent.click(screen.getByRole('radio', { name: /失败/ }));
    expect(onSelect).toHaveBeenCalledWith('failed');
  });

  it('count 以 tabular-nums 徽章渲染', () => {
    render(<AiToolChips items={items} activeKey="all" onSelect={() => {}} />);
    const badge = screen.getByText('3');
    expect(badge.className).toContain('tabular-nums');
  });

  it('active 为实心反白（bg-ai-ink），inactive 带 border-ai-line', () => {
    render(<AiToolChips items={items} activeKey="all" onSelect={() => {}} />);
    expect(screen.getByRole('radio', { name: '全部' }).className).toContain('bg-ai-ink');
    expect(screen.getByRole('radio', { name: /失败/ }).className).toContain('border-ai-line');
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiToolChips.test.tsx`
Expected: FAIL（组件不存在）

- [ ] **Step 3: Write implementation**

**(a) `src-frontend/src/components/ui/ai/AiToolChips.tsx`** — 新建：

```tsx
/**
 * AiToolChips — 单选筛选 chips 组（提取自 beautifului ToolChips 的 chip 视觉语法）
 *
 * 受控约定：items/activeKey/onSelect 全部由调用方提供；剥离参考实现的
 * ROWS/DIFFS 演示数据、STEP_MS 自运行步进、工具调用行展开明细（与本批
 * 筛选条集成点语义不符，轨迹场景已由 P1 AiThinking 覆盖）。
 * active 实心反白取自参考 primary 样式（bg-ink text-canvas → bg-ai-ink text-ai-surface）。
 */
import { cn } from '@/utils/cn';

export interface AiToolChipItem {
  key: string;
  label: string;
  count?: number;
  mono?: boolean;
}

export interface AiToolChipsProps {
  items: AiToolChipItem[];
  activeKey: string;
  onSelect: (key: string) => void;
  ariaLabel?: string;
  className?: string;
}

export function AiToolChips({ items, activeKey, onSelect, ariaLabel, className }: AiToolChipsProps) {
  return (
    <div
      role="radiogroup"
      aria-label={ariaLabel}
      className={cn('flex flex-wrap gap-1.5', className)}
      data-testid="ai-tool-chips"
    >
      {items.map((item, i) => {
        const active = item.key === activeKey;
        return (
          <button
            key={item.key}
            type="button"
            role="radio"
            aria-checked={active}
            onClick={() => onSelect(item.key)}
            className={cn(
              'animate-pop-in inline-flex h-7 items-center gap-1 rounded-full px-2.5 text-[12px] font-medium transition-[background-color,color,transform] duration-150 active:scale-[0.96]',
              item.mono && 'font-mono',
              active
                ? 'bg-ai-ink text-ai-surface'
                : 'border border-ai-line bg-ai-surface text-ai-ink-2 hover:bg-ai-hover'
            )}
            style={{ animationDelay: `${i * 60}ms` }}
          >
            {item.label}
            {typeof item.count === 'number' && (
              <span className="text-[11px] tabular-nums opacity-70">{item.count}</span>
            )}
          </button>
        );
      })}
    </div>
  );
}

export default AiToolChips;
```

**(b) `Tasks.tsx`**：import 区（L19 `import { cn } from '@/utils/cn';` 行后）加：

```tsx
import { AiToolChips } from '@/components/ui/ai/AiToolChips';
```

筛选条（现状 L661-680）：

```tsx
      <div className="flex gap-1 mb-4">
        {(['all', 'running', 'pending', 'completed', 'failed'] as StatusFilter[]).map(f => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={cn(
              'px-3 py-1.5 rounded-lg text-xs font-medium transition-colors',
              filter === f
                ? 'bg-cinema-gold/20 text-cinema-gold'
                : 'text-gray-500 hover:text-gray-300 hover:bg-cinema-800/50'
            )}
          >
            {f === 'all' && '全部'}
            {f === 'running' && '执行中'}
            {f === 'pending' && '等待中'}
            {f === 'completed' && '已完成'}
            {f === 'failed' && '失败'}
          </button>
        ))}
      </div>
```

替换为（`StatusFilter` 类型 L36 不动）：

```tsx
      <div className="mb-4">
        <AiToolChips
          ariaLabel="任务状态筛选"
          activeKey={filter}
          onSelect={key => setFilter(key as StatusFilter)}
          items={[
            { key: 'all', label: '全部' },
            { key: 'running', label: '执行中' },
            { key: 'pending', label: '等待中' },
            { key: 'completed', label: '已完成' },
            { key: 'failed', label: '失败' },
          ]}
        />
      </div>
```

**(c) `Skills.tsx`**：import 区加（放在既有 `@/components/ui/...` import 附近）：

```tsx
import { AiToolChips } from '@/components/ui/ai/AiToolChips';
```

分类条（现状 L277-288）：

```tsx
      <div className="flex flex-wrap gap-2">
        {categories.map(cat => (
          <Button
            key={cat.id}
            variant={selectedCategory === cat.id ? 'primary' : 'secondary'}
            size="sm"
            onClick={() => setSelectedCategory(cat.id)}
          >
            {cat.label}
          </Button>
        ))}
      </div>
```

替换为（categories 定义 L32、selectedCategory state L61 不动；`SkillCategory | 'all'` 类型不变）：

```tsx
      <AiToolChips
        ariaLabel="技能分类筛选"
        activeKey={selectedCategory}
        onSelect={key => setSelectedCategory(key as SkillCategory | 'all')}
        items={categories.map(cat => ({ key: cat.id, label: cat.label }))}
      />
```

替换后若 `Button` 在 Skills.tsx 仍被「导入技能」按钮（L243-246）使用则保留 import（当前在用，保留）。

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiToolChips.test.tsx && npx tsc --noEmit`
Expected: 5 passed；tsc 干净（Tasks/Skills 无既有页面测试，无需回归断言）

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiToolChips.tsx src-frontend/src/components/ui/ai/__tests__/AiToolChips.test.tsx src-frontend/src/pages/Tasks.tsx src-frontend/src/pages/Skills.tsx
git commit -m "feat: AiToolChips 组件入库并替换 Tasks/Skills 筛选条（P2 Task2）"
```

---

### Task 3: AiRecommendationCard 组件 + CascadeRewriteDetail 逐段接受/拒绝卡片

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiRecommendationCard.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiRecommendationCard.test.tsx`
- Modify: `src-frontend/src/pages/Tasks.tsx`（CascadeRewriteDetail 段落卡 L347-398 内部；函数 props 签名 L254 不动——Task 4 依赖）

**Interfaces:**
- Consumes: P1 令牌与 `animate-fade-in`；`cn`
- Produces:
  - `export interface AiRecommendationOption { key: string; body: React.ReactNode; short: string; signal: 0 | 1 | 2 | 3; label: string }`
  - `export interface AiRecommendationCardProps { title: string; options: AiRecommendationOption[]; status?: 'pending' | 'accepted' | 'rejected'; acceptLabel?: string; rejectLabel?: string; alternativesLabel?: string; onAccept: (key: string) => void; onReject?: (key: string) => void; className?: string }`
  - `export function AiRecommendationCard(props): JSX.Element`；`data-testid="ai-recommendation-card"`
- 受控化映射：参考实现内部 `accepted` state → `status` prop（调用方持有决策结果）；`selected`/`open` 为纯视图状态保留组件内部；`ctaStyle`/`tone` 字符串 → 按 signal 推导（3 → `--ai-green`，1-2 → `--ai-orange`，0 → `--ai-ink-3`）；`primitive-card-pad/footer` → `p-3` / `px-3 py-2`；`green-tint` 等 → `bg-ai-*/10`；`options.length > 1` 时才渲染 Alternatives 抽屉按钮（级联改写场景恒为单选项，抽屉不出现但组件能力保留）。

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiRecommendationCard.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiRecommendationCard, type AiRecommendationOption } from '../AiRecommendationCard';

const twoOptions: AiRecommendationOption[] = [
  { key: 'a', body: <p>方案 A 正文</p>, short: '方案 A', signal: 3, label: '高置信' },
  { key: 'b', body: <p>方案 B 正文</p>, short: '方案 B', signal: 1, label: '需复核' },
];

describe('AiRecommendationCard', () => {
  it('渲染标题与当前选项 body / 信号标签', () => {
    render(
      <AiRecommendationCard title="段落 3：时序矛盾" options={twoOptions} onAccept={() => {}} />
    );
    expect(screen.getByText('段落 3：时序矛盾')).toBeInTheDocument();
    expect(screen.getByText('方案 A 正文')).toBeInTheDocument();
    expect(screen.getAllByText('高置信').length).toBeGreaterThan(0);
  });

  it('signal 渲染 3 根信号条', () => {
    const { container } = render(
      <AiRecommendationCard title="t" options={twoOptions} onAccept={() => {}} />
    );
    expect(container.querySelectorAll('[data-testid="ai-rec-meter"] > span')).toHaveLength(3);
  });

  it('点击接受调用 onAccept(key)，点击拒绝调用 onReject(key)', () => {
    const onAccept = vi.fn();
    const onReject = vi.fn();
    render(
      <AiRecommendationCard title="t" options={twoOptions} onAccept={onAccept} onReject={onReject} />
    );
    fireEvent.click(screen.getByRole('button', { name: '接受' }));
    expect(onAccept).toHaveBeenCalledWith('a');
    fireEvent.click(screen.getByRole('button', { name: '拒绝' }));
    expect(onReject).toHaveBeenCalledWith('a');
  });

  it('Alternatives 抽屉切换到方案 B 后正文与接受键随之更新', () => {
    const onAccept = vi.fn();
    render(<AiRecommendationCard title="t" options={twoOptions} onAccept={onAccept} />);
    fireEvent.click(screen.getByRole('button', { name: '备选' }));
    fireEvent.click(screen.getByRole('button', { name: /方案 B/ }));
    expect(screen.getByText('方案 B 正文')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: '接受' }));
    expect(onAccept).toHaveBeenCalledWith('b');
  });

  it('status=accepted 时接受按钮变为已接受并禁用；status=rejected 时显示已拒绝', () => {
    const { rerender } = render(
      <AiRecommendationCard title="t" options={twoOptions} status="accepted" onAccept={() => {}} />
    );
    expect(screen.getByRole('button', { name: '已接受' })).toBeDisabled();
    rerender(
      <AiRecommendationCard title="t" options={twoOptions} status="rejected" onAccept={() => {}} />
    );
    expect(screen.getByText('已拒绝')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: '接受' })).not.toBeInTheDocument();
  });

  it('单选项时不渲染 Alternatives 按钮', () => {
    render(
      <AiRecommendationCard title="t" options={[twoOptions[0]]} onAccept={() => {}} />
    );
    expect(screen.queryByRole('button', { name: '备选' })).not.toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiRecommendationCard.test.tsx`
Expected: FAIL（组件不存在）

- [ ] **Step 3: Write implementation**

**(a) `src-frontend/src/components/ui/ai/AiRecommendationCard.tsx`** — 新建：

```tsx
/**
 * AiRecommendationCard — AI 建议确认卡（适配自 beautifului RecommendationCard）
 *
 * 受控约定：title/options/status/回调全部由调用方提供；剥离参考实现的
 * OPTIONS 演示数据与内部 accepted state（改为 status prop：pending/accepted/rejected）。
 * selected/open 为纯视图状态保留内部（同 P1 AiThinking manualExpanded 先例）。
 * 移植说明：tone/ctaStyle 字符串改为按 signal 推导（3→--ai-green，1-2→--ai-orange，
 * 0→--ai-ink-3）；primitive-card-pad/footer → p-3 / px-3 py-2；shadow-card →
 * border-ai-line；shadow-btn 删除（用 border 替代）；缓动 cubic-bezier 保留内联。
 * 拒绝按钮为本组件相对参考实现的新增（级联改写场景需要 接受/拒绝 双动作）。
 */
import { useState } from 'react';
import { cn } from '@/utils/cn';

export interface AiRecommendationOption {
  key: string;
  body: React.ReactNode;
  short: string;
  signal: 0 | 1 | 2 | 3;
  label: string;
}

export interface AiRecommendationCardProps {
  title: string;
  options: AiRecommendationOption[];
  status?: 'pending' | 'accepted' | 'rejected';
  acceptLabel?: string;
  rejectLabel?: string;
  alternativesLabel?: string;
  onAccept: (key: string) => void;
  onReject?: (key: string) => void;
  className?: string;
}

function signalTone(signal: number): string {
  if (signal >= 3) return 'var(--ai-green)';
  if (signal >= 1) return 'var(--ai-orange)';
  return 'var(--ai-ink-3)';
}

function Meter({ signal }: { signal: number }) {
  return (
    <span className="flex items-end gap-0.5" data-testid="ai-rec-meter" aria-hidden>
      {[0, 1, 2].map(bar => (
        <span
          key={bar}
          className="w-1 rounded-full transition-colors duration-300"
          style={{
            height: 10,
            background: bar < signal ? signalTone(signal) : 'var(--ai-line-strong)',
          }}
        />
      ))}
    </span>
  );
}

export function AiRecommendationCard({
  title,
  options,
  status = 'pending',
  acceptLabel = '接受',
  rejectLabel = '拒绝',
  alternativesLabel = '备选',
  onAccept,
  onReject,
  className,
}: AiRecommendationCardProps) {
  const [selected, setSelected] = useState(0);
  const [open, setOpen] = useState(false);

  const active = options[Math.min(selected, options.length - 1)];
  const others = options.map((o, i) => ({ o, i })).filter(({ i }) => i !== selected);

  return (
    <div
      className={cn(
        'w-full overflow-hidden rounded-[12px] border border-ai-line bg-ai-surface',
        className
      )}
      data-testid="ai-recommendation-card"
    >
      <div className="p-3">
        <span className="text-[13px] font-semibold text-ai-ink">{title}</span>
        <div key={active.key} className="animate-fade-in mt-1.5 text-[13px] leading-relaxed text-ai-ink-2">
          {active.body}
        </div>
      </div>

      {/* alternatives 抽屉（仅多选项时出现） */}
      {options.length > 1 && (
        <div
          className="grid transition-[grid-template-rows,opacity] duration-300"
          style={{
            gridTemplateRows: open ? '1fr' : '0fr',
            opacity: open ? 1 : 0,
            transitionTimingFunction: 'cubic-bezier(0.16, 1, 0.3, 1)',
          }}
        >
          <div className="overflow-hidden">
            <div className="border-t border-ai-line bg-ai-inset px-2 py-2">
              <p className="px-1.5 pb-1 text-[11px] font-medium text-ai-ink-3">其他选项</p>
              {others.map(({ o, i }) => (
                <button
                  key={o.key}
                  type="button"
                  onClick={() => {
                    setSelected(i);
                    setOpen(false);
                  }}
                  className="flex w-full items-center gap-2.5 rounded-[8px] px-1.5 py-1.5 text-left transition-colors duration-100 hover:bg-ai-hover"
                >
                  <Meter signal={o.signal} />
                  <span className="min-w-0 flex-1 truncate text-[12.5px] text-ai-ink">{o.short}</span>
                  <span className="shrink-0 text-[11px] text-ai-ink-3">{o.label}</span>
                </button>
              ))}
            </div>
          </div>
        </div>
      )}

      <div className="flex items-center justify-between gap-3 border-t border-ai-line bg-ai-inset px-3 py-2">
        <span className="flex items-center gap-2">
          <Meter signal={active.signal} />
          <span className="text-[12.5px] font-medium text-ai-ink-2">{active.label}</span>
        </span>

        <span className="-mr-0.5 flex items-center gap-2">
          {status === 'pending' && options.length > 1 && (
            <button
              type="button"
              aria-expanded={open}
              onClick={() => setOpen(current => !current)}
              className={cn(
                'h-7 rounded-[8px] px-2.5 text-[12.5px] font-medium transition-[background-color,transform] duration-100 active:scale-[0.96]',
                open ? 'bg-ai-hover text-ai-ink' : 'bg-ai-surface text-ai-ink hover:bg-ai-hover'
              )}
            >
              {alternativesLabel}
            </button>
          )}
          {status === 'pending' && onReject && (
            <button
              type="button"
              onClick={() => onReject(active.key)}
              className="h-7 rounded-[8px] border border-ai-line bg-ai-surface px-2.5 text-[12.5px] font-medium text-ai-ink-2 transition-[background-color,transform] duration-100 hover:bg-ai-hover active:scale-[0.96]"
            >
              {rejectLabel}
            </button>
          )}
          {status === 'rejected' && (
            <span className="text-[12.5px] font-medium text-ai-ink-3">已拒绝</span>
          )}
          {status !== 'rejected' && (
            <button
              type="button"
              disabled={status !== 'pending'}
              onClick={() => onAccept(active.key)}
              className={cn(
                'h-7 rounded-[8px] px-3 text-[12.5px] font-medium transition-[background-color,transform] duration-150 active:scale-[0.96] disabled:cursor-default',
                status === 'accepted' ? 'bg-ai-green text-white' : 'bg-ai-ink text-ai-surface hover:opacity-90'
              )}
            >
              {status === 'accepted' ? '已接受' : acceptLabel}
            </button>
          )}
        </span>
      </div>
    </div>
  );
}

export default AiRecommendationCard;
```

**(b) `Tasks.tsx` CascadeRewriteDetail**：import 区（Task 2 已加的 `import { AiToolChips } ...` 行后）加：

```tsx
import { AiRecommendationCard } from '@/components/ui/ai/AiRecommendationCard';
```

段落卡列表（现状 L347-398）：

```tsx
      <div className="space-y-2 max-h-96 overflow-y-auto">
        {result.segments.map((segment, idx) => (
          <div key={idx} className="p-2 bg-cinema-900 rounded border border-cinema-700">
            <div className="flex items-center justify-between mb-1">
              <span className="text-[10px] text-gray-500">
                场景 {segment.scene_id.slice(0, 8)}... · 段落 {segment.paragraph_index + 1}
              </span>
              <span
                className={cn(
                  'text-[10px] px-1.5 py-0.5 rounded',
                  segment.user_decision === 'pending' && 'bg-yellow-500/20 text-yellow-400',
                  segment.user_decision === 'accepted' && 'bg-green-500/20 text-green-400',
                  segment.user_decision === 'rejected' && 'bg-gray-500/20 text-gray-400'
                )}
              >
                {segment.user_decision === 'pending' && '待确认'}
                {segment.user_decision === 'accepted' && '已接受'}
                {segment.user_decision === 'rejected' && '已拒绝'}
              </span>
            </div>
            <p className="text-[10px] text-cinema-gold mb-1">{segment.change_reason}</p>
            <div className="grid grid-cols-2 gap-2">
              <div className="p-1.5 bg-red-500/10 rounded">
                <p className="text-[10px] text-red-400 mb-0.5">原文</p>
                <p className="text-[11px] text-gray-400 line-through">{segment.original_text}</p>
              </div>
              <div className="p-1.5 bg-green-500/10 rounded">
                <p className="text-[10px] text-green-400 mb-0.5">改写</p>
                <p className="text-[11px] text-gray-300">{segment.rewritten_text}</p>
              </div>
            </div>
            {segment.user_decision === 'pending' && (
              <div className="flex gap-2 mt-1.5">
                <button
                  onClick={() => handleAccept(idx)}
                  disabled={applyMutation.isPending}
                  className="px-2 py-0.5 text-[10px] bg-green-500/20 text-green-400 rounded hover:bg-green-500/30 transition-colors disabled:opacity-50"
                >
                  接受
                </button>
                <button
                  onClick={() => handleReject(idx)}
                  disabled={rejectMutation.isPending}
                  className="px-2 py-0.5 text-[10px] bg-gray-500/20 text-gray-400 rounded hover:bg-gray-500/30 transition-colors disabled:opacity-50"
                >
                  拒绝
                </button>
              </div>
            )}
          </div>
        ))}
      </div>
```

替换为（语义 1:1：每段 = 一张单选项推荐卡；`user_decision` 直通 `status` prop；头部「级联改写预览 / 全部接受 / 全部拒绝」（L313-345）与 warnings 块不动；原文/改写双栏保留为卡 body，颜色改 `--ai-*` 直引以保持卡内令牌一致）：

```tsx
      <div className="space-y-2 max-h-96 overflow-y-auto">
        {result.segments.map((segment, idx) => (
          <AiRecommendationCard
            key={idx}
            title={`场景 ${segment.scene_id.slice(0, 8)}… · 段落 ${segment.paragraph_index + 1}：${segment.change_reason}`}
            status={segment.user_decision}
            options={[
              {
                key: String(idx),
                short: segment.change_reason,
                signal: 0,
                label: 'AI 改写建议',
                body: (
                  <div className="grid grid-cols-2 gap-2">
                    <div className="rounded-[8px] bg-ai-red/10 p-1.5">
                      <p className="mb-0.5 text-[10px] text-ai-red">原文</p>
                      <p className="text-[11px] text-ai-ink-3 line-through">{segment.original_text}</p>
                    </div>
                    <div className="rounded-[8px] bg-ai-green/10 p-1.5">
                      <p className="mb-0.5 text-[10px] text-ai-green">改写</p>
                      <p className="text-[11px] text-ai-ink">{segment.rewritten_text}</p>
                    </div>
                  </div>
                ),
              },
            ]}
            onAccept={() => handleAccept(idx)}
            onReject={() => handleReject(idx)}
          />
        ))}
      </div>
```

说明：`signal: 0` + `label: 'AI 改写建议'`——后端未提供置信度，信号条恒为中性（`--ai-ink-3`），不虚构数据；`applyMutation.isPending`/`rejectMutation.isPending` 的按钮禁用态由 status 仍 pending 时的短暂窗口承担，不再单独禁用（mutation 完成后 `user_decision` 翻转为 accepted/rejected，卡片自动进入已决态）。`segment.user_decision` 类型为 `'pending' | 'accepted' | 'rejected'`（`RewriteSegment`，`@/hooks/useTasks`），与 status prop 类型一致，无需断言。

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiRecommendationCard.test.tsx && npx tsc --noEmit`
Expected: 6 passed；tsc 干净

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiRecommendationCard.tsx src-frontend/src/components/ui/ai/__tests__/AiRecommendationCard.test.tsx src-frontend/src/pages/Tasks.tsx
git commit -m "feat: AiRecommendationCard 组件入库并替换级联改写逐段确认卡（P2 Task3）"
```

---

### Task 4: AiTaskRows 组件 + Tasks.tsx TaskRow 行外壳替换

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiTaskRows.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiTaskRows.test.tsx`
- Modify: `src-frontend/src/pages/Tasks.tsx`（TaskRow L74-252 行外壳；调用点 L710-775 不动；列表容器 L700-701 样式微调）

**Interfaces:**
- Consumes: P1 令牌与 `animate-ai-fade-up` / `animate-pop-in` / `animate-ai-spin`（SpinnerRing，reduced-motion 冻结已在 P1 tokens.css/frontstage.css 覆盖）；`cn`；lucide `Check` / `X` / `ChevronDown`
- Produces:
  - `export type AiTaskRowStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'`
  - `export interface AiTaskRowDetail { label: string; meta?: string }`
  - `export interface AiTaskRowItem<T = unknown> { key: string; status: AiTaskRowStatus; progress?: number; index?: number; label: string; meta?: string; pill?: React.ReactNode; trailing?: React.ReactNode; details?: AiTaskRowDetail[]; payload?: T }`
  - `export interface AiTaskRowsProps<T = unknown> { rows: AiTaskRowItem<T>[]; expandedKey?: string | null; onToggle: (key: string) => void; variant?: 'capsules' | 'list'; renderDetail?: (row: AiTaskRowItem<T>) => React.ReactNode; className?: string }`
  - `export function AiTaskRows<T>(props: AiTaskRowsProps<T>): JSX.Element`；`data-testid="ai-task-rows"`
- 受控化映射：参考实现 TICKS/useTick 状态机演示 → `status`/`progress` props；`manualOpen` → `expandedKey`/`onToggle` 受控；行展开内容：`details` 简单明细或 `renderDetail` 自定义（Tasks 集成用后者挂既有 TaskDetail）；`green-tint`/`red-tint` pill → 由调用方经 `pill` 插槽传入（组件不管配色语义）；`spin 1.1s` 内联裸 keyframe → `animate-ai-spin`（700ms，P1 已注册且 reduced-motion 已冻结）。
- 状态徽章内置：completed → 绿底白 Check（pop-in）、failed → 红底白 X、cancelled → 橙底白 X、running → SpinnerRing active（环内显示 `progress` 百分比数字，无 progress 显示 `index`）、pending → SpinnerRing 静态（环内 `index`）。

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiTaskRows.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiTaskRows, type AiTaskRowItem } from '../AiTaskRows';

const rows: AiTaskRowItem[] = [
  { key: 't1', status: 'completed', label: '拆书分析', meta: '一次性', pill: <span>已完成</span> },
  { key: 't2', status: 'running', progress: 45, label: '级联改写', meta: '每天' },
  { key: 't3', status: 'failed', label: '定时审稿', meta: 'cron', index: 3 },
];

describe('AiTaskRows', () => {
  it('渲染行标签 / meta / pill', () => {
    render(<AiTaskRows rows={rows} onToggle={() => {}} />);
    expect(screen.getByText('拆书分析')).toBeInTheDocument();
    expect(screen.getByText('一次性')).toBeInTheDocument();
    expect(screen.getByText('已完成')).toBeInTheDocument();
  });

  it('completed/failed 渲染对应徽章，running 渲染进度环（环内为百分比）', () => {
    const { container } = render(<AiTaskRows rows={rows} onToggle={() => {}} />);
    expect(container.querySelector('[data-testid="ai-task-badge-completed"]')).toBeTruthy();
    expect(container.querySelector('[data-testid="ai-task-badge-failed"]')).toBeTruthy();
    const ring = container.querySelector('[data-testid="ai-task-ring"]');
    expect(ring?.textContent).toBe('45');
    expect(ring?.querySelector('svg')?.classList.contains('animate-ai-spin')).toBe(true);
  });

  it('点击行调用 onToggle(key)，trailing 点击不触发行 toggle', () => {
    const onToggle = vi.fn();
    const onTrailing = vi.fn();
    render(
      <AiTaskRows
        rows={[{ key: 't1', status: 'pending', index: 1, label: 'x', trailing: <button onClick={onTrailing}>执行</button> }]}
        onToggle={onToggle}
      />
    );
    fireEvent.click(screen.getByText('x'));
    expect(onToggle).toHaveBeenCalledWith('t1');
    fireEvent.click(screen.getByText('执行'));
    expect(onTrailing).toHaveBeenCalled();
    expect(onToggle).toHaveBeenCalledTimes(1);
  });

  it('expandedKey 行展开 renderDetail 内容（grid 1fr），其余 0fr', () => {
    const { container } = render(
      <AiTaskRows
        rows={rows}
        expandedKey="t2"
        onToggle={() => {}}
        renderDetail={row => <div>详情-{row.label}</div>}
      />
    );
    expect(screen.getByText('详情-级联改写')).toBeInTheDocument();
    const expanded = screen.getByText('详情-级联改写').closest('[data-testid="ai-task-detail"]')!;
    expect(expanded.style.gridTemplateRows).toBe('1fr');
    expect(container.querySelectorAll('[data-testid="ai-task-detail"]').length).toBe(3);
  });

  it('details 数组在无 renderDetail 时作为默认展开内容', () => {
    render(
      <AiTaskRows
        rows={[{ key: 't1', status: 'completed', label: 'x', details: [{ label: '匹配记录', meta: '12/12' }] }]}
        expandedKey="t1"
        onToggle={() => {}}
      />
    );
    expect(screen.getByText('匹配记录')).toBeInTheDocument();
    expect(screen.getByText('12/12')).toBeInTheDocument();
  });

  it('list 变体行有 border-b，capsules 变体行为独立卡', () => {
    const { container, rerender } = render(<AiTaskRows rows={rows} variant="list" onToggle={() => {}} />);
    expect(container.querySelector('[data-testid="ai-task-rows"] .border-b')).toBeTruthy();
    rerender(<AiTaskRows rows={rows} variant="capsules" onToggle={() => {}} />);
    expect(container.querySelector('[data-testid="ai-task-rows"] .rounded-\\[14px\\], [data-testid="ai-task-rows"] .rounded-\\[22px\\]')).toBeTruthy();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiTaskRows.test.tsx`
Expected: FAIL（组件不存在）

- [ ] **Step 3: Write implementation**

**(a) `src-frontend/src/components/ui/ai/AiTaskRows.tsx`** — 新建：

```tsx
/**
 * AiTaskRows — 任务行列表（适配自 beautifului TaskRows）
 *
 * 受控约定：rows/expandedKey/onToggle 全部由调用方提供；剥离参考实现的
 * TICKS/useTick 状态机演示（failed→done 自动翻转）与 manualOpen 内部展开态。
 * 展开内容两选一：details 简单明细，或 renderDetail 自定义（Tasks 页挂既有 TaskDetail）。
 * 移植说明：spin 内联裸 keyframe → animate-ai-spin（P1 已注册，reduced-motion 已冻结）；
 * green-tint/red-tint pill → pill/trailing 插槽由调用方传入；rounded-card/shadow-card →
 * rounded-[14px]/rounded-[22px] + border-ai-line；variant='List' → 'list'（小写）。
 */
import { Check, ChevronDown, X } from 'lucide-react';
import { cn } from '@/utils/cn';

export type AiTaskRowStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface AiTaskRowDetail {
  label: string;
  meta?: string;
}

export interface AiTaskRowItem<T = unknown> {
  key: string;
  status: AiTaskRowStatus;
  /** 0-100；running 时显示在进度环内 */
  progress?: number;
  /** 环内序号（pending 或无 progress 时） */
  index?: number;
  label: string;
  meta?: string;
  pill?: React.ReactNode;
  /** 行尾操作区（chevron 之前）；组件侧已统一 stopPropagation，点击不触发行 toggle */
  trailing?: React.ReactNode;
  details?: AiTaskRowDetail[];
  payload?: T;
}

export interface AiTaskRowsProps<T = unknown> {
  rows: AiTaskRowItem<T>[];
  expandedKey?: string | null;
  onToggle: (key: string) => void;
  variant?: 'capsules' | 'list';
  renderDetail?: (row: AiTaskRowItem<T>) => React.ReactNode;
  className?: string;
}

function SpinnerRing({ active, children }: { active?: boolean; children?: React.ReactNode }) {
  const size = 24;
  const stroke = 2;
  const r = (size - stroke) / 2;
  const c = 2 * Math.PI * r;
  return (
    <span
      className="relative inline-flex shrink-0 items-center justify-center"
      style={{ width: size, height: size }}
      data-testid="ai-task-ring"
    >
      <svg
        width={size}
        height={size}
        className={cn('absolute inset-0', active && 'animate-ai-spin')}
        aria-hidden
      >
        <circle cx={size / 2} cy={size / 2} r={r} fill="none" stroke="var(--ai-line)" strokeWidth={stroke} />
        {active && (
          <circle
            cx={size / 2}
            cy={size / 2}
            r={r}
            fill="none"
            stroke="var(--ai-ink-3)"
            strokeWidth={stroke}
            strokeLinecap="round"
            strokeDasharray={`${c * 0.28} ${c * 0.72}`}
          />
        )}
      </svg>
      <span className="relative text-[10.5px] font-semibold text-ai-ink tabular-nums">{children}</span>
    </span>
  );
}

function StatusBadge({ status }: { status: AiTaskRowStatus }) {
  if (status === 'completed' || status === 'failed' || status === 'cancelled') {
    const bg =
      status === 'completed'
        ? 'var(--ai-green)'
        : status === 'failed'
          ? 'var(--ai-red)'
          : 'var(--ai-orange)';
    return (
      <span
        className="animate-pop-in flex size-[22px] shrink-0 items-center justify-center rounded-full text-white"
        style={{ background: bg }}
        data-testid={`ai-task-badge-${status}`}
        aria-hidden
      >
        {status === 'completed' ? <Check size={13} strokeWidth={3.5} /> : <X size={12} strokeWidth={3.5} />}
      </span>
    );
  }
  return null;
}

function RowBadge({ row }: { row: AiTaskRowItem }) {
  if (row.status === 'running') {
    return <SpinnerRing active>{row.progress ?? row.index ?? ''}</SpinnerRing>;
  }
  if (row.status === 'pending') {
    return <SpinnerRing>{row.index ?? ''}</SpinnerRing>;
  }
  return <StatusBadge status={row.status} />;
}

export function AiTaskRows<T = unknown>({
  rows,
  expandedKey = null,
  onToggle,
  variant = 'capsules',
  renderDetail,
  className,
}: AiTaskRowsProps<T>) {
  const list = variant === 'list';
  return (
    <div
      className={cn('flex w-full flex-col', list ? 'gap-0' : 'gap-2', className)}
      data-testid="ai-task-rows"
    >
      {rows.map((row, i) => {
        const open = expandedKey === row.key;
        return (
          <div
            key={row.key}
            className={cn(
              'animate-ai-fade-up self-stretch overflow-hidden bg-ai-surface transition-[border-radius] duration-300',
              list
                ? 'border-b border-ai-line last:border-b-0'
                : cn('border border-ai-line', open ? 'rounded-[14px]' : 'rounded-[22px]')
            )}
            style={{ animationDelay: `${i * 80}ms` }}
          >
            <div
              role="button"
              tabIndex={0}
              aria-expanded={open}
              onClick={() => onToggle(row.key)}
              onKeyDown={e => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  onToggle(row.key);
                }
              }}
              className="flex h-11 w-full cursor-pointer items-center gap-2.5 px-2.5 text-left transition-colors duration-100 hover:bg-ai-inset"
            >
              <span className="flex size-6 shrink-0 items-center justify-center">
                <RowBadge row={row} />
              </span>
              <span className="min-w-0 flex-1 truncate text-[13px] font-medium text-ai-ink">
                {row.label}
              </span>
              {row.meta && (
                <span className="shrink-0 text-[12.5px] text-ai-ink-2 tabular-nums">{row.meta}</span>
              )}
              {row.pill}
              {row.trailing && (
                /* trailing 插槽点击不触发行 toggle（组件侧统一拦截） */
                <span className="flex shrink-0 items-center gap-1" onClick={e => e.stopPropagation()}>
                  {row.trailing}
                </span>
              )}
              <span
                aria-hidden="true"
                className="-ml-1 flex size-7 shrink-0 items-center justify-center rounded-full text-ai-ink-3"
              >
                <ChevronDown
                  size={15}
                  strokeWidth={2.2}
                  className="transition-transform duration-300"
                  style={{ transform: open ? 'rotate(180deg)' : 'rotate(0deg)' }}
                />
              </span>
            </div>

            <div
              className="grid transition-[grid-template-rows,opacity] duration-300"
              style={{
                gridTemplateRows: open ? '1fr' : '0fr',
                opacity: open ? 1 : 0,
                transitionTimingFunction: 'cubic-bezier(0.23, 1, 0.32, 1)',
              }}
              data-testid="ai-task-detail"
            >
              <div className="min-h-0 overflow-hidden">
                {renderDetail ? (
                  renderDetail(row)
                ) : (
                  <div className="mb-2.5 grid grid-cols-[24px_1fr] gap-2.5 px-2.5">
                    <span aria-hidden className="mx-auto h-full w-px bg-ai-line" />
                    <div className="flex flex-col gap-1.5">
                      {(row.details ?? []).map((d, j) => (
                        <div
                          key={d.label}
                          className="flex items-center justify-between"
                          style={
                            open
                              ? { animation: `fade-up 300ms cubic-bezier(0.23,1,0.32,1) ${120 + j * 100}ms both` }
                              : undefined
                          }
                        >
                          <span className="text-[12px] text-ai-ink-2">{d.label}</span>
                          {d.meta && (
                            <span className="font-mono text-[11.5px] text-ai-ink-3 tabular-nums">{d.meta}</span>
                          )}
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}

export default AiTaskRows;
```

注意：上面 details 入场动画的 `fade-up` 是**内联裸 keyframes 名**——tailwind 不会输出它。它只在 `animate-ai-fade-up` 已注册（keyframe 名 `fade-up` 由 tailwind.config.js L126-129 全局输出）的前提下有效：因为 Task 1-3 的组件类用到 `animate-ai-fade-up`，该 keyframe 一定存在于产物 CSS 中，内联引用同名 keyframe 合法。保留此写法与参考实现一致；若执行时发现 details 动画不播放（内容裁剪导致 keyframe 被摇树），改为 `className="animate-ai-fade-up"` + 内联 animationDelay 即可，测试不受影响。

**(b) `Tasks.tsx` TaskRow 行外壳**：import 区（Task 3 已加的 AiRecommendationCard import 行后）加：

```tsx
import { AiTaskRows, type AiTaskRowItem } from '@/components/ui/ai/AiTaskRows';
```

TaskRow 的 return（现状 L132-251；mutations 与 handleDelete/handleTrigger/handleCancel/handleRetry L83-130 全部保留不动）：

```tsx
  return (
    <div className="border-b border-cinema-800 last:border-b-0">
      <div
        className="flex items-center gap-3 px-4 py-3 hover:bg-cinema-800/30 transition-colors cursor-pointer"
        onClick={onToggleExpand}
      >
        {/* …状态图标/名称/调度/心跳/进度条/操作按钮…（L138-245） */}
      </div>

      {/* Expanded detail */}
      {isExpanded && <TaskDetail task={task} />}
    </div>
  );
```

整体替换为（行外壳交给 AiTaskRows 单行 capsules 卡；状态徽章/进度/心跳信息分别映射到内置徽章、环内进度与 meta；操作按钮进 `trailing` 插槽并保留既有 `stopPropagation`；展开区原样挂 `<TaskDetail task={task} />`——CascadeRewriteDetail 链路不受影响）：

```tsx
  const metaParts = [
    scheduleTypeLabels[task.schedule_type] || task.schedule_type,
    task.cron_pattern || null,
    task.status === 'running' && task.progress > 0 ? `${task.progress}%` : null,
    task.retry_count > 0 ? `重试 ${task.retry_count}/${task.max_retries}` : null,
    task.status === 'running' ? `心跳${heartbeat.text}` : null,
  ].filter(Boolean);

  const statusPill: Record<string, string> = {
    running: 'bg-ai-accent-tint text-ai-accent-ink',
    completed: 'bg-ai-green/10 text-ai-green',
    failed: 'bg-ai-red/10 text-ai-red',
    cancelled: 'bg-ai-orange/10 text-ai-orange',
    pending: 'bg-ai-hover text-ai-ink-2',
  };

  const item: AiTaskRowItem<Task> = {
    key: task.id,
    status: task.status in statusPill ? task.status : 'pending',
    progress: task.status === 'running' ? task.progress : undefined,
    label: task.name,
    meta: metaParts.join(' · '),
    pill: (
      <span
        className={cn(
          'inline-flex h-[22px] items-center rounded-full px-2 text-[11.5px] font-medium',
          statusPill[task.status] || statusPill.pending
        )}
      >
        {status.label}
      </span>
    ),
    trailing: (
      <>
        {task.status === 'running' ? (
          <button
            onClick={handleCancel}
            className="p-1.5 rounded hover:bg-ai-red/10 text-ai-ink-3 hover:text-ai-red transition-colors"
            title="取消"
          >
            <Square className="w-3.5 h-3.5" />
          </button>
        ) : task.status === 'failed' ? (
          <button
            onClick={handleRetry}
            className="p-1.5 rounded hover:bg-ai-orange/10 text-ai-ink-3 hover:text-ai-orange transition-colors"
            title="重试"
          >
            <Play className="w-3.5 h-3.5" />
          </button>
        ) : (
          <button
            onClick={handleTrigger}
            className="p-1.5 rounded hover:bg-ai-green/10 text-ai-ink-3 hover:text-ai-green transition-colors"
            title="执行"
          >
            <Play className="w-3.5 h-3.5" />
          </button>
        )}
        <button
          onClick={handleDelete}
          disabled={isDeleting}
          className="p-1.5 rounded hover:bg-ai-red/10 text-ai-ink-3 hover:text-ai-red transition-colors"
          title="删除"
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
      </>
    ),
    payload: task,
  };

  return (
    <AiTaskRows
      rows={[item]}
      expandedKey={isExpanded ? task.id : null}
      onToggle={() => onToggleExpand()}
      renderDetail={() => <TaskDetail task={task} />}
    />
  );
```

替换后清理：`StatusIcon`/`statusConfig` 的行内图标用法消失，但 `statusConfig` 的 `label` 仍在 pill 中使用——将 `statusConfig` 的 `color`/`icon` 字段与 lucide import（`Clock`/`CheckCircle2`/`XCircle`/`AlertCircle`/`Loader2`/`Heart`/`ChevronDown`/`ChevronUp`）中因此不再使用的符号一并删除（`Loader2` 在 Tasks 主组件 L685 加载态仍用，保留；`Play`/`Square`/`Trash2`/`Plus`/`ListChecks` 保留）。以 tsc/eslint 未使用告警为准逐个删。

**(c) `Tasks.tsx` 列表容器（L700-701）**：单行 capsules 卡自带 `border-ai-line` 外壳，外层容器不再需要影院色边框包裹：

```tsx
        <div className="bg-cinema-900/50 rounded-lg border border-cinema-800 overflow-hidden">
```

改为：

```tsx
        <div>
```

（分组标题行 L707/L722/L737/L752 的 `px-4 py-2 bg-cinema-800/30` 样式保留不动。）

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiTaskRows.test.tsx && npx tsc --noEmit`
Expected: 6 passed；tsc 干净

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiTaskRows.tsx src-frontend/src/components/ui/ai/__tests__/AiTaskRows.test.tsx src-frontend/src/pages/Tasks.tsx
git commit -m "feat: AiTaskRows 组件入库并替换 Tasks 任务行外壳（P2 Task4）"
```

---

### Task 5: AiSelectionActions 组件 + RichTextEditor 划词 AI 操作浮条（新增挂载）

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiSelectionActions.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiSelectionActions.test.tsx`
- Modify: `src-frontend/src/frontstage/components/RichTextEditor.tsx`（state 区 L288 后、handler 区 L949 `handleSlashCancel` 后、渲染层 L1408 EditorContextMenu 前、import 区 L85 后）
- Modify: `src-frontend/src/frontstage/styles/frontstage.css`（`:root` 内 L79 `--ai-orange: #f59e0b;` 行后补 `--shadow-float`）

**Interfaces:**
- Consumes: P1 令牌与 `animate-pop-in` / `animate-shimmer-text` / `animate-stream-in` / `animate-ai-spin`；`segmentStreamText`（`./AiStreamingText` 的既有导出函数，复用不改）；`cn`；lucide `Sparkles` / `Type` / `Scissors` / `ArrowUp` / `ChevronRight` / `Check` / `X` / `RefreshCw`；宿主 `smartExecute`（RichTextEditor L80 既有 import）与 `editor.commands.insertContentAt`（L1094 既有用法先例）
- Produces:
  - `export type AiSelectionActionKey = 'polish' | 'expand' | 'rewrite' | 'custom'`
  - `export type AiSelectionPhase = 'idle' | 'thinking' | 'result'`
  - `export interface AiSelectionActionsProps { containerRef: React.RefObject<HTMLElement | null>; selectedText: string; phase: AiSelectionPhase; resultText?: string; onRun: (action: AiSelectionActionKey, customInstruction?: string) => void; onAccept: () => void; onDiscard: () => void }`
  - `export function AiSelectionActions(props): JSX.Element | null`；`data-testid="ai-selection-actions"`

**交互方案决策（本 Task 的核心设计选择，已评估三种候选）：**

1. ~~**ghost/insertText 全文替换通路**~~：把改写结果接入 FrontstageApp 的 generatedText 幽灵文本管线。否决——幽灵管线是「光标处追加续写」语义，不是「选区替换」；且 P1 已明令不改 FrontstageApp 的打字机/race-lock 逻辑（shouldShowGhostTree L1239 等多道锁），接入风险最高。
2. ~~**注入指令到底栏输入框**~~：动作点击后把「润色：\<选文\」预填进 FrontstageBottomBar 的 AiPromptBar。否决——底栏在 FrontstageApp 层（FrontstageBottomBar.tsx），需要新增 RichTextEditor→FrontstageApp→FrontstageBottomBar 的 props 钻孔与焦点管理；更关键的是结果仍走「光标处追加」幽灵通路，Tab 接受后插入在选区之后而非替换选区，语义错误，用户需手动删原文。
3. ✅ **组件内结果面板 + 宿主 insertContentAt 替换（选用）**：浮条负责动作选择与 thinking 反馈；宿主（RichTextEditor）调既有 `smartExecute`（与 inline suggestion L865-869 同一 agent 入口，`selected_text` 字段携带选文）拿到 `final_content` 后，结果在浮条下方的结果面板中以嵌入 StreamText 逐段显现；「保留」→ `editor.commands.insertContentAt({ from, to }, resultText)` 直接替换选区（智能排版 `handleFormatText` L910-929 直接改内容、appendText L1094 insertContentAt 均有先例，不触碰 race-lock）；「放弃」→ 清空状态。语义与参考实现的 Keep/Discard 1:1，零 FrontstageApp 改动，零后端改动。

**iconoir → lucide 映射（参考实现 10 图标）：** ChatBubbleQuestion→MessageCircleQuestion、Spark→Sparkles、Scissor→Scissors、EmojiSatisfied→Smile、TextBox→Type、ArrowUp→ArrowUp、NavArrowRight→ChevronRight、Check→Check、Xmark→X、Refresh→RefreshCw。本批动作集为 润色（Sparkles）/扩写（Type）/改写（Scissors）+ 自定义指令（ArrowUp）+ 结果态 保留（Check）/放弃（X）/重试（RefreshCw）；MessageCircleQuestion（explain）与 Smile（tone）两个映射暂不使用（对应动作本批不落）。

**移植说明：** atoms/Shimmer 源码缺失 → 按 AiThinking.tsx:66-73 的 `animate-shimmer-text` 渐变模式复刻；atoms/StreamText 源码缺失 → 组件内嵌 ~30 行私有 `SelectionStreamText`（复用 `segmentStreamText` + `animate-stream-in`，错峰 animationDelay 模拟流式，不动 P1 受控版 AiStreamingText）；定位保留参考的 `selection.getClientRects` + ResizeObserver + rAF + 宽度动画（Web Animations API，纯 DOM 无依赖）；`mousedown preventDefault` 防选区塌陷（同 EditorContextMenu.tsx:95-98 既有模式）；演示文案 LEAD/PICKED/REWRITE 与自运行 thinking→streaming 定时器全部剥离；`bg-[color-mix(...)]` 选区高亮不落（编辑器原生选区自带高亮）。

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiSelectionActions.test.tsx
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { createRef } from 'react';
import { AiSelectionActions, type AiSelectionActionsProps } from '../AiSelectionActions';

// jsdom 无真实选区矩形：stub getSelection 与 rAF，让 place() 能算出锚点
const rect = {
  left: 100,
  top: 100,
  right: 200,
  bottom: 120,
  width: 100,
  height: 20,
  x: 100,
  y: 100,
  toJSON: () => ({}),
};

beforeEach(() => {
  vi.stubGlobal('getSelection', () => ({
    rangeCount: 1,
    getRangeAt: () => ({
      getBoundingClientRect: () => rect,
      getClientRects: () => [rect],
    }),
  }));
  vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => setTimeout(cb, 0));
  vi.stubGlobal('cancelAnimationFrame', (id: number) => clearTimeout(id));
});

async function flushPlace() {
  await act(async () => {
    await new Promise(r => setTimeout(r, 0));
  });
}

function renderBar(props: Partial<AiSelectionActionsProps> = {}) {
  const containerRef = createRef<HTMLElement>();
  const utils = render(
    <div ref={containerRef as React.RefObject<HTMLDivElement>}>
      <AiSelectionActions
        containerRef={containerRef as React.RefObject<HTMLElement>}
        selectedText="被选中的文字"
        phase="idle"
        onRun={() => {}}
        onAccept={() => {}}
        onDiscard={() => {}}
        {...props}
      />
    </div>
  );
  return utils;
}

describe('AiSelectionActions', () => {
  it('selectedText 为空字符串时不渲染', () => {
    const { container } = renderBar({ selectedText: '' });
    expect(container.querySelector('[data-testid="ai-selection-actions"]')).toBeNull();
  });

  it('idle 渲染润色/扩写动作，点击调用 onRun(action)', async () => {
    const onRun = vi.fn();
    renderBar({ onRun });
    await flushPlace();
    fireEvent.click(screen.getByRole('button', { name: /润色/ }));
    expect(onRun).toHaveBeenCalledWith('polish', undefined);
  });

  it('展开 chevron 后出现改写动作', async () => {
    const onRun = vi.fn();
    renderBar({ onRun });
    await flushPlace();
    fireEvent.click(screen.getByRole('button', { name: '展开更多操作' }));
    fireEvent.click(screen.getByRole('button', { name: /改写/ }));
    expect(onRun).toHaveBeenCalledWith('rewrite', undefined);
  });

  it('自定义指令输入后回车调用 onRun(custom, 文本)', async () => {
    const onRun = vi.fn();
    renderBar({ onRun });
    await flushPlace();
    const input = screen.getByLabelText('描述修改要求');
    fireEvent.change(input, { target: { value: '改成古文腔' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onRun).toHaveBeenCalledWith('custom', '改成古文腔');
  });

  it('thinking 阶段显示 shimmer 忙碌标签', () => {
    renderBar({ phase: 'thinking' });
    const busy = screen.getByTestId('ai-selection-busy');
    expect(busy.className).toContain('animate-shimmer-text');
  });

  it('result 阶段渲染结果分词与 保留/放弃/重试，回调正确', () => {
    const onAccept = vi.fn();
    const onDiscard = vi.fn();
    renderBar({ phase: 'result', resultText: '改写后的文字', onAccept, onDiscard });
    expect(screen.getByTestId('ai-selection-stream')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /保留/ }));
    expect(onAccept).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: /放弃/ }));
    expect(onDiscard).toHaveBeenCalled();
  });

  it('浮条 mousedown 被 preventDefault（防选区塌陷）', () => {
    renderBar();
    const bar = screen.getByTestId('ai-selection-actions');
    const event = new MouseEvent('mousedown', { bubbles: true, cancelable: true });
    bar.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiSelectionActions.test.tsx`
Expected: FAIL（组件不存在）

- [ ] **Step 3: Write implementation**

**(a) `src-frontend/src/components/ui/ai/AiSelectionActions.tsx`** — 新建：

```tsx
/**
 * AiSelectionActions — 划词 AI 操作浮条（适配自 beautifului SelectionActions）
 *
 * 受控约定：selectedText/phase/resultText/回调全部由宿主（RichTextEditor）提供；
 * 剥离参考实现：演示文案 LEAD/PICKED/REWRITE、自运行 thinking→streaming 定时器、
 * iconoir-react 依赖（映射 lucide-react，见计划映射表）、Explain/Tone/Grammar 动作、
 * color-mix 选区高亮（编辑器原生选区自带高亮）。
 *
 * 移植说明：
 * - atoms/Shimmer 源码缺失 → 按 AiThinking.tsx:66-73 的 animate-shimmer-text 渐变模式复刻；
 * - atoms/StreamText 源码缺失 → 内嵌私有 SelectionStreamText（复用 segmentStreamText +
 *   animate-stream-in，错峰 animationDelay 模拟流式），不动 P1 受控版 AiStreamingText；
 * - 定位保留参考的 selection.getClientRects + ResizeObserver + rAF + 宽度动画（纯 DOM）；
 * - mousedown preventDefault 防选区塌陷（同 EditorContextMenu.tsx:95-98 既有模式）；
 * - shadow-overlay → shadow-float（需宿主窗口定义 --shadow-float，幕前见 frontstage.css）。
 */
import { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react';
import { ArrowUp, Check, ChevronRight, RefreshCw, Scissors, Sparkles, Type, X } from 'lucide-react';
import { segmentStreamText } from './AiStreamingText';
import { cn } from '@/utils/cn';

export type AiSelectionActionKey = 'polish' | 'expand' | 'rewrite' | 'custom';
export type AiSelectionPhase = 'idle' | 'thinking' | 'result';

export interface AiSelectionActionsProps {
  /** 定位宿主（相对定位的编辑器容器）；浮条坐标相对它计算 */
  containerRef: React.RefObject<HTMLElement | null>;
  /** 空字符串 = 不渲染 */
  selectedText: string;
  phase: AiSelectionPhase;
  resultText?: string;
  onRun: (action: AiSelectionActionKey, customInstruction?: string) => void;
  onAccept: () => void;
  onDiscard: () => void;
}

const ACTIONS: { key: Exclude<AiSelectionActionKey, 'custom'>; label: string; icon: typeof Sparkles }[] = [
  { key: 'polish', label: '润色', icon: Sparkles },
  { key: 'expand', label: '扩写', icon: Type },
  { key: 'rewrite', label: '改写', icon: Scissors },
];

const BUSY_LABEL: Record<AiSelectionActionKey, string> = {
  polish: '润色中',
  expand: '扩写中',
  rewrite: '改写中',
  custom: '处理中',
};

const control =
  'inline-flex h-7 shrink-0 items-center gap-1 rounded-full px-2.5 text-[12px] font-normal text-ai-ink transition-[background-color,color,transform] duration-150 hover:bg-ai-hover active:scale-[0.96]';

const primary =
  'inline-flex h-7 shrink-0 items-center gap-1 rounded-full bg-ai-ink px-2.5 text-[12.5px] font-normal text-ai-surface transition-[opacity,transform] duration-150 hover:opacity-90 active:scale-[0.96]';

/** 内嵌私有流式显现（源码 atoms/StreamText 缺失；错峰 delay 模拟流式，无定时器） */
function SelectionStreamText({ text, onProgress }: { text: string; onProgress?: () => void }) {
  const tokens = segmentStreamText(text);
  useEffect(() => {
    onProgress?.();
  }, [tokens.length, onProgress]);
  return (
    <span data-testid="ai-selection-stream">
      {tokens.map((token, i) => (
        <span
          key={i}
          className="animate-stream-in inline [will-change:filter,opacity]"
          style={{ animationDelay: `${i * 45}ms` }}
        >
          {token}
        </span>
      ))}
    </span>
  );
}

/** atoms/Shimmer 缺失：复刻 AiThinking.tsx:66-73 的 shimmer 渐变文字 */
function ShimmerLabel({ children }: { children: React.ReactNode }) {
  return (
    <span
      className="animate-shimmer-text bg-clip-text text-transparent"
      data-testid="ai-selection-busy"
      style={{
        backgroundImage:
          'linear-gradient(90deg, var(--ai-ink-3) 35%, var(--ai-ink) 50%, var(--ai-ink-3) 65%)',
        backgroundSize: '200% 100%',
      }}
    >
      {children}
    </span>
  );
}

export function AiSelectionActions({
  containerRef,
  selectedText,
  phase,
  resultText,
  onRun,
  onAccept,
  onDiscard,
}: AiSelectionActionsProps) {
  const [expanded, setExpanded] = useState(false);
  const [prompt, setPrompt] = useState('');
  const [lastAction, setLastAction] = useState<AiSelectionActionKey>('polish');
  const [anchor, setAnchor] = useState<{ x: number; y: number } | null>(null);

  const barRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const frameRef = useRef<ReturnType<typeof requestAnimationFrame> | null>(null);
  const prevPhaseRef = useRef<AiSelectionPhase>('idle');
  const lastWidthRef = useRef(0);
  const widthAnimationRef = useRef<Animation | null>(null);

  const visible = selectedText.trim().length > 0;
  const hasPrompt = prompt.trim().length > 0;

  /* 贴在最末一个选区行下方，横向对准整个选区中心；rAF 批合测量 */
  const place = useCallback(() => {
    if (frameRef.current !== null) cancelAnimationFrame(frameRef.current);
    frameRef.current = requestAnimationFrame(() => {
      const host = containerRef.current;
      const selection = window.getSelection();
      if (!host || !selection || selection.rangeCount === 0) return;
      const range = selection.getRangeAt(0);
      const bounds = range.getBoundingClientRect();
      const lines = Array.from(range.getClientRects());
      const lastLine = lines.at(-1);
      if (!lastLine || (bounds.width === 0 && bounds.height === 0)) return;
      const hostBounds = host.getBoundingClientRect();
      const next = {
        x: Math.round(bounds.left - hostBounds.left + bounds.width / 2),
        y: Math.round(lastLine.bottom - hostBounds.top + 8),
      };
      setAnchor(current => (current && current.x === next.x && current.y === next.y ? current : next));
    });
  }, [containerRef]);

  useLayoutEffect(() => {
    if (visible) place();
  }, [visible, phase, place]);

  useEffect(() => {
    if (!visible) return;
    const host = containerRef.current;
    if (!host) return;
    const observer = new ResizeObserver(place);
    observer.observe(host);
    // 编辑器滚动容器在 host 内部；scroll 不冒泡，捕获阶段监听
    host.addEventListener('scroll', place, true);
    window.addEventListener('resize', place);
    return () => {
      observer.disconnect();
      host.removeEventListener('scroll', place, true);
      window.removeEventListener('resize', place);
      if (frameRef.current !== null) cancelAnimationFrame(frameRef.current);
    };
  }, [visible, place, containerRef]);

  /* phase 切换时从上一个渲染宽度动画到新的固有宽度（Web Animations API） */
  useLayoutEffect(() => {
    const bar = barRef.current;
    const content = contentRef.current;
    if (!bar || !content || !visible) return;
    const nextWidth = Math.ceil(content.getBoundingClientRect().width) + 8;
    const previousWidth = lastWidthRef.current || Math.ceil(bar.getBoundingClientRect().width);
    if (prevPhaseRef.current !== phase && Math.abs(nextWidth - previousWidth) > 1) {
      widthAnimationRef.current?.cancel();
      const animation = bar.animate(
        [{ width: `${previousWidth}px` }, { width: `${nextWidth}px` }],
        { duration: 320, easing: 'cubic-bezier(0.23,1,0.32,1)' }
      );
      widthAnimationRef.current = animation;
      animation.onfinish = () => {
        lastWidthRef.current = nextWidth;
        widthAnimationRef.current = null;
      };
    } else {
      lastWidthRef.current = nextWidth;
    }
    prevPhaseRef.current = phase;
  }, [phase, visible]);

  useEffect(() => {
    const content = contentRef.current;
    if (!content) return;
    const observer = new ResizeObserver(() => {
      if (widthAnimationRef.current?.playState === 'running') return;
      lastWidthRef.current = Math.ceil(content.getBoundingClientRect().width) + 8;
    });
    observer.observe(content);
    return () => {
      observer.disconnect();
      widthAnimationRef.current?.cancel();
    };
  }, []);

  if (!visible) return null;

  const run = (action: AiSelectionActionKey, customInstruction?: string) => {
    setLastAction(action);
    setExpanded(false);
    onRun(action, customInstruction);
  };

  const submitCustom = () => {
    const text = prompt.trim();
    if (!text) return;
    run('custom', text);
  };

  const shown = anchor !== null;

  return (
    <div
      className="absolute top-0 left-0 z-10"
      style={{
        transform: `translate3d(${anchor?.x ?? 0}px, ${anchor?.y ?? 0}px, 0) translateX(-50%)`,
        transition: 'transform 320ms cubic-bezier(0.77,0,0.175,1), opacity 180ms ease-out',
        opacity: shown ? 1 : 0,
        pointerEvents: shown ? 'auto' : 'none',
        willChange: 'transform',
      }}
    >
      <div
        ref={barRef}
        data-testid="ai-selection-actions"
        onMouseDown={e => {
          e.preventDefault();
          e.stopPropagation();
        }}
        className={cn(
          'flex h-9 w-fit max-w-[calc(100vw-48px)] items-center justify-center gap-0.5 overflow-hidden rounded-full border border-ai-line bg-ai-surface p-1 text-ai-ink shadow-float antialiased',
          shown && 'animate-pop-in'
        )}
      >
        <div ref={contentRef} className="flex w-fit shrink-0 items-center justify-center gap-0.5">
          {phase === 'thinking' && (
            <span className="inline-flex h-7 items-center gap-1.5 px-2.5 text-[12.5px] whitespace-nowrap text-ai-ink-2">
              <span className="animate-ai-spin size-3 shrink-0 rounded-full border-[1.5px] border-ai-line-strong border-t-ai-ink-2" />
              <ShimmerLabel>{BUSY_LABEL[lastAction]}…</ShimmerLabel>
            </span>
          )}

          {phase === 'result' && (
            <>
              <button type="button" onClick={onAccept} className={primary}>
                <Check size={14} strokeWidth={1.8} aria-hidden />
                保留
              </button>
              <button type="button" onClick={onDiscard} className={control}>
                <X size={14} strokeWidth={1.8} aria-hidden />
                放弃
              </button>
              <span className="mx-0.5 h-4 w-px shrink-0 bg-ai-line" />
              <button
                type="button"
                aria-label="重试"
                onClick={() => run(lastAction)}
                className="flex size-7 shrink-0 items-center justify-center rounded-full text-ai-ink-3 transition-[background-color,color,transform] duration-150 hover:bg-ai-hover-2 hover:text-ai-ink-2 active:scale-[0.96]"
              >
                <RefreshCw size={14} strokeWidth={1.8} aria-hidden />
              </button>
            </>
          )}

          {phase === 'idle' && (
            <>
              {/* 自定义指令输入（有内容时吃掉动作区宽度） */}
              <div
                className="flex min-w-0 items-center overflow-hidden transition-[max-width,opacity,transform] duration-[400ms]"
                style={{
                  maxWidth: expanded ? 0 : 145,
                  opacity: expanded ? 0 : 1,
                  transform: expanded ? 'translateX(-8px)' : 'translateX(0)',
                  transitionTimingFunction: 'cubic-bezier(0.23,1,0.32,1)',
                }}
              >
                <input
                  value={prompt}
                  onChange={e => setPrompt(e.target.value)}
                  onKeyDown={e => {
                    if (e.key === 'Enter') {
                      e.preventDefault();
                      submitCustom();
                    }
                  }}
                  aria-label="描述修改要求"
                  placeholder="描述修改要求…"
                  className="h-7 w-[145px] bg-transparent pr-2.5 pl-3 text-[12.5px] text-ai-ink placeholder:text-ai-ink-3"
                />
              </div>

              <div
                className="flex min-w-0 items-center gap-0.5 overflow-hidden transition-[max-width,opacity,transform] duration-[400ms]"
                style={{
                  maxWidth: hasPrompt ? 0 : expanded ? 300 : 150,
                  opacity: hasPrompt ? 0 : 1,
                  transform: hasPrompt ? 'translateX(-8px)' : 'translateX(0)',
                  transitionTimingFunction: 'cubic-bezier(0.23,1,0.32,1)',
                }}
              >
                {!expanded && <span className="mx-1 h-4 w-px shrink-0 bg-ai-line-strong" />}
                {ACTIONS.slice(0, expanded ? 3 : 2).map(({ key, label, icon: Icon }) => (
                  <button key={key} type="button" onClick={() => run(key)} className={control}>
                    <Icon size={14} strokeWidth={1.8} aria-hidden />
                    {label}
                  </button>
                ))}
                <span className="mx-0.5 h-4 w-px shrink-0 bg-ai-line" />
                <button
                  type="button"
                  aria-label={expanded ? '收起操作' : '展开更多操作'}
                  aria-expanded={expanded}
                  onClick={() => setExpanded(v => !v)}
                  className="flex size-7 shrink-0 items-center justify-center rounded-full text-ai-ink transition-[background-color,transform] duration-200 hover:bg-ai-hover active:scale-[0.96]"
                >
                  <ChevronRight
                    size={14}
                    strokeWidth={1.8}
                    aria-hidden
                    className="transition-transform duration-[400ms]"
                    style={{
                      transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)',
                      transitionTimingFunction: 'cubic-bezier(0.23,1,0.32,1)',
                    }}
                  />
                </button>
              </div>

              {/* 有自定义文本时的发送钮 */}
              <div
                className="flex min-w-0 items-center overflow-hidden transition-[max-width,opacity,transform] duration-[400ms]"
                style={{
                  maxWidth: hasPrompt ? 30 : 0,
                  opacity: hasPrompt ? 1 : 0,
                  transform: hasPrompt ? 'scale(1)' : 'scale(0.88)',
                  transitionTimingFunction: 'cubic-bezier(0.23,1,0.32,1)',
                }}
              >
                <button
                  type="button"
                  aria-label="发送修改指令"
                  onClick={submitCustom}
                  className="flex size-7 shrink-0 items-center justify-center rounded-full bg-ai-ink text-ai-surface transition-[opacity,transform] duration-200 active:scale-[0.94]"
                >
                  <ArrowUp size={16} strokeWidth={2.4} aria-hidden />
                </button>
              </div>
            </>
          )}
        </div>
      </div>

      {/* 结果面板：浮条下方流式显现改写结果（保留/放弃在浮条内） */}
      {phase === 'result' && resultText && (
        <div className="absolute top-full left-1/2 mt-2 w-[min(420px,calc(100vw-48px))] -translate-x-1/2 rounded-[12px] border border-ai-line bg-ai-surface p-3 shadow-float">
          <p className="text-[13px] leading-relaxed text-ai-ink">
            <SelectionStreamText text={resultText} onProgress={place} />
          </p>
        </div>
      )}
    </div>
  );
}

export default AiSelectionActions;
```

**(b) `frontstage.css` 补 `--shadow-float`（幕前）**：在 `:root` 内 L79 `--ai-orange: #f59e0b;` 行后插入：

```css
  /* P2 Task5：幕前浮动层阴影。shadow-float 工具（tailwind.config.js L71）的变量此前仅
     幕后 tokens.css 定义，幕前解析为空导致 P1 AiPromptBar 弹出层无阴影；补齐后
     AiSelectionActions 浮条/结果面板与 P1 幕前组件同步获得预期阴影（属修复）。 */
  --shadow-float: 0 8px 24px rgba(0, 0, 0, 0.12);
```

（取值依据：frontstage.css L611 既有浮层同款 `0 8px 24px rgba(0, 0, 0, 0.12)`。）

**(c) `RichTextEditor.tsx` 接线**：

import 区（L85 `import { AiStreamingText } from '@/components/ui/ai/AiStreamingText';` 行后）加：

```tsx
import {
  AiSelectionActions,
  type AiSelectionActionKey,
  type AiSelectionPhase,
} from '@/components/ui/ai/AiSelectionActions';
```

state 区（L288-292 `selectedRange` state 之后）加：

```tsx
    // 划词 AI 操作浮条状态（P2 Task5）：phase 由浮条动作驱动，结果经 insertContentAt 替换选区
    const [selectionAction, setSelectionAction] = useState<{
      phase: AiSelectionPhase;
      resultText?: string;
    }>({ phase: 'idle' });
    // thinking 开始后锁定选区范围，用户后续改动选区不影响替换目标
    const selectionActionRangeRef = useRef<{ from: number; to: number } | null>(null);
```

handler 区（L949 `handleSlashCancel` 定义之后）加：

```tsx
    // ===== 划词 AI 操作浮条（P2 Task5）=====
    // 与 inline suggestion（L857-907）同一 smartExecute agent 入口；
    // 结果不走幽灵管线（光标追加语义不符），由用户「保留」时 insertContentAt 直接替换选区。
    const handleSelectionRun = useCallback(
      async (action: AiSelectionActionKey, customInstruction?: string) => {
        if (!editor || !selectedRange) return;
        const PRESET_INSTRUCTIONS: Record<Exclude<AiSelectionActionKey, 'custom'>, string> = {
          polish: '润色这段文字，保持原意与篇幅',
          expand: '扩写这段文字，丰富细节与画面感',
          rewrite: '改写这段文字，换一种表达方式',
        };
        const instruction =
          action === 'custom'
            ? customInstruction?.trim() || ''
            : PRESET_INSTRUCTIONS[action];
        if (!instruction) return;
        selectionActionRangeRef.current = { from: selectedRange.from, to: selectedRange.to };
        setSelectionAction({ phase: 'thinking' });
        try {
          const result = await smartExecute({
            user_input: instruction,
            current_content: editor.getHTML() || '',
            selected_text: selectedRange.text,
          });
          const text = (result.final_content || '').replace(/<[^>]*>/g, '').trim();
          if (text) {
            setSelectionAction({ phase: 'result', resultText: text });
          } else {
            setSelectionAction({ phase: 'idle' });
            onShowStatus?.('未获得改写结果');
          }
        } catch (err) {
          rtEditorLogger.error('Selection action failed', { error: err });
          onShowStatus?.(`划词改写失败：${extractMessage(err)}`);
          setSelectionAction({ phase: 'idle' });
        }
      },
      [editor, selectedRange, onShowStatus]
    );

    const handleSelectionAccept = useCallback(() => {
      const range = selectionActionRangeRef.current;
      const text = selectionAction.resultText;
      if (editor && range && text) {
        editor.commands.insertContentAt({ from: range.from, to: range.to }, text);
        onShowStatus?.('已替换为改写内容');
      }
      selectionActionRangeRef.current = null;
      setSelectionAction({ phase: 'idle' });
    }, [editor, selectionAction.resultText, onShowStatus]);

    const handleSelectionDiscard = useCallback(() => {
      selectionActionRangeRef.current = null;
      setSelectionAction({ phase: 'idle' });
    }, []);
```

渲染层（L1408 `{/* 编辑器右键菜单 */}` 注释前）插入：

```tsx
          {/* 划词 AI 操作浮条（P2 Task5）：生成中/幽灵文本显示中/禅模式不出现，
              避免与 ghost 树（L1369-1389）和萤火提示抢占视觉焦点 */}
          {selectedRange && !generatedText && !isGenerating && !isZenMode && (
            <AiSelectionActions
              containerRef={containerRef}
              selectedText={selectedRange.text}
              phase={selectionAction.phase}
              resultText={selectionAction.resultText}
              onRun={handleSelectionRun}
              onAccept={handleSelectionAccept}
              onDiscard={handleSelectionDiscard}
            />
          )}
```

注意点：
- `containerRef` 是 L1299-1300 外层 `relative` 容器的既有 ref，直接复用，不新建。
- 浮条挂载点在滚动容器（L1314）**之外**、EditorContextMenu 之前的同层级：坐标由 `getBoundingClientRect`（视口系）相对 host 矩形换算，滚动时经捕获阶段 scroll 监听重定位，不会被 `overflow-y-auto` 裁剪。
- 角色卡片弹窗（点击角色名 L1032-1036 也构造选区）与浮条会同时出现——既有行为不冲突（弹窗在 CharacterCardPopup 层，z 序更高）；若执行中实测视觉打架，给浮条追加 `!showPopup` 条件即可，组件无需改。
- `isAiThinking`（inline suggestion 通路占用中）时浮条动作会排队失败——`smartExecute` 串行语义与 inline suggestion 相同，不加额外锁；若实测并发报错，给挂载条件追加 `!isAiThinking`。

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiSelectionActions.test.tsx && npx tsc --noEmit`
Expected: 7 passed；tsc 干净（RichTextEditor 接线无单测——TipTap 在 jsdom 不可运行，既有 FrontstageApp 测试均 mock 本组件，此先例沿用）

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiSelectionActions.tsx src-frontend/src/components/ui/ai/__tests__/AiSelectionActions.test.tsx src-frontend/src/frontstage/components/RichTextEditor.tsx src-frontend/src/frontstage/styles/frontstage.css
git commit -m "feat: AiSelectionActions 划词浮条入库并接入幕前编辑器（P2 Task5）"
```

---

### Task 6: 全量回归门 + barrel 导出 + 文档同步

**Files:**
- Modify: `src-frontend/src/components/index.ts`（barrel 导出 5 个组件，P1 分组段 L5-24 之后）
- Modify: `CHANGELOG.md`（顶部加 P2 Unreleased 段）
- Modify: `PROJECT_STATUS.md`（「最近完成功能」加 P2 条目）
- Modify: `AGENTS.md`（编码风格节 AI 组件行更新）

**Interfaces:**
- Consumes: Task 1-5 全部产出
- Produces: barrel 导出 `AiContextCards` / `AiToolChips` / `AiRecommendationCard` / `AiTaskRows` / `AiSelectionActions`（+ 各自 Props/Item/Option/Detail/Phase 类型用 `export type`）

- [ ] **Step 1: barrel 导出**

`src-frontend/src/components/index.ts` 在 L24（AiApprovalCard 类型导出块的 `} from './ui/ai/AiApprovalCard';` 行）后、L25 `export { DataLoader } from './DataLoader';` 前插入：

```ts
// P2 - AI Native Components（代理与任务）
export { AiContextCards } from './ui/ai/AiContextCards';
export { AiToolChips } from './ui/ai/AiToolChips';
export { AiRecommendationCard } from './ui/ai/AiRecommendationCard';
export { AiTaskRows } from './ui/ai/AiTaskRows';
export { AiSelectionActions } from './ui/ai/AiSelectionActions';
export type {
  AiContextCardsProps,
  AiContextCardItem,
  AiContextCardSource,
} from './ui/ai/AiContextCards';
export type { AiToolChipsProps, AiToolChipItem } from './ui/ai/AiToolChips';
export type {
  AiRecommendationCardProps,
  AiRecommendationOption,
} from './ui/ai/AiRecommendationCard';
export type {
  AiTaskRowsProps,
  AiTaskRowItem,
  AiTaskRowDetail,
  AiTaskRowStatus,
} from './ui/ai/AiTaskRows';
export type {
  AiSelectionActionsProps,
  AiSelectionActionKey,
  AiSelectionPhase,
} from './ui/ai/AiSelectionActions';
```

- [ ] **Step 2: 全量回归门**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | tail -3 && npm run format:check`
Expected: tsc 干净；vitest **≥514 passed / 3 skipped**（基线 487 + Task1 5 + Task2 5 + Task3 6 + Task4 6 + Task5 7 = 516 预期下限 514，以实际输出为准并记录进 CHANGELOG；只允许比基线多）；format 通过

Run: `python3 scripts/architecture_guard.py`
Expected: 退出码 0（纯前端改动，应为通过）

（Rust 侧无改动：`cargo test` 基线 1326 passed / 2 ignored 不变，本批不重跑。）

- [ ] **Step 3: 文档同步（版本号不动，发版另行进行）**

**(a) `CHANGELOG.md`** — 在 L4 空行后、`## Unreleased（P1 AI 原生组件库 · 生成体验）`（L5）前插入：

```markdown
## Unreleased（P2 AI 原生组件库 · 代理与任务）

### 功能：beautifului AI 原生组件第二批（设计文档 P2 范围）

将 beautifului.dev 的 5 个代理与任务组件适配为受控组件入库 `src-frontend/src/components/ui/ai/`，并逐点接入幕后/幕前落点。沿用 P1 令牌桥（`--ai-*` 双窗口各自定义），不引新依赖（图标 lucide-react，iconoir 10 图标已映射）；纯前端阶段，无后端改动。

- **AiContextCards（Task1）**：检索上下文卡片列表（标题/正文/来源 chip 三层，纯 CSS 错峰入场），替换 PromptCoverageBar 上下文槽位勾叉清单（SLOT_LABELS 10 项数据零改造）；可选接入 AgencyStudio 黑板条目。
- **AiToolChips（Task2）**：单选筛选 chips 组（提取参考 chip 视觉语法：pop-in 交错入场 + active 实心反白 + radiogroup 语义），替换 Tasks 状态筛选条与 Skills 分类筛选条。
- **AiRecommendationCard（Task3）**：AI 建议确认卡（信号条 + Alternatives 抽屉 + 接受/拒绝双动作，status 受控），替换级联改写 CascadeRewriteDetail 逐段确认卡，语义 1:1。
- **AiTaskRows（Task4）**：任务行列表（状态徽章/进度环/pill/trailing 插槽 + 受控展开），替换 Tasks 任务行外壳；展开区原样挂 TaskDetail/CascadeRewriteDetail，操作按钮与 mutations 不变。
- **AiSelectionActions（Task5）**：划词 AI 操作浮条（润色/扩写/改写 + 自定义指令，selection.getClientRects 定位 + 宽度动画 + mousedown 防选区塌陷），新增挂载 RichTextEditor；结果经既有 smartExecute 通路生成，浮条下面板流式显现，「保留」insertContentAt 替换选区；frontstage.css 补 `--shadow-float`（修复 P1 幕前组件阴影变量缺失）。

### 测试

- src-frontend `npx vitest run`：**<以 Task6 Step2 实际输出填写> passed / 3 skipped**（基线 487 + 本批新增 29）。
```

**(b) `PROJECT_STATUS.md`** — 在 `## ✅ 最近完成功能` 下、`### Unreleased - beautifului AI 原生组件 P1（生成体验五件套）（2026-08-12）`（L18）条目前插入：

```markdown
### Unreleased - beautifului AI 原生组件 P2（代理与任务五件套）（2026-08-12）

- **五组件入库** `components/ui/ai/`：AiContextCards（PromptCoverageBar 槽位清单）、AiToolChips（Tasks/Skills 筛选条）、AiRecommendationCard（级联改写逐段确认卡）、AiTaskRows（Tasks 任务行外壳）、AiSelectionActions（幕前划词浮条，smartExecute + insertContentAt 选区替换）。
- **修复**：frontstage.css 补 `--shadow-float`（P1 幕前组件弹出层阴影变量此前未定义）。
- **验证**：`npx tsc --noEmit` / `npx vitest run`（<实际数> passed / 3 skipped）/ `format:check` / `architecture_guard.py` 全绿；Rust 无改动。版本号未动，发版另行进行。
```

**(c) `AGENTS.md`** — 将编码风格节 L30 的 AI 原生组件行整体替换为：

```markdown
- **AI 原生组件**: `src-frontend/src/components/ui/ai/`（P1 生成体验：AiLoading/AiThinking/AiStreamingText/AiPromptBar/AiApprovalCard；P2 代理与任务：AiContextCards/AiToolChips/AiRecommendationCard/AiTaskRows/AiSelectionActions），只引用 `--ai-*` 语义令牌（幕后 tokens.css / 幕前 frontstage.css 各自定义），不写死颜色；动画用 tailwind.config.js 注册的 ai keyframes 工具类；受控组件，禁止引入自运行演示逻辑；组件内嵌私有动效（如 AiSelectionActions 的 SelectionStreamText）不复用为公共 API。
```

- [ ] **Step 4: Commit**

```bash
git add src-frontend/src/components/index.ts CHANGELOG.md PROJECT_STATUS.md AGENTS.md
git commit -m "docs: P2 AI 原生组件库 barrel 导出与文档同步（P2 Task6）"
```

---

## 全量回归清单（每个 Task 末尾必过；Task 6 Step2 为总闸）

| 检查项 | 命令 | 通过标准 |
| --- | --- | --- |
| 类型检查 | `cd src-frontend && npx tsc --noEmit` | 0 error |
| 组件单测（本 Task） | `npx vitest run <本 Task 测试路径>` | 全绿 |
| 全量前端测试 | `npx vitest run` | ≥487 基线，只允许增加（最终 ≥514） |
| 受影响既有测试 | Task1 加跑 `src/components/__tests__/PromptCoverageBar.test.tsx` | 3 passed 不回归 |
| 格式化 | `npm run format:check` | 通过 |
| 架构守卫 | `python3 scripts/architecture_guard.py`（仓库根） | 退出码 0 |
| Rust 测试 | 不重跑（纯前端阶段） | 基线 1326 passed / 2 ignored 不变 |
| Commit 规范 | 中文 conventional commit，不 --no-verify，不推送不打 tag | 每个 Task 独立 commit + 评审 |

## 文件清单汇总表

| 文件 | Task | 动作 |
| --- | --- | --- |
| `src-frontend/src/components/ui/ai/AiContextCards.tsx` | 1 | 新建 |
| `src-frontend/src/components/ui/ai/__tests__/AiContextCards.test.tsx` | 1 | 新建 |
| `src-frontend/src/components/PromptCoverageBar.tsx` | 1 | 修改（L6 import、L67-84 槽位清单） |
| `src-frontend/src/pages/AgencyStudio.tsx` | 1（可选） | 修改（L332-352 黑板条目） |
| `src-frontend/src/components/ui/ai/AiToolChips.tsx` | 2 | 新建 |
| `src-frontend/src/components/ui/ai/__tests__/AiToolChips.test.tsx` | 2 | 新建 |
| `src-frontend/src/pages/Tasks.tsx` | 2 / 3 / 4 | 修改（L661-680 筛选条 / L347-398 段落卡 / L74-252 行外壳 + L700-701 容器——三 Task 区域互不重叠） |
| `src-frontend/src/pages/Skills.tsx` | 2 | 修改（L277-288 分类条） |
| `src-frontend/src/components/ui/ai/AiRecommendationCard.tsx` | 3 | 新建 |
| `src-frontend/src/components/ui/ai/__tests__/AiRecommendationCard.test.tsx` | 3 | 新建 |
| `src-frontend/src/components/ui/ai/AiTaskRows.tsx` | 4 | 新建 |
| `src-frontend/src/components/ui/ai/__tests__/AiTaskRows.test.tsx` | 4 | 新建 |
| `src-frontend/src/components/ui/ai/AiSelectionActions.tsx` | 5 | 新建 |
| `src-frontend/src/components/ui/ai/__tests__/AiSelectionActions.test.tsx` | 5 | 新建 |
| `src-frontend/src/frontstage/components/RichTextEditor.tsx` | 5 | 修改（import L85 后、state L292 后、handler L949 后、渲染 L1408 前） |
| `src-frontend/src/frontstage/styles/frontstage.css` | 5 | 修改（L79 后补 `--shadow-float`） |
| `src-frontend/src/components/index.ts` | 6 | 修改（L24 后 barrel） |
| `CHANGELOG.md` / `PROJECT_STATUS.md` / `AGENTS.md` | 6 | 修改（文档同步） |

## 依赖顺序图

```
P1 令牌桥（--ai-* / ai-* 工具类 / keyframes，已入库）
  │
  ├─ Task1 AiContextCards ──→ PromptCoverageBar（+可选 AgencyStudio）
  ├─ Task2 AiToolChips ─────→ Tasks.tsx 筛选条、Skills.tsx 分类条
  ├─ Task3 AiRecommendationCard ──→ Tasks.tsx CascadeRewriteDetail（段落卡）
  ├─ Task4 AiTaskRows ──────→ Tasks.tsx TaskRow（行外壳；展开区复用 Task3 改后的 CascadeRewriteDetail）
  └─ Task5 AiSelectionActions ──→ RichTextEditor（复用 P1 segmentStreamText / animate-stream-in；
        │                          补 frontstage.css --shadow-float）
        └─ 依赖 P1 AiStreamingText 的导出函数（只 import，不修改）
  │
  └─ Task6 barrel + 全量回归门 + 文档同步（依赖 Task1-5 全部）

Tasks.tsx 内部顺序约束：Task2（L661-680）→ Task3（L347-398）→ Task4（L74-252+L700-701），
区域自底向上递减，逐个 commit 无冲突；Task3 不改 CascadeRewriteDetail 签名，Task4 不受影响。
```

## Self-Review 结论

- **Spec coverage（对照任务书 + 勘察结论 p2-recon-summary.md）**：
  - 五组件全部覆盖，集成点与任务书一致；AgencyStudio BoardItem 作为 Task1 可选 Step 6（独立 commit，不阻塞主线）。
  - 勘察结论逐条落实：无 liveline/glimm/iconoir-react；iconoir 10 图标映射表写入 Task5（实装 8 个，MessageCircleQuestion/Smile 对应动作本批不落，已注明）；Shimmer 按 AiThinking.tsx:66-73 复刻；StreamText 内嵌 ~30 行私有实现（SelectionStreamText），不动 P1 AiStreamingText；primitive-card-pad/footer/bar → tailwind 数值；裸 keyframes 内联引用 → animate-* 类 + animationDelay（Task4 details 行内 `fade-up` 例外已注明依据与回退方案）；无 dark:*；tint 系 → bg-ai-*/10；rounded/shadow 私有类 → rounded-[Npx]/border-ai-line/shadow-float；全部受控化。
  - Task 顺序按「简单先行、SelectionActions 殿后」；Tasks.tsx 三 Task 区域不重叠且顺序自底向上，冲突规避已写入 Global Constraints 与依赖图。
- **行号核实**：PromptCoverageBar L6/L25-36/L67-84、Tasks.tsx L36/L74-252/L254-401/L661-680/L700-701、Skills.tsx L32/L61/L277-288、AgencyStudio.tsx L332-352、RichTextEditor（frontstage/components/）L80/L85/L288-292/L691-713/L857-907/L949/L1299-1314/L1369-1389/L1408、EditorContextMenu.tsx:95-98、AiThinking.tsx:66-73、AiStreamingText.tsx:26（segmentStreamText 导出）、tailwind.config.js L71（shadow-float）/L126-129（fade-up keyframe）、tokens.css:67（--shadow-float 仅幕后）、frontstage.css:79（--ai-orange 锚点）/L611（阴影取值）、index.ts L5-24、CHANGELOG.md L5、PROJECT_STATUS.md L18、AGENTS.md L30——全部经 Read/Grep 实地核实。RichTextEditor 实际路径为 `src-frontend/src/frontstage/components/`（非任务书所写 `src-frontend/src/components/`），已按实际路径写入计划。
- **新决策（相对勘察结论）**：
  1. **SelectionActions 交互方案**：否决 ghost 管线接入（违反 P1「不改 FrontstageApp race-lock」约束且语义为光标追加）与底栏注入（props 钻孔 + 语义错误），选用组件内结果面板 + 宿主 `smartExecute`/`insertContentAt` 选区替换——零 FrontstageApp 改动、零后端改动、Keep/Discard 语义 1:1。
  2. **AiToolChips 语义裁剪**：参考实现本体（工具调用行）与筛选条集成点语义不符，明确提取 chip 视觉语法做受控单选组，差异已写入 Task2 说明，避免执行者照抄参考导致返工。
  3. **`--shadow-float` 幕前补齐**：勘察未提及 shadow-float 变量仅存在于幕后 tokens.css（frontstage.css 无定义，P1 幕前组件阴影实际失效）；Task5 Step3(b) 补值并注明对 P1 组件的修复性影响。
  4. **ContextCards chip 入场去定时器化**：chipsShown 700ms 定时器改为纯 CSS animationDelay，组件零 hooks（className 合并除外），更贴受控约定。
- **Placeholder scan**：全文无 TBD；唯一待定值为 CHANGELOG/PROJECT_STATUS 中最终测试计数 `<以 Task6 Step2 实际输出填写>`——设计上必须由执行者填入的真实命令输出，非占位符。
- **Type consistency**：
  - `AiRecommendationCardProps.status` 联合类型 `'pending'|'accepted'|'rejected'` 与 `RewriteSegment.user_decision`（Tasks.tsx L357-364 用法可证）一致，集成处无需类型断言。
  - `AiTaskRowItem<T>.payload` 泛型承载 `Task`（`@/hooks/useTasks` 导出类型，Tasks.tsx L30 import 可证），renderDetail 回传同行数据，TaskRow adapter 无类型断裂。
  - `AiSelectionActionsProps.onRun(action, customInstruction?)` 与测试断言 `toHaveBeenCalledWith('polish', undefined)` / `('custom', '改成古文腔')` 一致；`insertContentAt({from,to}, text)` 范围替换形式为 TipTap 既有 API（RichTextEditor L1094 单 pos 用法先例）。
  - AiContextCards 集成处 `tone` 用 `as const` 保持字面量类型；既有 PromptCoverageBar 测试断言文本（合同红线/KG摘要）由 item.title 继续提供。
