# P3 AI 原生组件库第三批（数据展示）Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 beautifului.dev 的 6 个 AI 原生数据展示组件（SearchList / CodeBlock / DiffTable / FilterTable / RecordsTable / InsightCards）适配为受控组件入库 `src-frontend/src/components/ui/ai/`，并逐点替换：PromptsPanel 搜索+计数区、六处裸 pre/JSON.stringify 代码块（TracingPanel/Logs×2/Mcp/Skills/IntentionGraphDiagnostics/PromptsPanel）、AgencyEval CheckpointCompare 检查点对比、UsageStats 分组 tabs + 最近调用表、PromptsPanel 分组行列表 + AgencyEval 判定历史/token 用量双表、UsageStats 四统计卡 + AgencyEval 三统计卡。

**Architecture:** 复用 P1/P2 令牌层（`--ai-*` 16 变量契约 + tailwind `ai-*` 色与 keyframes，本批**零扩令牌**：tint 缺口一律 color-mix 内联）→ 组件层（`components/ui/ai/`，全部受控、剥离演示状态机/计时器）→ 集成层（逐文件 before/after 替换）。组件只引用 `--ai-*` 语义令牌与 tailwind 注册的 keyframes 工具类，不写死颜色、不引新依赖（liveline 用 ~80 行自研 SVG MiniLineChart 静态快照替代）；图标用既有 `lucide-react`。

**Tech Stack:** React 18 + Tailwind v3.4（`var()` 色映射；注意 v3 无 `size-5.5`/`h-6.5` 动态间距，参考实现中的此类值一律改任意值 `size-[22px]`/`h-[26px]`）、vitest 4 + Testing Library、jsdom、lucide-react（既有依赖）。

## Global Constraints

- 仓库 /Users/yuzaimu/projects/StoryForge；master 直接工作；中文 conventional commit；不 --no-verify；不推送、不打 tag。
- **不引入新依赖**：禁止 `liveline` / `glimm` / `iconoir-react` / framer-motion；图标只用 `lucide-react`。InsightCards 的 liveline 依赖用组件内嵌私有 `MiniLineChart`（SVG polyline 静态快照，无 hover scrub、无实时动效）替代。
- **组件全部为受控组件**：剥离参考实现中的 ITEMS/LINES/RAW/ROWS/FILTERS/INITIAL_ROWS/TAG_COLORS/STRENGTH/PAGES 等演示数据与 useStage/useTick/自运行 setTimeout 步进逻辑；纯视图状态（复制反馈翻转、chips 交互）允许组件内部持有（同 P1 AiThinking manualExpanded、P2 AiApprovalCard 定时器先例）。
- **不改 P1/P2 已入库组件**：AiLoading/AiThinking/AiStreamingText/AiPromptBar/AiApprovalCard/AiContextCards/AiToolChips/AiRecommendationCard/AiTaskRows/AiSelectionActions 代码一行不动。
- **纯前端阶段**：不改任何 Rust 后端代码；`cargo test --lib` 基线 **1328 passed / 2 ignored** 不变，本批无需重跑。
- 本批全部集成点落在**幕后**窗口（tokens.css 提供 `--ai-*`），无幕前改动、无 CSS 文件改动。
- **AgencyEval 浅色裸样式页策略（同 P2 AgencyStudio 切口先例）**：AgencyEval.tsx 是浅色裸样式页（`text-gray-*`/`border` 直写，与 cinema 暗色主题不一致），接入的 AiDiffTable/AiRecordsTable/AiInsightCards 走 `--ai-*` 深色令牌，会与周边浅色裸样式形成对比——**声明为本批可接受的风格切口，P4 统一处理后台页令牌**，本批不为该页单独做浅色适配。既有 `AgencyEval.test.tsx` 只断言文本（`50%`、`gate-第1章-r1`、`writer`、`本故事累计（检查点）：42000 tokens / 2 runs`），不受颜色影响，必须保持全绿。
- 移植规则（勘察结论 + 本批实地核实，逐条执行）：tint 零扩令牌（`color-mix(in srgb, var(--ai-red) 12%, transparent)` 内联，不动 16 变量契约与 `src-frontend/src/styles/__tests__/aiTokens.test.ts`）；SearchList 的内联 `animation: fade-in …` 裸 keyframes → `animate-fade-in`/`animate-ai-fade-up` 类；FilterTable 的 `filter-status-*` 全局类缺失 → pill 经 `column.render` 插槽由调用方给出（同 P2 AiTaskRows pill 插槽思路），chips 圆点硬编码 hex 收进 props（`dot` 为 CSS 颜色值，宿主传 `var(--ai-*)`）；RecordsTable 的 `records-*` 约 25 个全局类（payload 未含定义）全部 Tailwind 自研内联；`rounded-card/control/chip`、`shadow-card/btn/hairline/raised` → `rounded-[Npx]`/`border-ai-line`；`primitive-card-bar/table-cell` → tailwind padding 数值；cubic-bezier 缓动保留内联；删全部 `dark:*` 变体（本批参考源码无 dark:，但 InsightCards 的 `useDarkMode` MutationObserver 整段删除——双窗口固定主题由 `--ai-*` 接管）。
- **行号口径**：本计划全部行号基于勘察基线（P2 完成后 master）。前置 Task 合并会引起后续文件行号漂移（UsageStats 被 Task4/Task6 触及、AgencyEval 被 Task3/Task5/Task6 触及、PromptsPanel 被 Task1/Task2/Task5 触及、Logs 被 Task2/Task4 触及）——执行时以**锚点代码内容**定位，行号仅作初始参考。各文件改动区域互不重叠（见依赖顺序图）。
- 组件风格约定（同 P1/P2）：文件头块注释（适配自 beautifului XXX + 受控约定 + 剥离了什么）、命名导出 + Props 导出 + 文件尾 default 导出、`ai-*` 令牌类、lucide 图标、每组件配 `__tests__` 测试、barrel 登记在 Task 7 统一做（`components/index.ts` P2 段 L25-51 后加 P3 分组，执行时以 AiSelectionActions 类型导出块之后、`export { DataLoader }` 之前为锚点）。
- 准入线：`cd src-frontend && npx tsc --noEmit && npx vitest run && npm run format:check` 全绿 + 仓库根 `python3 scripts/architecture_guard.py` 通过；vitest 基线 **523 passed / 3 skipped**（P2 完成后实测，见 PROJECT_STATUS.md v0.39.0 段），只允许增加。
- 设计文档：`docs/plans/2026-08-12-beautifului-ai-native-design.md`（§8 P3 范围，其中「AiChat」以勘察结论关闭，见 Task 7）；参考组件源码：`.superpowers/sdd/reference/beautifului/`；勘察结论：`.superpowers/sdd/p3-recon-summary.md`。

---

### Task 1: AiSearchList 组件 + PromptsPanel 搜索+计数区替换

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiSearchList.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiSearchList.test.tsx`
- Modify: `src-frontend/src/pages/settings/PromptsPanel.tsx`（搜索+分类行 L613-644、计数行 L646-650、空态 L779-785、handleClearSearch L414-416、import 区 L1-22）

**Interfaces:**
- Consumes: P1 令牌 `bg-ai-surface` / `bg-ai-inset` / `text-ai-ink` / `text-ai-ink-2` / `text-ai-ink-3` / `border-ai-line` / `border-ai-line-strong` / `hover:bg-ai-hover` / `animate-fade-in`（tailwind.config.js L90 `fadeIn 0.4s`）；`cn`（`@/utils/cn`，clsx + twMerge）；lucide `Search` / `X`
- Produces:
  - `export interface AiSearchListProps { value: string; onChange: (value: string) => void; placeholder?: string; ariaLabel?: string; resultCount?: number; emptyText?: string; emptyHint?: string; className?: string }`
  - `export function AiSearchList(props: AiSearchListProps): JSX.Element`；`data-testid="ai-search-list"`；计数行 `data-testid="ai-search-count"`；空态 `data-testid="ai-search-empty"`
- 语义裁剪说明（重要）：参考实现 SearchList 的本体是「输入即下拉结果列表，点击回填 query」的命令面板演示；PromptsPanel 的结果集是下方主列表（filteredEntries/grouped 既有过滤逻辑），下拉结果语义不符。本组件**提取其搜索框视觉语法**（图标 + 清除钮 fade-in + 空态卡）+ 计数行，结果为受控 `value/onChange`；ITEMS 演示数据与点击回填行为剥离。清除钮的 `animation: fade-in 150ms` 裸 keyframes → `animate-fade-in` 类；`size-5.5`（Tailwind v4 动态间距）→ `size-[22px]`。

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiSearchList.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiSearchList } from '../AiSearchList';

describe('AiSearchList', () => {
  it('渲染输入框（placeholder 与 aria-label）', () => {
    render(
      <AiSearchList value="" onChange={() => {}} placeholder="搜索提示词…" ariaLabel="搜索提示词" />
    );
    expect(screen.getByLabelText('搜索提示词')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('搜索提示词…')).toBeInTheDocument();
  });

  it('输入调用 onChange(新值)', () => {
    const onChange = vi.fn();
    render(<AiSearchList value="" onChange={onChange} />);
    fireEvent.change(screen.getByLabelText('搜索'), { target: { value: '写' } });
    expect(onChange).toHaveBeenCalledWith('写');
  });

  it('有值时渲染清除按钮，点击调用 onChange(空串)', () => {
    const onChange = vi.fn();
    render(<AiSearchList value="写作" onChange={onChange} />);
    fireEvent.click(screen.getByRole('button', { name: '清除搜索' }));
    expect(onChange).toHaveBeenCalledWith('');
  });

  it('空值时不渲染清除按钮与计数行', () => {
    render(<AiSearchList value="" onChange={() => {}} resultCount={3} />);
    expect(screen.queryByRole('button', { name: '清除搜索' })).not.toBeInTheDocument();
    expect(screen.queryByTestId('ai-search-count')).not.toBeInTheDocument();
  });

  it('有值且提供 resultCount>0 时渲染计数行', () => {
    render(<AiSearchList value="写作" onChange={() => {}} resultCount={5} />);
    expect(screen.getByTestId('ai-search-count')).toHaveTextContent('搜索 “写作” 找到 5 条结果');
  });

  it('resultCount=0 时渲染空态而非计数行', () => {
    render(
      <AiSearchList
        value="zzz"
        onChange={() => {}}
        resultCount={0}
        emptyText="未找到匹配的提示词"
        emptyHint="尝试调整搜索条件"
      />
    );
    expect(screen.getByTestId('ai-search-empty')).toBeInTheDocument();
    expect(screen.getByText('未找到匹配的提示词')).toBeInTheDocument();
    expect(screen.getByText('尝试调整搜索条件')).toBeInTheDocument();
    expect(screen.queryByTestId('ai-search-count')).not.toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiSearchList.test.tsx`
Expected: FAIL（组件不存在，import 报错）

- [ ] **Step 3: Write implementation**

**(a) `src-frontend/src/components/ui/ai/AiSearchList.tsx`** — 新建：

```tsx
/**
 * AiSearchList — 搜索框 + 结果计数/空态（适配自 beautifului SearchList）
 *
 * 受控约定：value/onChange/resultCount 全部由调用方提供，组件不含演示数据；
 * 剥离参考实现：ITEMS 演示数据、结果下拉列表与点击回填（集成点的结果集是
 * 宿主主列表，下拉语义不符）、min-h/max-w-72 演示尺寸。
 * 移植说明：内联 animation: fade-in 裸 keyframes → animate-fade-in 类
 * （tailwind.config.js L90 已注册 fadeIn）；size-5.5（Tailwind v4 动态间距）
 * → size-[22px]；rounded-card/shadow-raised/rounded-control/shadow-hairline →
 * rounded-[12px]/border-ai-line/rounded-[8px]；var(--ink-3) 等直引改 ai-* 令牌类。
 */
import { Search, X } from 'lucide-react';
import { cn } from '@/utils/cn';

export interface AiSearchListProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  ariaLabel?: string;
  /** 有搜索词且提供时渲染计数行；为 0 时渲染空态卡 */
  resultCount?: number;
  emptyText?: string;
  emptyHint?: string;
  className?: string;
}

export function AiSearchList({
  value,
  onChange,
  placeholder = '搜索…',
  ariaLabel = '搜索',
  resultCount,
  emptyText = '未找到匹配结果',
  emptyHint = '尝试调整搜索关键词',
  className,
}: AiSearchListProps) {
  const hasQuery = value.trim().length > 0;
  const empty = hasQuery && resultCount === 0;

  return (
    <div className={cn('flex w-full flex-col gap-2', className)} data-testid="ai-search-list">
      <div className="flex h-10 items-center gap-2 rounded-[12px] border border-ai-line bg-ai-surface px-3 transition-colors duration-100 focus-within:border-ai-line-strong hover:bg-ai-hover">
        <Search size={14} strokeWidth={2} aria-hidden className="shrink-0 text-ai-ink-3" />
        <input
          value={value}
          onChange={e => onChange(e.target.value)}
          placeholder={placeholder}
          aria-label={ariaLabel}
          className="min-w-0 flex-1 bg-transparent text-[13px] text-ai-ink outline-none placeholder:text-ai-ink-3"
        />
        {hasQuery && (
          <button
            type="button"
            aria-label="清除搜索"
            onClick={() => onChange('')}
            className="animate-fade-in flex size-[22px] shrink-0 items-center justify-center rounded-full text-ai-ink-3 transition-colors duration-100 hover:bg-ai-line/70 hover:text-ai-ink"
          >
            <X size={11} strokeWidth={2.2} aria-hidden />
          </button>
        )}
      </div>

      {hasQuery && typeof resultCount === 'number' && !empty && (
        <p className="animate-fade-in px-0.5 text-[12.5px] text-ai-ink-2" data-testid="ai-search-count">
          搜索 “{value}” 找到 <span className="tabular-nums">{resultCount}</span> 条结果
        </p>
      )}

      {empty && (
        <div
          className="animate-fade-in flex flex-col items-center justify-center gap-1 rounded-[12px] border border-ai-line bg-ai-surface px-4 py-8"
          data-testid="ai-search-empty"
        >
          <span className="mb-1.5 flex size-8 items-center justify-center rounded-[8px] border border-ai-line bg-ai-inset text-ai-ink-3">
            <Search size={15} strokeWidth={1.8} aria-hidden />
          </span>
          <span className="text-[13px] font-medium text-ai-ink">{emptyText}</span>
          <span className="text-[12px] text-ai-ink-3">{emptyHint}</span>
        </div>
      )}
    </div>
  );
}

export default AiSearchList;
```

**(b) `PromptsPanel.tsx`**：import 区（L21 `import { cn } from '@/utils/cn';` 行后）加：

```tsx
import { AiSearchList } from '@/components/ui/ai/AiSearchList';
```

搜索+计数区（现状 L612-650）：

```tsx
      {/* Search and Filter */}
      <div className="flex items-center gap-3 flex-wrap">
        <div className="relative flex-1 min-w-[200px]">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
          <input
            type="text"
            placeholder="搜索提示词 ID、名称、描述或内容..."
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
            className="w-full pl-9 pr-9 py-2 bg-cinema-900 border border-cinema-700 rounded text-sm text-white placeholder-gray-500"
          />
          {searchQuery && (
            <button
              onClick={handleClearSearch}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-white"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>
        <select
          value={activeCategory}
          onChange={e => setActiveCategory(e.target.value as PromptCategory | 'all')}
          className="px-3 py-2 bg-cinema-900 border border-cinema-700 rounded text-sm text-white"
        >
          <option value="all">全部分类</option>
          {CATEGORY_ORDER.map(cat => (
            <option key={cat} value={cat}>
              {CATEGORY_LABELS[cat]}
            </option>
          ))}
        </select>
      </div>

      {searchQuery && (
        <div className="text-sm text-gray-400">
          搜索 "{searchQuery}" 找到 {filteredEntries.length} 条结果
        </div>
      )}
```

替换为（分类 select 原样保留；计数行由组件接管；外层改 items-start 使 select 与输入框顶对齐、h-10 同高）：

```tsx
      {/* Search and Filter */}
      <div className="flex items-start gap-3 flex-wrap">
        <AiSearchList
          className="flex-1 min-w-[200px]"
          value={searchQuery}
          onChange={setSearchQuery}
          placeholder="搜索提示词 ID、名称、描述或内容..."
          ariaLabel="搜索提示词"
          resultCount={filteredEntries.length}
          emptyText="未找到匹配的提示词"
          emptyHint="尝试调整搜索条件或分类筛选"
        />
        <select
          value={activeCategory}
          onChange={e => setActiveCategory(e.target.value as PromptCategory | 'all')}
          className="h-10 px-3 bg-cinema-900 border border-cinema-700 rounded text-sm text-white"
        >
          <option value="all">全部分类</option>
          {CATEGORY_ORDER.map(cat => (
            <option key={cat} value={cat}>
              {CATEGORY_LABELS[cat]}
            </option>
          ))}
        </select>
      </div>
```

替换后清理：
- `handleClearSearch`（L414-416）不再被引用，整段删除（`useCallback` 仍被 fetchEntries/fetchComposition 等使用，import 保留）。
- lucide `X`（L9）若不再使用则从 import 删除（该 icon 在本文件仅用于已替换的清除钮；重置确认弹窗用的是 AlertTriangle）。
- lucide `Search`（L8）保留——底部空态 L781 仍用。
- 宿主大空态（L779-785）与组件空态在「搜索无结果」时会双显，给宿主空态加 `!searchQuery` 条件（仅在无搜索词且零条目时显示）：

```tsx
      {filteredEntries.length === 0 && !searchQuery && (
```

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiSearchList.test.tsx src/pages/settings/__tests__/PromptsPanel.test.tsx && npx tsc --noEmit`
Expected: AiSearchList 6 passed；PromptsPanel 既有 5 passed 不回归（本 Task 不动列表区，`data-prompt-id`/`prompt-editor` 链路不变）；tsc 干净

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiSearchList.tsx src-frontend/src/components/ui/ai/__tests__/AiSearchList.test.tsx src-frontend/src/pages/settings/PromptsPanel.tsx
git commit -m "feat: AiSearchList 组件入库并替换 PromptsPanel 搜索计数区（P3 Task1）"
```

---

### Task 2: AiCodeBlock 组件 + 六文件七处裸 pre/JSON.stringify 批量替换

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiCodeBlock.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiCodeBlock.test.tsx`
- Modify: `src-frontend/src/pages/TracingPanel.tsx`（step.details pre L140-144；import 区）
- Modify: `src-frontend/src/pages/Logs.tsx`（系统日志 pre L246-248；LogRow details pre L312-318；import 区 L12-22）
- Modify: `src-frontend/src/pages/Mcp.tsx`（toolResult pre L275-281；import 区）
- Modify: `src-frontend/src/pages/Skills.tsx`（executionResult pre L271-273；import 区）
- Modify: `src-frontend/src/pages/IntentionGraphDiagnostics.tsx`（plan_json pre L309-322 + result_json pre L324-337；import 区）
- Modify: `src-frontend/src/pages/settings/PromptsPanel.tsx`（内置默认值只读块 L717-726；import 区 Task 1 已加的 AiSearchList 行后）

**Interfaces:**
- Consumes: P1 令牌与 `animate-ai-fade-up`（tailwind.config.js L97）；`cn`；lucide `Check` / `Copy`
- Produces:
  - `export interface AiCodeBlockProps { code: string; language?: string; title?: string; lineNumbers?: boolean; maxHeight?: number; copyable?: boolean; className?: string }`
  - `export function AiCodeBlock(props: AiCodeBlockProps): JSX.Element`；`data-testid="ai-code-block"`
- 受控化映射：参考实现的 LINE_MS/HOLD_MS 逐行流式演示循环与 LINES/RAW 演示数据全部剥离；Tok 语法着色系统剥离（本批集成点均为 JSON/文本 dump，无着色需求；保留等宽字体 + `text-ai-ink-2` 单色）；复制按钮的 `copied` 1500ms 翻转为交互式 UI 反馈（非自运行演示），保留并加卸载清理（同 P2 AiApprovalCard 定时器先例）。`fade-up` 裸 keyframes 内联 → `animate-ai-fade-up` 类 + 错峰 animationDelay（封顶 20 行，防大日志块成百节点动画）；`max-w-95` 演示宽度删除。

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiCodeBlock.test.tsx
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, act } from '@testing-library/react';
import { AiCodeBlock } from '../AiCodeBlock';

const writeText = vi.fn().mockResolvedValue(undefined);

beforeEach(() => {
  vi.useFakeTimers();
  Object.assign(navigator, { clipboard: { writeText } });
  writeText.mockClear();
});

afterEach(() => {
  vi.useRealTimers();
});

const CODE = '{\n  "a": 1,\n  "b": 2\n}';

describe('AiCodeBlock', () => {
  it('渲染代码全部行', () => {
    render(<AiCodeBlock code={CODE} />);
    expect(screen.getByTestId('ai-code-block')).toHaveTextContent('"a": 1');
    expect(screen.getByTestId('ai-code-block')).toHaveTextContent('"b": 2');
  });

  it('渲染 title 与 language', () => {
    render(<AiCodeBlock code={CODE} title="结果" language="JSON" />);
    expect(screen.getByText('结果')).toBeInTheDocument();
    expect(screen.getByText('JSON')).toBeInTheDocument();
  });

  it('lineNumbers 时渲染行号 1-4', () => {
    render(<AiCodeBlock code={CODE} lineNumbers />);
    const block = screen.getByTestId('ai-code-block');
    for (const n of ['1', '2', '3', '4']) {
      expect(block.querySelector(`[data-line-no="${n}"]`)).toBeTruthy();
    }
  });

  it('点击复制写剪贴板并翻转已复制，1500ms 后恢复', async () => {
    render(<AiCodeBlock code={CODE} />);
    await act(async () => {
      fireEvent.click(screen.getByRole('button', { name: '复制' }));
    });
    expect(writeText).toHaveBeenCalledWith(CODE);
    expect(screen.getByRole('button', { name: '已复制' })).toBeInTheDocument();
    act(() => {
      vi.advanceTimersByTime(1600);
    });
    expect(screen.getByRole('button', { name: '复制' })).toBeInTheDocument();
  });

  it('copyable=false 时不渲染复制按钮', () => {
    render(<AiCodeBlock code={CODE} copyable={false} />);
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('maxHeight 应用到 pre 样式', () => {
    render(<AiCodeBlock code={CODE} maxHeight={192} />);
    const pre = screen.getByTestId('ai-code-block').querySelector('pre')!;
    expect(pre.style.maxHeight).toBe('192px');
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiCodeBlock.test.tsx`
Expected: FAIL（组件不存在）

- [ ] **Step 3: Write implementation**

**(a) `src-frontend/src/components/ui/ai/AiCodeBlock.tsx`** — 新建：

```tsx
/**
 * AiCodeBlock — 只读代码块（适配自 beautifului CodeBlock）
 *
 * 受控约定：code/language/title 全部由调用方提供；剥离参考实现的
 * LINE_MS/HOLD_MS 逐行流式演示循环、LINES/RAW 演示数据与 Tok 语法着色
 * （本批集成点均为 JSON/文本 dump，无着色需求）；复制反馈 copied 1500ms
 * 翻转为交互式 UI 状态保留（非自运行演示），带卸载清理。
 * 移植说明：rounded-card → rounded-[12px]、shadow-card → border-ai-line、
 * primitive-card-bar → px-3 py-2、fade-up 裸 keyframes → animate-ai-fade-up
 * （逐行错峰 animationDelay，封顶 20 行防大日志块动画节点爆炸）；
 * max-w-95 演示宽度限制删除（宽度由宿主决定）。
 */
import { useEffect, useRef, useState } from 'react';
import { Check, Copy } from 'lucide-react';
import { cn } from '@/utils/cn';

export interface AiCodeBlockProps {
  code: string;
  language?: string;
  title?: string;
  /** 行号（默认关） */
  lineNumbers?: boolean;
  /** px；超出滚动 */
  maxHeight?: number;
  copyable?: boolean;
  className?: string;
}

export function AiCodeBlock({
  code,
  language,
  title,
  lineNumbers = false,
  maxHeight,
  copyable = true,
  className,
}: AiCodeBlockProps) {
  const [copied, setCopied] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(
    () => () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    },
    []
  );

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => setCopied(false), 1500);
    } catch {
      // 剪贴板不可用（权限/非安全上下文）时静默
    }
  };

  const lines = code.split('\n');

  return (
    <div
      className={cn('w-full overflow-hidden rounded-[12px] border border-ai-line bg-ai-surface', className)}
      data-testid="ai-code-block"
    >
      <div className="flex items-center justify-between gap-2 border-b border-ai-line px-3 py-2">
        <span className="flex min-w-0 items-baseline gap-2">
          {title && (
            <span className="truncate font-mono text-[12px] font-medium text-ai-ink">{title}</span>
          )}
          {language && <span className="shrink-0 text-[11.5px] text-ai-ink-3">{language}</span>}
        </span>
        {copyable && (
          <button
            type="button"
            aria-label={copied ? '已复制' : '复制'}
            onClick={copy}
            className={cn(
              'flex h-6 shrink-0 items-center gap-1 rounded-[6px] px-1.5 text-[11.5px] font-medium transition-colors duration-100 hover:bg-ai-hover',
              copied ? 'text-ai-green' : 'text-ai-ink-3 hover:text-ai-ink'
            )}
          >
            {copied ? (
              <Check size={10} strokeWidth={3} aria-hidden />
            ) : (
              <Copy size={10} strokeWidth={2} aria-hidden />
            )}
            {copied ? '已复制' : '复制'}
          </button>
        )}
      </div>

      <pre
        className="overflow-auto bg-ai-inset px-3 py-2.5 font-mono text-[11.5px] leading-[1.7] text-ai-ink-2"
        style={maxHeight ? { maxHeight } : undefined}
      >
        {lineNumbers ? (
          lines.map((line, i) => (
            <div
              key={i}
              className="animate-ai-fade-up flex"
              style={{ animationDelay: `${Math.min(i, 20) * 25}ms` }}
            >
              <span
                data-line-no={i + 1}
                className="w-8 shrink-0 pr-2.5 text-right text-[10.5px] leading-[1.86] text-ai-ink-3/60 select-none"
              >
                {i + 1}
              </span>
              <span className="min-w-0 whitespace-pre-wrap break-all">{line}</span>
            </div>
          ))
        ) : (
          <code className="whitespace-pre-wrap break-all">{code}</code>
        )}
      </pre>
    </div>
  );
}

export default AiCodeBlock;
```

**(b) `TracingPanel.tsx`**：import 区加 `import { AiCodeBlock } from '@/components/ui/ai/AiCodeBlock';`。step.details 块（现状 L140-144）：

```tsx
        {!!step.details && (
          <pre className="mt-2 text-xs text-gray-500 bg-cinema-950 rounded-lg p-2 overflow-auto max-h-48">
            {JSON.stringify(step.details, null, 2)}
          </pre>
        )}
```

替换为：

```tsx
        {!!step.details && (
          <AiCodeBlock
            className="mt-2"
            code={JSON.stringify(step.details, null, 2)}
            language="JSON"
            maxHeight={192}
          />
        )}
```

**(c) `Logs.tsx`**：import 区（L22 lucide import 块后）加 `import { AiCodeBlock } from '@/components/ui/ai/AiCodeBlock';`。

系统日志 pre（现状 L246-248）：

```tsx
                <pre className="text-xs font-mono text-cinema-300 whitespace-pre-wrap break-all leading-relaxed">
                  {systemLogs.data || '暂无系统日志'}
                </pre>
```

替换为（原 pre 无 maxHeight，滚动由外层 Card 的 max-h 容器承担，不传 maxHeight；纯文本不传 language）：

```tsx
                <AiCodeBlock code={systemLogs.data || '暂无系统日志'} copyable={false} />
```

（复制能力已由页面顶部「复制」按钮 L126-129 提供，块内不重复。）

LogRow details pre（现状 L312-318）：

```tsx
      {expanded && hasDetails && (
        <div className="mt-1.5 ml-8">
          <pre className="text-xs font-mono text-cinema-400 bg-cinema-900/50 rounded p-2 overflow-x-auto">
            {JSON.stringify(entry.details, null, 2)}
          </pre>
        </div>
      )}
```

替换为（原 pre 为 nowrap 横向滚动，组件统一为 pre-wrap 折行——日志 details 场景折行更可读，接受此视觉差异）：

```tsx
      {expanded && hasDetails && (
        <div className="mt-1.5 ml-8">
          <AiCodeBlock code={JSON.stringify(entry.details, null, 2)} language="JSON" />
        </div>
      )}
```

**(d) `Mcp.tsx`**：import 区加 AiCodeBlock import。toolResult 块（现状 L275-281）：

```tsx
                {toolResult !== null && (
                  <div className="mt-4">
                    <label className="block text-sm text-gray-400 mb-2">结果:</label>
                    <pre className="bg-cinema-900 p-3 rounded-lg text-xs text-gray-300 overflow-auto max-h-60">
                      {JSON.stringify(toolResult, null, 2)}
                    </pre>
                  </div>
                )}
```

替换为（label 文案并入组件 title）：

```tsx
                {toolResult !== null && (
                  <AiCodeBlock
                    className="mt-4"
                    code={JSON.stringify(toolResult, null, 2)}
                    title="结果"
                    language="JSON"
                    maxHeight={240}
                  />
                )}
```

**(e) `Skills.tsx`**：import 区加 AiCodeBlock import（放在 P2 已加的 AiToolChips import 附近）。executionResult pre（现状 L271-273）：

```tsx
          <pre className="text-xs text-gray-300 bg-cinema-900/80 rounded p-3 overflow-auto max-h-48">
            {JSON.stringify(executionResult.result, null, 2)}
          </pre>
```

替换为：

```tsx
          <AiCodeBlock
            code={JSON.stringify(executionResult.result, null, 2)}
            language="JSON"
            maxHeight={192}
          />
```

**(f) `IntentionGraphDiagnostics.tsx`**：import 区加 AiCodeBlock import。两处 pre 的 try/parse IIFE 保留在宿主（数据整形是宿主职责），只换渲染。plan_json 块（现状 L309-322）与 result_json 块（现状 L324-337）结构同构，以 plan_json 为例：

```tsx
                {selectedGraph.plan_json && (
                  <div className="border-t border-cinema-800 pt-4">
                    <p className="text-sm text-gray-500 mb-2">执行计划</p>
                    <pre className="text-xs text-gray-300 bg-cinema-900/50 p-3 rounded-lg overflow-auto max-h-48">
                      {(() => {
                        try {
                          return JSON.stringify(JSON.parse(selectedGraph.plan_json!), null, 2);
                        } catch {
                          return selectedGraph.plan_json;
                        }
                      })()}
                    </pre>
                  </div>
                )}
```

替换为：

```tsx
                {selectedGraph.plan_json && (
                  <div className="border-t border-cinema-800 pt-4">
                    <p className="text-sm text-gray-500 mb-2">执行计划</p>
                    <AiCodeBlock
                      language="JSON"
                      maxHeight={192}
                      code={(() => {
                        try {
                          return JSON.stringify(JSON.parse(selectedGraph.plan_json!), null, 2);
                        } catch {
                          return selectedGraph.plan_json!;
                        }
                      })()}
                    />
                  </div>
                )}
```

result_json 块（L324-337）同理，title 文案「执行结果」的 `<p>` 保留，pre 换 AiCodeBlock。

**(g) `PromptsPanel.tsx`**：import 区（Task 1 已加的 AiSearchList import 行后）加 `import { AiCodeBlock } from '@/components/ui/ai/AiCodeBlock';`。内置默认值只读块（现状 L717-726，位于展开详情内）：

```tsx
                          {entry.is_overridden && (
                            <div className="space-y-1">
                              <div className="text-xs text-gray-500 font-medium">
                                内置默认值（只读）：
                              </div>
                              <div className="w-full px-3 py-2 bg-cinema-950 border border-cinema-800 rounded text-sm text-gray-400 font-mono max-h-32 overflow-y-auto whitespace-pre-wrap">
                                {entry.default_content}
                              </div>
                            </div>
                          )}
```

替换为（label 并入组件 title；内容为 prompt 文本非 JSON，不传 language）：

```tsx
                          {entry.is_overridden && (
                            <AiCodeBlock
                              code={entry.default_content}
                              title="内置默认值（只读）"
                              maxHeight={128}
                            />
                          )}
```

注意：此块位于 Task 5 将重构的分组行列表（L653-777）区域内——Task 5 的 renderDetail 须原样携带本步替换后的 AiCodeBlock 用法，不回退。

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiCodeBlock.test.tsx src/pages/settings/__tests__/PromptsPanel.test.tsx && npx tsc --noEmit`
Expected: AiCodeBlock 6 passed；PromptsPanel 5 passed 不回归（L717-726 块在展开详情内，既有测试 entry `is_overridden: false` 不触及）；tsc 干净（其余四文件无既有页面测试）

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiCodeBlock.tsx src-frontend/src/components/ui/ai/__tests__/AiCodeBlock.test.tsx src-frontend/src/pages/TracingPanel.tsx src-frontend/src/pages/Logs.tsx src-frontend/src/pages/Mcp.tsx src-frontend/src/pages/Skills.tsx src-frontend/src/pages/IntentionGraphDiagnostics.tsx src-frontend/src/pages/settings/PromptsPanel.tsx
git commit -m "feat: AiCodeBlock 组件入库并批量替换六文件七处裸 pre/JSON 块（P3 Task2）"
```

---

### Task 3: AiDiffTable 组件 + AgencyEval CheckpointCompare 行式对比替换

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiDiffTable.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiDiffTable.test.tsx`
- Modify: `src-frontend/src/pages/AgencyEval.tsx`（CheckpointCompare diff 展示 L86-117；select 行 L56-85 不动；文件头 import 区 L1-5）

**Interfaces:**
- Consumes: P1 令牌 `bg-ai-surface` / `bg-ai-inset` / `text-ai-ink` / `text-ai-ink-2` / `text-ai-ink-3` / `border-ai-line` / `hover:bg-ai-hover`；tint 用 `color-mix(in srgb, … 12%, transparent)` 内联（零扩令牌）；`cn`；lucide `ArrowUpRight` / `ArrowDownRight` / `Minus`
- Produces:
  - `export interface AiDiffRow { key: string; label: string; base: React.ReactNode; compare: React.ReactNode; delta: number; formatDelta?: (delta: number) => string; betterWhen?: 'higher' | 'lower' }`
  - `export interface AiDiffTableProps { title?: string; rows: AiDiffRow[]; baseLabel?: string; compareLabel?: string; className?: string }`
  - `export function AiDiffTable(props: AiDiffTableProps): JSX.Element`；`data-testid="ai-diff-table"`；Δ 徽章 `data-testid="ai-diff-delta"`
- 受控化映射：参考实现的 useStage 三段时序演示（plain → red tint → added row）与 ROWS/DOT 演示数据全部剥离；「删除行红 tint/新增行绿 tint」演示语法不落（集成点为指标 基准/对比/Δ 行式对比，无行增删场景）。Δ 着色语义为本组件新增：`delta=0` → `--ai-ink-3` + Minus；非零按 `betterWhen`（默认 `higher`）判定改善 → `--ai-green` / 恶化 → `--ai-red`，箭头方向随 delta 正负。`red-tint`/`green-tint` 无令牌 → color-mix 内联；`primitive-card-bar/table-cell` → tailwind 数值；`h-5.5` → `h-[22px]`。
- **数据来源决策（关键，已核实后端）**：`CheckpointDiff`（`services/api/agency.ts` L53-58）只有四个 delta 字段，无基准/对比绝对值；但 `AgencyCheckpoint.metrics_json`（同文件 L20-28）内含绝对值——key 与后端 `coordinator.rs` L516-519 对齐：`words_total` / `chapters_done` / `tokens_used` / `gate_scores[].weighted`（取末条）。宿主侧解析两个选中 checkpoint 的 metrics_json 提供基准/对比列，解析失败回退 `—`；**零后端改动**。

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiDiffTable.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AiDiffTable, type AiDiffRow } from '../AiDiffTable';

const rows: AiDiffRow[] = [
  { key: 'words', label: '字数', base: '42000', compare: '43500', delta: 1500 },
  { key: 'tokens', label: 'tokens', base: '8000', compare: '9100', delta: 1100, betterWhen: 'lower' },
  { key: 'weighted', label: '加权分', base: '0.80', compare: '0.80', delta: 0, formatDelta: d => d.toFixed(2) },
];

describe('AiDiffTable', () => {
  it('渲染标题与表头（指标/基准/对比/Δ）', () => {
    render(<AiDiffTable title="指标对比" rows={rows} />);
    expect(screen.getByText('指标对比')).toBeInTheDocument();
    for (const h of ['指标', '基准', '对比', 'Δ']) {
      expect(screen.getByText(h)).toBeInTheDocument();
    }
  });

  it('渲染每行 label/base/compare', () => {
    render(<AiDiffTable rows={rows} />);
    expect(screen.getByText('字数')).toBeInTheDocument();
    expect(screen.getByText('42000')).toBeInTheDocument();
    expect(screen.getByText('43500')).toBeInTheDocument();
  });

  it('delta 默认带 + 号；formatDelta 自定义优先', () => {
    render(<AiDiffTable rows={rows} />);
    expect(screen.getByText('+1500')).toBeInTheDocument();
    expect(screen.getByText('0.00')).toBeInTheDocument();
  });

  it('betterWhen=higher 且 delta>0 → 绿；delta<0 → 红', () => {
    render(<AiDiffTable rows={rows} />);
    const pill = screen.getByText('+1500').closest('span')!;
    expect(pill.style.color).toContain('var(--ai-green)');
  });

  it('betterWhen=lower 且 delta>0 → 红（恶化）', () => {
    render(<AiDiffTable rows={rows} />);
    const pill = screen.getByText('+1100').closest('span')!;
    expect(pill.style.color).toContain('var(--ai-red)');
  });

  it('delta=0 → 中性 ink-3', () => {
    render(<AiDiffTable rows={rows} />);
    const pill = screen.getByText('0.00').closest('span')!;
    expect(pill.style.color).toContain('var(--ai-ink-3)');
  });

  it('非零 delta 徽章底色为 color-mix tint（零扩令牌）', () => {
    render(<AiDiffTable rows={rows} />);
    const pill = screen.getByText('+1500').closest('span')!;
    expect(pill.style.background).toContain('color-mix(in srgb, var(--ai-green) 12%, transparent)');
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiDiffTable.test.tsx`
Expected: FAIL（组件不存在）

- [ ] **Step 3: Write implementation**

**(a) `src-frontend/src/components/ui/ai/AiDiffTable.tsx`** — 新建：

```tsx
/**
 * AiDiffTable — 指标基准/对比/Δ 行式对比表（适配自 beautifului DiffTable）
 *
 * 受控约定：title/rows/baseLabel/compareLabel 全部由调用方提供；剥离参考实现的
 * useStage 三段时序演示（plain→red tint→added row）、ROWS/DOT 演示数据与
 * 「删除行/新增行」演示语法（集成点无行增删场景）。
 * Δ 着色语义为本组件新增：delta=0 → --ai-ink-3；非零按 betterWhen（默认 higher）
 * 判定改善（--ai-green）/恶化（--ai-red），箭头随 delta 正负。
 * 移植说明：red-tint/green-tint 无令牌 → color-mix(in srgb, … 12%, transparent)
 * 内联（零扩令牌，不动 16 变量契约）；primitive-card-bar/table-cell → tailwind
 * 数值；rounded-card/shadow-card → rounded-[12px]/border-ai-line。
 */
import { ArrowDownRight, ArrowUpRight, Minus } from 'lucide-react';
import { cn } from '@/utils/cn';

export interface AiDiffRow {
  key: string;
  label: string;
  base: React.ReactNode;
  compare: React.ReactNode;
  /** 数值差（compare - base）；着色与箭头由 delta 正负 + betterWhen 决定 */
  delta: number;
  formatDelta?: (delta: number) => string;
  /** 默认 higher：delta>0 为改善；lower 反之（如 tokens 成本） */
  betterWhen?: 'higher' | 'lower';
}

export interface AiDiffTableProps {
  title?: string;
  rows: AiDiffRow[];
  baseLabel?: string;
  compareLabel?: string;
  className?: string;
}

const TONE_VAR = {
  good: 'var(--ai-green)',
  bad: 'var(--ai-red)',
  neutral: 'var(--ai-ink-3)',
} as const;

export function AiDiffTable({
  title,
  rows,
  baseLabel = '基准',
  compareLabel = '对比',
  className,
}: AiDiffTableProps) {
  return (
    <div
      className={cn('w-full overflow-hidden rounded-[12px] border border-ai-line bg-ai-surface', className)}
      data-testid="ai-diff-table"
    >
      {title && (
        <div className="border-b border-ai-line px-3 py-2">
          <span className="text-[12.5px] font-medium text-ai-ink">{title}</span>
        </div>
      )}
      <table className="w-full table-fixed border-collapse text-left">
        <colgroup>
          <col className="w-[28%]" />
          <col className="w-[24%]" />
          <col className="w-[24%]" />
          <col className="w-[24%]" />
        </colgroup>
        <thead>
          <tr className="border-b border-ai-line">
            {['指标', baseLabel, compareLabel, 'Δ'].map(h => (
              <th key={h} className="px-3 py-2 text-[12px] font-medium text-ai-ink-3">
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map(row => {
            const tone =
              row.delta === 0
                ? 'neutral'
                : (row.betterWhen ?? 'higher') === 'higher'
                  ? row.delta > 0
                    ? 'good'
                    : 'bad'
                  : row.delta < 0
                    ? 'good'
                    : 'bad';
            const color = TONE_VAR[tone];
            const Icon = row.delta === 0 ? Minus : row.delta > 0 ? ArrowUpRight : ArrowDownRight;
            const text = row.formatDelta
              ? row.formatDelta(row.delta)
              : `${row.delta >= 0 ? '+' : ''}${row.delta}`;
            return (
              <tr
                key={row.key}
                className="border-b border-ai-line transition-colors duration-150 last:border-0 hover:bg-ai-hover"
              >
                <td className="px-3 py-2 text-[13px] font-medium text-ai-ink">{row.label}</td>
                <td className="px-3 py-2 text-[12.5px] text-ai-ink-2 tabular-nums">{row.base}</td>
                <td className="px-3 py-2 text-[12.5px] text-ai-ink-2 tabular-nums">{row.compare}</td>
                <td className="px-3 py-2">
                  <span
                    data-testid="ai-diff-delta"
                    className="inline-flex h-[22px] items-center gap-1 rounded-full px-2 text-[12px] font-medium tabular-nums"
                    style={{
                      color,
                      background:
                        tone === 'neutral'
                          ? 'var(--ai-inset)'
                          : `color-mix(in srgb, ${color} 12%, transparent)`,
                    }}
                  >
                    <Icon size={12} strokeWidth={2.5} aria-hidden />
                    {text}
                  </span>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

export default AiDiffTable;
```

**(b) `AgencyEval.tsx`**：import 区（L5 `import type { GateHistoryItem } from '@/services/api/agency';` 行后）加：

```tsx
import { AiDiffTable } from '@/components/ui/ai/AiDiffTable';
```

文件头（CheckpointCompare 定义之前）加 metrics_json 解析助手（key 与后端 coordinator.rs L516-519 对齐，注释注明）：

```tsx
/** 解析 checkpoint metrics_json；key 与后端 agency/coordinator.rs compare_checkpoints 对齐
 *  （words_total / chapters_done / tokens_used / gate_scores 末条 weighted）。解析失败回退 null。 */
function parseCheckpointMetrics(json: string): Record<string, unknown> | null {
  try {
    return JSON.parse(json) as Record<string, unknown>;
  } catch {
    return null;
  }
}

function metricNumber(m: Record<string, unknown> | null, key: string): number | null {
  const v = m?.[key];
  return typeof v === 'number' ? v : null;
}

function metricWeighted(m: Record<string, unknown> | null): number | null {
  const scores = m?.gate_scores;
  if (!Array.isArray(scores) || scores.length === 0) return null;
  const last = scores[scores.length - 1] as Record<string, unknown> | undefined;
  return typeof last?.weighted === 'number' ? last.weighted : null;
}
```

CheckpointCompare 的 diff 展示（现状 L86-117）：

```tsx
      {diff && (
        <div className="mt-2 grid grid-cols-4 gap-2 text-center text-sm">
          <div className="rounded border p-2">
            <div className="text-gray-500">字数</div>
            <div>
              {diff.words_delta >= 0 ? '+' : ''}
              {diff.words_delta}
            </div>
          </div>
          <div className="rounded border p-2">
            <div className="text-gray-500">章节</div>
            <div>
              {diff.chapters_delta >= 0 ? '+' : ''}
              {diff.chapters_delta}
            </div>
          </div>
          <div className="rounded border p-2">
            <div className="text-gray-500">tokens</div>
            <div>
              {diff.tokens_delta >= 0 ? '+' : ''}
              {diff.tokens_delta}
            </div>
          </div>
          <div className="rounded border p-2">
            <div className="text-gray-500">加权分</div>
            <div>
              {diff.gate_weighted_delta >= 0 ? '+' : ''}
              {diff.gate_weighted_delta.toFixed(2)}
            </div>
          </div>
        </div>
      )}
```

替换为（基准/对比绝对值由两个选中 checkpoint 的 metrics_json 解析；缺失回退 `—`；tokens 按成本视角 `betterWhen="lower"`；select 行 L56-85 不动）：

```tsx
      {diff && (
        <AiDiffTable
          className="mt-2"
          title="指标对比"
          rows={(() => {
            const ma = parseCheckpointMetrics(
              checkpoints.find(c => c.id === a)?.metrics_json ?? ''
            );
            const mb = parseCheckpointMetrics(
              checkpoints.find(c => c.id === b)?.metrics_json ?? ''
            );
            const fmt = (v: number | null) => (v === null ? '—' : String(v));
            const fmtW = (v: number | null) => (v === null ? '—' : v.toFixed(2));
            return [
              {
                key: 'words',
                label: '字数',
                base: fmt(metricNumber(ma, 'words_total')),
                compare: fmt(metricNumber(mb, 'words_total')),
                delta: diff.words_delta,
              },
              {
                key: 'chapters',
                label: '章节',
                base: fmt(metricNumber(ma, 'chapters_done')),
                compare: fmt(metricNumber(mb, 'chapters_done')),
                delta: diff.chapters_delta,
              },
              {
                key: 'tokens',
                label: 'tokens',
                base: fmt(metricNumber(ma, 'tokens_used')),
                compare: fmt(metricNumber(mb, 'tokens_used')),
                delta: diff.tokens_delta,
                betterWhen: 'lower' as const,
              },
              {
                key: 'weighted',
                label: '加权分',
                base: fmtW(metricWeighted(ma)),
                compare: fmtW(metricWeighted(mb)),
                delta: diff.gate_weighted_delta,
                formatDelta: (d: number) => `${d >= 0 ? '+' : ''}${d.toFixed(2)}`,
              },
            ];
          })()}
        />
      )}
```

主题说明：AgencyEval 为浅色裸样式页，AiDiffTable 走 `--ai-*` 深色令牌与周边形成对比——Global Constraints 已声明为可接受切口（同 P2 AgencyStudio 先例），P4 统一。

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiDiffTable.test.tsx src/pages/__tests__/AgencyEval.test.tsx && npx tsc --noEmit`
Expected: AiDiffTable 7 passed；AgencyEval 既有 1 passed 不回归（mock `listCheckpoints` 返回 `[]`，CheckpointCompare 提前 return null，不触及本改动）；tsc 干净

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiDiffTable.tsx src-frontend/src/components/ui/ai/__tests__/AiDiffTable.test.tsx src-frontend/src/pages/AgencyEval.tsx
git commit -m "feat: AiDiffTable 组件入库并替换 AgencyEval 检查点对比（P3 Task3）"
```

---

### Task 4: AiFilterTable 组件 + UsageStats 分组 tabs + 最近调用表（+ 可选 Logs 级别筛选）

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiFilterTable.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiFilterTable.test.tsx`
- Modify: `src-frontend/src/pages/UsageStats.tsx`（分组 tabs L173-193；最近调用表 L282-331；operationCounts 派生；import 区 L1-17）
- Modify（可选 Step 6）: `src-frontend/src/pages/Logs.tsx`（级别筛选 L165-182；import 区）

**Interfaces:**
- Consumes: P1 令牌与 `animate-ai-fade-up`；`cn`
- Produces:
  - `export interface AiFilterChipItem { key: string; label: string; count?: number; dot?: string; mono?: boolean }`（`dot` 为 CSS 颜色值，宿主传 `var(--ai-*)`，硬编码 hex 已收进 props）
  - `export interface AiFilterChipsBarProps { items: AiFilterChipItem[]; activeKey: string; onSelect: (key: string) => void; ariaLabel?: string; className?: string }`
  - `export function AiFilterChipsBar(props): JSX.Element`（chips 条独立命名导出，Logs 等仅需 chips 的场景复用；`data-testid="ai-filter-chips"`）
  - `export interface AiFilterColumn<T> { key: string; label: React.ReactNode; align?: 'left' | 'right' | 'center'; width?: string; render: (row: T) => React.ReactNode }`
  - `export interface AiFilterTableProps<T> { chips?: AiFilterChipItem[]; activeChip?: string; onChipSelect?: (key: string) => void; chipsAriaLabel?: string; columns: AiFilterColumn<T>[]; rows: T[]; rowKey: (row: T) => React.Key; emptyText?: string; minWidth?: number; className?: string }`
  - `export function AiFilterTable<T>(props): JSX.Element`；`data-testid="ai-filter-table"`
- 受控化映射：参考实现内部 `filter` state 与 FILTERS/ROWS 演示数据剥离——**行过滤在宿主完成**（UsageStats 已有 `filteredCalls` useMemo，L122-125），组件只渲染 chips 条 + 已过滤行；`filter-status-*` 全局类缺失 → pill 经 `column.render` 插槽由调用方给出；行显隐的 grid 0fr/1fr 动画改 `animate-ai-fade-up` 错峰入场（封顶 12 行）；`h-6.5` → `h-[26px]`；`shadow-btn/shadow-card/rounded-card` → `border-ai-line`/`rounded-[12px]`；`scrollbarWidth: none` 内联 → `[scrollbar-width:none]` 任意属性类。

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiFilterTable.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiFilterTable, AiFilterChipsBar, type AiFilterColumn } from '../AiFilterTable';

interface Row {
  id: number;
  name: string;
  status: string;
}

const chips = [
  { key: 'all', label: '全部', count: 3 },
  { key: 'todo', label: '待办', dot: 'var(--ai-orange)', count: 2 },
  { key: 'done', label: '完成', mono: true, count: 1 },
];

const columns: AiFilterColumn<Row>[] = [
  { key: 'name', label: '名称', width: '2fr', render: r => r.name },
  { key: 'status', label: '状态', align: 'center', render: r => <span>{r.status}</span> },
];

const rows: Row[] = [
  { id: 1, name: '条目一', status: 'todo' },
  { id: 2, name: '条目二', status: 'done' },
];

describe('AiFilterChipsBar', () => {
  it('渲染全部 chips 与计数徽章，dot 颜色透传 style', () => {
    render(<AiFilterChipsBar items={chips} activeKey="all" onSelect={() => {}} />);
    expect(screen.getByText('全部')).toBeInTheDocument();
    expect(screen.getByText('3')).toBeInTheDocument();
    // 按钮内第一个 span 即 dot（getByText 命中的是 chip 按钮本体，故在按钮内部查）
    const dot = screen.getByRole('button', { name: /待办/ }).querySelector('span')!;
    expect(dot.style.background).toContain('var(--ai-orange)');
  });

  it('activeKey 对应 chip aria-pressed=true，点击调用 onSelect(key)', () => {
    const onSelect = vi.fn();
    render(<AiFilterChipsBar items={chips} activeKey="all" onSelect={onSelect} />);
    expect(screen.getByRole('button', { name: /全部/ })).toHaveAttribute('aria-pressed', 'true');
    fireEvent.click(screen.getByRole('button', { name: /待办/ }));
    expect(onSelect).toHaveBeenCalledWith('todo');
  });

  it('active chip 实心（bg-ai-surface + border-ai-line），mono chip 带 font-mono', () => {
    render(<AiFilterChipsBar items={chips} activeKey="all" onSelect={() => {}} />);
    expect(screen.getByRole('button', { name: /全部/ }).className).toContain('bg-ai-surface');
    expect(screen.getByRole('button', { name: /完成/ }).className).toContain('font-mono');
  });
});

describe('AiFilterTable', () => {
  it('渲染 chips + 表头 + 行（column.render 生效）', () => {
    render(
      <AiFilterTable
        chips={chips}
        activeChip="all"
        onChipSelect={() => {}}
        columns={columns}
        rows={rows}
        rowKey={r => r.id}
      />
    );
    expect(screen.getByText('名称')).toBeInTheDocument();
    expect(screen.getByText('条目一')).toBeInTheDocument();
    expect(screen.getByText('done')).toBeInTheDocument();
  });

  it('无 chips props 时只渲染表格（表格单用场景）', () => {
    render(<AiFilterTable columns={columns} rows={rows} rowKey={r => r.id} />);
    expect(screen.queryByTestId('ai-filter-chips')).not.toBeInTheDocument();
    expect(screen.getByText('条目二')).toBeInTheDocument();
  });

  it('空行渲染 emptyText；align=right 列带 text-right', () => {
    render(
      <AiFilterTable
        columns={[{ key: 'n', label: '数值', align: 'right', render: (r: Row) => r.name }]}
        rows={[]}
        rowKey={(r: Row) => r.id}
        emptyText="暂无 LLM 调用记录"
      />
    );
    expect(screen.getByText('暂无 LLM 调用记录')).toBeInTheDocument();
    expect(screen.getByText('数值').className).toContain('text-right');
  });

  it('行错峰 animationDelay 递增（封顶 12 行）', () => {
    const many = Array.from({ length: 15 }, (_, i) => ({ id: i, name: `r${i}`, status: 'x' }));
    render(<AiFilterTable columns={columns} rows={many} rowKey={r => r.id} />);
    const first = screen.getByText('r0').closest('.grid')! as HTMLElement;
    const last = screen.getByText('r14').closest('.grid')! as HTMLElement;
    expect(first.style.animationDelay).toBe('0ms');
    expect(last.style.animationDelay).toBe('480ms');
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiFilterTable.test.tsx`
Expected: FAIL（组件不存在）

- [ ] **Step 3: Write implementation**

**(a) `src-frontend/src/components/ui/ai/AiFilterTable.tsx`** — 新建：

```tsx
/**
 * AiFilterTable — 筛选 chips + 数据表（适配自 beautifului FilterTable）
 *
 * 受控约定：chips/activeChip/onChipSelect 与 columns/rows 全部由调用方提供，
 * 行过滤在宿主完成（参考实现内部 filter state 剥离）；剥离 FILTERS/ROWS 演示
 * 数据、filter-status-* 全局类（pill 经 column.render 插槽由调用方给出，同
 * P2 AiTaskRows pill 插槽思路）；chips 圆点硬编码 hex 收进 props（dot 为 CSS
 * 颜色值，宿主传 var(--ai-*)）。
 * 移植说明：行显隐 grid 0fr/1fr 动画 → animate-ai-fade-up 错峰入场（封顶 12 行）；
 * h-6.5 → h-[26px]；shadow-btn/shadow-card/rounded-card → border-ai-line/
 * rounded-[12px]；scrollbarWidth 内联 → [scrollbar-width:none]。
 * AiFilterChipsBar 为 chips 条的独立命名导出（仅需 chips 的场景复用，如 Logs 级别筛选）。
 */
import { cn } from '@/utils/cn';

export interface AiFilterChipItem {
  key: string;
  label: string;
  count?: number;
  /** CSS 颜色值（建议 var(--ai-*)） */
  dot?: string;
  mono?: boolean;
}

export interface AiFilterChipsBarProps {
  items: AiFilterChipItem[];
  activeKey: string;
  onSelect: (key: string) => void;
  ariaLabel?: string;
  className?: string;
}

export function AiFilterChipsBar({ items, activeKey, onSelect, ariaLabel, className }: AiFilterChipsBarProps) {
  return (
    <div
      role="group"
      aria-label={ariaLabel}
      className={cn('-mx-1 flex items-center gap-1 overflow-x-auto px-1 py-1 [scrollbar-width:none]', className)}
      data-testid="ai-filter-chips"
    >
      {items.map(item => {
        const active = item.key === activeKey;
        return (
          <button
            key={item.key}
            type="button"
            aria-pressed={active}
            onClick={() => onSelect(item.key)}
            className={cn(
              'flex h-[26px] shrink-0 items-center gap-1.5 rounded-full px-2.5 text-[12px] font-medium transition-[background-color,color] duration-200',
              item.mono && 'font-mono',
              active
                ? 'border border-ai-line bg-ai-surface text-ai-ink'
                : 'text-ai-ink-2 hover:bg-ai-hover'
            )}
          >
            {item.dot && (
              <span className="size-1.5 rounded-full" style={{ background: item.dot }} aria-hidden />
            )}
            {item.label}
            {typeof item.count === 'number' && (
              <span
                className={cn(
                  'rounded-[4px] px-1 text-[10.5px] tabular-nums',
                  active ? 'bg-ai-field text-ai-ink-2' : 'text-ai-ink-3'
                )}
              >
                {item.count}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}

export interface AiFilterColumn<T> {
  key: string;
  label: React.ReactNode;
  align?: 'left' | 'right' | 'center';
  /** grid 列宽（fr 或固定值），默认 1fr */
  width?: string;
  render: (row: T) => React.ReactNode;
}

export interface AiFilterTableProps<T> {
  /** chips 三件套同时提供时才渲染筛选条（表格可单用） */
  chips?: AiFilterChipItem[];
  activeChip?: string;
  onChipSelect?: (key: string) => void;
  chipsAriaLabel?: string;
  columns: AiFilterColumn<T>[];
  rows: T[];
  rowKey: (row: T) => React.Key;
  emptyText?: string;
  minWidth?: number;
  className?: string;
}

const ALIGN = { left: 'text-left', right: 'text-right', center: 'text-center' } as const;

export function AiFilterTable<T>({
  chips,
  activeChip,
  onChipSelect,
  chipsAriaLabel,
  columns,
  rows,
  rowKey,
  emptyText = '暂无数据',
  minWidth = 420,
  className,
}: AiFilterTableProps<T>) {
  const template = columns.map(c => c.width ?? '1fr').join(' ');
  return (
    <div className={cn('w-full', className)} data-testid="ai-filter-table">
      {chips && activeChip !== undefined && onChipSelect && (
        <AiFilterChipsBar
          items={chips}
          activeKey={activeChip}
          onSelect={onChipSelect}
          ariaLabel={chipsAriaLabel}
          className="mb-1"
        />
      )}
      <div
        role="region"
        aria-label="数据表（可横向滚动）"
        tabIndex={0}
        className="overflow-x-auto rounded-[12px] border border-ai-line bg-ai-surface [scrollbar-width:none]"
      >
        <div style={{ minWidth }}>
          <div
            className="grid border-b border-ai-line px-3 py-2 text-[11.5px] font-medium text-ai-ink-3"
            style={{ gridTemplateColumns: template }}
          >
            {columns.map(c => (
              <span key={c.key} className={ALIGN[c.align ?? 'left']}>
                {c.label}
              </span>
            ))}
          </div>
          {rows.length === 0 ? (
            <div className="px-3 py-8 text-center text-[12.5px] text-ai-ink-3">{emptyText}</div>
          ) : (
            rows.map((row, i) => (
              <div
                key={rowKey(row)}
                className="animate-ai-fade-up grid items-center border-b border-ai-line px-3 py-2 text-[12px] transition-colors duration-100 last:border-0 hover:bg-ai-hover"
                style={{
                  gridTemplateColumns: template,
                  animationDelay: `${Math.min(i, 12) * 40}ms`,
                }}
              >
                {columns.map(c => (
                  <span key={c.key} className={cn('min-w-0', ALIGN[c.align ?? 'left'])}>
                    {c.render(row)}
                  </span>
                ))}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

export default AiFilterTable;
```

**(b) `UsageStats.tsx`**：import 区（L5 `import { Card, CardContent } from '@/components/ui/Card';` 行后）加：

```tsx
import { AiFilterChipsBar, AiFilterTable } from '@/components/ui/ai/AiFilterTable';
```

派生各分组计数（放在 `filteredStats` useMemo 之后，L137 行后）：

```tsx
  const operationCounts = useMemo(() => {
    const counts: Record<OperationTab, number> = {
      all: recentCalls.length,
      bootstrap: 0,
      smart_execute: 0,
      other: 0,
    };
    for (const c of recentCalls) counts[deriveOperation(c)] += 1;
    return counts;
  }, [recentCalls]);
```

分组 tabs（现状 L173-193）：

```tsx
      <div className="flex flex-wrap items-center gap-2">
        {(['all', 'bootstrap', 'smart_execute', 'other'] as OperationTab[]).map(tab => (
          <button
            key={tab}
            onClick={() => setOperationTab(tab)}
            className={cn(
              'px-3 py-1.5 text-sm font-medium rounded-lg border transition-colors',
              operationTab === tab
                ? 'bg-cinema-gold/20 text-cinema-gold border-cinema-gold/30'
                : 'bg-cinema-900 border-cinema-700 text-cinema-300 hover:bg-cinema-800'
            )}
          >
            {TAB_LABELS[tab]}
          </button>
        ))}
        <span className="inline-flex items-center gap-1 text-xs text-cinema-500 ml-2">
          <Info className="w-3 h-3" />
          分组基于 purpose / task_type / metadata（含 JSON 中 operation、label
          等字段）关键词启发式推断
        </span>
      </div>
```

替换为（Info 提示行原样保留；新增各分组计数徽章——数据来自 operationCounts，不虚构）：

```tsx
      <div className="flex flex-wrap items-center gap-2">
        <AiFilterChipsBar
          ariaLabel="调用分组筛选"
          activeKey={operationTab}
          onSelect={key => setOperationTab(key as OperationTab)}
          items={(['all', 'bootstrap', 'smart_execute', 'other'] as OperationTab[]).map(tab => ({
            key: tab,
            label: TAB_LABELS[tab],
            count: operationCounts[tab],
          }))}
        />
        <span className="inline-flex items-center gap-1 text-xs text-cinema-500 ml-2">
          <Info className="w-3 h-3" />
          分组基于 purpose / task_type / metadata（含 JSON 中 operation、label
          等字段）关键词启发式推断
        </span>
      </div>
```

最近调用表（现状 L282-331，含空态分支与 overflow-x-auto 包裹）：

```tsx
          {filteredCalls.length === 0 ? (
            <div className="text-center py-8 text-gray-500">暂无 LLM 调用记录</div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                …（thead 7 列 + tbody filteredCalls.map，L286-329）…
              </table>
            </div>
          )}
```

整体替换为（空态由 `emptyText` 接管；表头「最近调用」+ 分组统计行 L271-280 与 Card 外壳不动）：

```tsx
          <AiFilterTable
            columns={[
              {
                key: 'purpose',
                label: '用途',
                width: '1.6fr',
                render: call => <span className="text-ai-ink">{call.purpose}</span>,
              },
              {
                key: 'operation',
                label: '操作',
                width: '0.8fr',
                render: call => (
                  <span className="text-ai-ink-2">{TAB_LABELS[deriveOperation(call)]}</span>
                ),
              },
              {
                key: 'model',
                label: '模型',
                width: '1fr',
                render: call => (
                  <span className="text-ai-ink-2">{call.model_name || call.model_id}</span>
                ),
              },
              {
                key: 'tokens',
                label: 'Token',
                align: 'right',
                width: '0.7fr',
                render: call => (
                  <span className="text-ai-ink-2 tabular-nums">
                    {call.total_tokens.toLocaleString()}
                  </span>
                ),
              },
              {
                key: 'duration',
                label: '耗时',
                align: 'right',
                width: '0.6fr',
                render: call => (
                  <span className="text-ai-ink-2 tabular-nums">
                    {call.duration_ms >= 1000
                      ? `${(call.duration_ms / 1000).toFixed(1)}s`
                      : `${call.duration_ms}ms`}
                  </span>
                ),
              },
              {
                key: 'status',
                label: '状态',
                align: 'center',
                width: '0.5fr',
                render: call =>
                  call.success ? (
                    <CheckCircle className="mx-auto h-4 w-4 text-ai-green" />
                  ) : (
                    <XCircle className="mx-auto h-4 w-4 text-ai-red" />
                  ),
              },
              {
                key: 'time',
                label: '时间',
                width: '1.1fr',
                render: call => (
                  <span className="text-xs text-ai-ink-3">
                    {new Date(call.created_at).toLocaleString()}
                  </span>
                ),
              },
            ]}
            rows={filteredCalls}
            rowKey={call => call.id}
            emptyText="暂无 LLM 调用记录"
          />
```

替换后 `cn` import（L2）若不再使用则删除（执行时以 tsc/eslint 未使用告警为准；L141 加载态仍在用 `cn` 则保留）。

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiFilterTable.test.tsx && npx tsc --noEmit`
Expected: 7 passed；tsc 干净（UsageStats 无既有页面测试）

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiFilterTable.tsx src-frontend/src/components/ui/ai/__tests__/AiFilterTable.test.tsx src-frontend/src/pages/UsageStats.tsx
git commit -m "feat: AiFilterTable 组件入库并替换 UsageStats 分组筛选与最近调用表（P3 Task4）"
```

- [ ] **Step 6（可选，时间允许时做，独立 commit）: Logs 级别筛选接入 AiFilterChipsBar**

勘察标注「评估工作量后决定是否同批」——工作量评估结论：级别筛选条与 AiFilterChipsBar 语义 1:1（单选 + mono 标签 + 可按级别计数），改动 ≤20 行，纳入本批可选 Step。Logs.tsx 搜索框（L185-193）与行数 select（L196-206）保持现状不动（搜索框已有深链 `logsSearchQuery` 逻辑 L53-59，不动为安）。

import 区（Task 2 已加的 AiCodeBlock import 行后）加 `import { AiFilterChipsBar } from '@/components/ui/ai/AiFilterTable';`。

级别筛选（现状 L165-182）：

```tsx
          {source === 'workflow' && (
            <div className="flex rounded-lg border border-cinema-700 overflow-hidden">
              {(['ALL', 'INFO', 'WARN', 'ERROR'] as LogLevel[]).map(l => (
                <button
                  key={l}
                  onClick={() => setLevel(l)}
                  className={cn(
                    'px-3 py-1.5 text-xs font-mono transition-colors',
                    level === l
                      ? 'bg-cinema-gold/20 text-cinema-gold'
                      : 'text-cinema-400 hover:bg-cinema-800'
                  )}
                >
                  {l}
                </button>
              ))}
            </div>
          )}
```

替换为（mono 保留；计数取自当前已加载日志，加载中 undefined 不渲染徽章）：

```tsx
          {source === 'workflow' && (
            <AiFilterChipsBar
              ariaLabel="日志级别筛选"
              activeKey={level}
              onSelect={key => setLevel(key as LogLevel)}
              items={(['ALL', 'INFO', 'WARN', 'ERROR'] as LogLevel[]).map(l => ({
                key: l,
                label: l,
                mono: true,
                count: workflowLogs.data
                  ? l === 'ALL'
                    ? workflowLogs.data.length
                    : workflowLogs.data.filter(e => e.level === l).length
                  : undefined,
              }))}
            />
          )}
```

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | tail -3`
Commit: `git add src-frontend/src/pages/Logs.tsx && git commit -m "feat: Logs 级别筛选接入 AiFilterChipsBar（P3 Task4 可选）"`

---

### Task 5: AiRecordsTable 组件 + PromptsPanel 分组行列表 + AgencyEval 双表

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiRecordsTable.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiRecordsTable.test.tsx`
- Modify: `src-frontend/src/pages/settings/PromptsPanel.tsx`（分组行列表 L664-773 内部；Card/头部 L656-663 保留；import 区 Task 2 已加的 AiCodeBlock 行后）
- Modify: `src-frontend/src/pages/AgencyEval.tsx`（判定历史表 L167-195；token 用量表 L203-222 + 排序派生；import 区 L1-5；文件头 OutcomePill 助手）

**Interfaces:**
- Consumes: P1 令牌 `bg-ai-surface` / `bg-ai-inset` / `bg-ai-field` / `bg-ai-hover` / `bg-ai-accent-tint` / `text-ai-ink` / `text-ai-ink-2` / `text-ai-ink-3` / `border-ai-line` / `border-ai-line-strong` / `animate-ai-fade-up`；`cn`；lucide `ArrowDown` / `Check` / `ChevronDown`
- Produces:
  - `export interface AiRecordsColumn<T> { key: string; label: React.ReactNode; icon?: React.ReactNode; align?: 'left' | 'right' | 'center'; width?: string; sortable?: boolean; render: (row: T) => React.ReactNode }`
  - `export interface AiRecordsSort { key: string; dir: 1 | -1 }`
  - `export interface AiRecordsTableProps<T> { columns: AiRecordsColumn<T>[]; rows: T[]; rowKey: (row: T) => string; selectable?: boolean; selectedKeys?: ReadonlySet<string>; onSelectionChange?: (next: Set<string>) => void; sort?: AiRecordsSort | null; onSortChange?: (sort: AiRecordsSort) => void; expandedKey?: string | null; onRowToggle?: (key: string) => void; renderDetail?: (row: T) => React.ReactNode; rowKeyAttribute?: string; footer?: React.ReactNode; emptyText?: string; ariaLabel?: string; className?: string }`
  - `export function AiRecordsTable<T>(props): JSX.Element`；`data-testid="ai-records-table"`；展开详情 `data-testid="ai-records-detail"`；排序箭头 `data-testid="ai-records-sort-{column.key}"`
- 受控化映射（勘察结论逐条落实）：**全受控** rows/selectedKeys/onSelectionChange/sort/onSortChange——排序与过滤都在宿主完成（参考实现内部 useState 剥离）；INITIAL_ROWS/TAG_COLORS/STRENGTH 演示数据剥离；`records-*` 约 25 个站点全局类（payload 未含定义）全部 Tailwind 自研内联；**Checkbox 自研含 mixed**（原生 input `indeterminate` 经 ref 设置 + `sr-only` + 自绘框）；tfoot 计算行 → 可选 `footer` 插槽。
- 相对参考的三处裁剪/新增（本 Task 核心设计决策，已评估）：
  1. **剥离** sticky 首列、真实链接列、tag 列、「Add calculation」交互——P3 三个宿主表均为窄表且无链接/tag 场景；tag/link 风格可由 `column.render` 自行给出，组件不内置。
  2. **新增受控展开行**（`expandedKey`/`onRowToggle`/`renderDetail`）——PromptsPanel 需保留展开编辑器；展开行**仅 open 时挂载**（`animate-ai-fade-up` 入场），不用 grid 0fr/1fr 常驻挂载（避免折叠态常驻重型 textarea 编辑器 DOM）。
  3. **新增 `rowKeyAttribute`**——为 tr 输出 `data-{attr}={key}`，兼容 PromptsPanel 既有测试选择器 `[data-prompt-id="writer_system"] button`（tr 的展开列 chevron 是第一个后代 button，点击即 toggle，测试零改动）。
- selection/sort 能力落地说明：selection 在 P3 无宿主使用（PromptsPanel/AgencyEval 均不需要勾选），组件能力 + 单测覆盖保留（后续 AgencyLearning 观察表等可直接启用）；sort 由 AgencyEval token 用量表真实启用（sortable 列 + 宿主 useMemo 排序）。

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiRecordsTable.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AiRecordsTable, type AiRecordsColumn } from '../AiRecordsTable';

interface Row {
  id: string;
  name: string;
  tokens: number;
}

const rows: Row[] = [
  { id: 'r1', name: '条目一', tokens: 8000 },
  { id: 'r2', name: '条目二', tokens: 3000 },
];

const columns: AiRecordsColumn<Row>[] = [
  { key: 'name', label: '名称', render: r => <span>{r.name}</span> },
  { key: 'tokens', label: '总 tokens', align: 'right', sortable: true, render: r => <span>{r.tokens}</span> },
];

describe('AiRecordsTable', () => {
  it('渲染表头与行（column.render 生效），空行渲染 emptyText', () => {
    const { rerender } = render(<AiRecordsTable columns={columns} rows={rows} rowKey={r => r.id} />);
    expect(screen.getByText('名称')).toBeInTheDocument();
    expect(screen.getByText('条目一')).toBeInTheDocument();
    rerender(<AiRecordsTable columns={columns} rows={[]} rowKey={r => r.id} emptyText="暂无判定记录" />);
    expect(screen.getByText('暂无判定记录')).toBeInTheDocument();
  });

  it('onRowToggle：点击行与 chevron 各触发一次（chevron 不双触发）', () => {
    const onToggle = vi.fn();
    render(
      <AiRecordsTable columns={columns} rows={rows} rowKey={r => r.id}
        onRowToggle={onToggle} renderDetail={r => <div>详情-{r.name}</div>} />
    );
    fireEvent.click(screen.getByText('条目一'));
    expect(onToggle).toHaveBeenCalledWith('r1');
    fireEvent.click(screen.getAllByRole('button', { name: '展开' })[1]);
    expect(onToggle).toHaveBeenCalledWith('r2');
    expect(onToggle).toHaveBeenCalledTimes(2);
  });

  it('expandedKey 行挂载 renderDetail，chevron aria-expanded=true；未展开行不挂载详情', () => {
    render(
      <AiRecordsTable columns={columns} rows={rows} rowKey={r => r.id}
        expandedKey="r2" onRowToggle={() => {}} renderDetail={r => <div>详情-{r.name}</div>} />
    );
    expect(screen.getByText('详情-条目二')).toBeInTheDocument();
    expect(screen.queryByText('详情-条目一')).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: '收起' })).toHaveAttribute('aria-expanded', 'true');
  });

  it('rowKeyAttribute 输出 data-{attr}={key} 到行 tr', () => {
    const { container } = render(
      <AiRecordsTable columns={columns} rows={rows} rowKey={r => r.id} rowKeyAttribute="prompt-id" />
    );
    expect(container.querySelector('tr[data-prompt-id="r1"]')).toBeTruthy();
  });

  it('selectable：行勾选与全选走 onSelectionChange，部分选中时全选框 indeterminate', () => {
    const onSelectionChange = vi.fn();
    render(
      <AiRecordsTable columns={columns} rows={rows} rowKey={r => r.id}
        selectable selectedKeys={new Set(['r1'])} onSelectionChange={onSelectionChange} />
    );
    const allBox = screen.getByLabelText('全选') as HTMLInputElement;
    expect(allBox.indeterminate).toBe(true);
    fireEvent.click(allBox);
    expect(onSelectionChange).toHaveBeenCalledWith(new Set(['r1', 'r2']));
    fireEvent.click(screen.getByLabelText('选择 r2'));
    expect(onSelectionChange).toHaveBeenCalledWith(new Set(['r1', 'r2']));
  });

  it('全选态下点击全选框清空选择', () => {
    const onSelectionChange = vi.fn();
    render(
      <AiRecordsTable columns={columns} rows={rows} rowKey={r => r.id}
        selectable selectedKeys={new Set(['r1', 'r2'])} onSelectionChange={onSelectionChange} />
    );
    fireEvent.click(screen.getByLabelText('全选'));
    expect(onSelectionChange).toHaveBeenCalledWith(new Set());
  });

  it('sortable 列表头点击调用 onSortChange（同 key 翻转 dir），箭头仅 active 列可见', () => {
    const onSortChange = vi.fn();
    const { container } = render(
      <AiRecordsTable columns={columns} rows={rows} rowKey={r => r.id}
        sort={{ key: 'tokens', dir: 1 }} onSortChange={onSortChange} />
    );
    fireEvent.click(screen.getByRole('button', { name: /总 tokens/ }));
    expect(onSortChange).toHaveBeenCalledWith({ key: 'tokens', dir: -1 });
    expect(container.querySelector('[data-testid="ai-records-sort-tokens"]')!.className).toContain('opacity-100');
  });

  it('th 输出 aria-sort；footer 插槽渲染在 tfoot', () => {
    render(
      <AiRecordsTable columns={columns} rows={rows} rowKey={r => r.id}
        sort={{ key: 'tokens', dir: -1 }} onSortChange={() => {}}
        footer={<span>合计 11000 tokens</span>} />
    );
    expect(screen.getByRole('columnheader', { name: /总 tokens/ })).toHaveAttribute('aria-sort', 'descending');
    expect(screen.getByText('合计 11000 tokens')).toBeInTheDocument();
  });

  it('非 sortable 列表头不渲染按钮', () => {
    render(
      <AiRecordsTable columns={columns} rows={rows} rowKey={r => r.id}
        sort={null} onSortChange={() => {}} />
    );
    expect(screen.queryByRole('button', { name: /名称/ })).not.toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiRecordsTable.test.tsx`
Expected: FAIL（组件不存在）

- [ ] **Step 3: Write implementation**

**(a) `src-frontend/src/components/ui/ai/AiRecordsTable.tsx`** — 新建：

```tsx
/**
 * AiRecordsTable — 记录表格（适配自 beautifului RecordsTable）
 *
 * 受控约定：columns/rows/selection/sort/expanded 全部由调用方提供（排序与过滤在
 * 宿主完成）；剥离参考实现的 INITIAL_ROWS/TAG_COLORS/STRENGTH 演示数据与内部
 * useState。records-* 约 25 个站点全局类（payload 未含定义）全部 Tailwind 自研。
 * 裁剪：sticky 首列、真实链接列、tag 列、「Add calculation」交互（P3 宿主无
 * 对应场景，tag/link 可由 column.render 自行给出）。
 * 新增（相对参考）：
 * - 受控展开行 expandedKey/onRowToggle/renderDetail（PromptsPanel 展开编辑器），
 *   详情仅 open 时挂载（animate-ai-fade-up 入场），避免折叠态常驻重型编辑器 DOM；
 * - rowKeyAttribute：tr 输出 data-{attr}={key}，兼容宿主既有测试选择器。
 * Checkbox 自研含 mixed（indeterminate 经 ref 设置）；tfoot 计算行 → 可选 footer 插槽。
 */
import { Fragment } from 'react';
import { ArrowDown, Check, ChevronDown } from 'lucide-react';
import { cn } from '@/utils/cn';

export interface AiRecordsColumn<T> {
  key: string;
  label: React.ReactNode;
  icon?: React.ReactNode;
  align?: 'left' | 'right' | 'center';
  /** col 宽度（固定值如 '120px'；不设则均分） */
  width?: string;
  sortable?: boolean;
  render: (row: T) => React.ReactNode;
}

export interface AiRecordsSort {
  key: string;
  dir: 1 | -1;
}

export interface AiRecordsTableProps<T> {
  columns: AiRecordsColumn<T>[];
  rows: T[];
  rowKey: (row: T) => string;
  /** 三件套同时提供才出现勾选列（selection 全受控） */
  selectable?: boolean;
  selectedKeys?: ReadonlySet<string>;
  onSelectionChange?: (next: Set<string>) => void;
  /** 受控排序：组件只渲染表头 UI 并回调，排序本身在宿主 */
  sort?: AiRecordsSort | null;
  onSortChange?: (sort: AiRecordsSort) => void;
  /** 受控展开行（onRowToggle + renderDetail 同时提供才出现展开列） */
  expandedKey?: string | null;
  onRowToggle?: (key: string) => void;
  renderDetail?: (row: T) => React.ReactNode;
  /** tr 输出 data-{rowKeyAttribute}={key}（宿主既有测试选择器兼容） */
  rowKeyAttribute?: string;
  /** tfoot 计算行的受控化：可选 footer 插槽 */
  footer?: React.ReactNode;
  emptyText?: string;
  ariaLabel?: string;
  className?: string;
}

function Checkbox({
  checked,
  mixed = false,
  onChange,
  label,
}: {
  checked: boolean;
  mixed?: boolean;
  onChange: () => void;
  label: string;
}) {
  return (
    <label
      className="inline-flex shrink-0 cursor-pointer items-center"
      title={label}
      onClick={e => e.stopPropagation()}
    >
      <input
        type="checkbox"
        className="sr-only"
        checked={checked}
        ref={el => {
          if (el) el.indeterminate = mixed;
        }}
        onChange={onChange}
        aria-label={label}
      />
      <span
        aria-hidden
        className={cn(
          'flex size-4 items-center justify-center rounded-[5px] border transition-colors duration-150',
          checked || mixed
            ? 'border-ai-ink bg-ai-ink text-ai-surface'
            : 'border-ai-line-strong bg-ai-surface'
        )}
      >
        {mixed ? (
          <span className="h-[2px] w-2 rounded-full bg-current" />
        ) : checked ? (
          <Check size={12} strokeWidth={3} />
        ) : null}
      </span>
    </label>
  );
}

export function AiRecordsTable<T>({
  columns,
  rows,
  rowKey,
  selectable = false,
  selectedKeys,
  onSelectionChange,
  sort = null,
  onSortChange,
  expandedKey = null,
  onRowToggle,
  renderDetail,
  rowKeyAttribute,
  footer,
  emptyText = '暂无数据',
  ariaLabel = '记录表格（可滚动查看全部列与记录）',
  className,
}: AiRecordsTableProps<T>) {
  const showSelection = selectable && !!onSelectionChange;
  const expandable = Boolean(onRowToggle && renderDetail);
  const colCount = columns.length + (showSelection ? 1 : 0) + (expandable ? 1 : 0);

  const selected = selectedKeys ?? new Set<string>();
  const allSelected = rows.length > 0 && rows.every(r => selected.has(rowKey(r)));
  const partiallySelected = !allSelected && rows.some(r => selected.has(rowKey(r)));

  const toggleRow = (key: string) => {
    if (!onSelectionChange) return;
    const next = new Set(selected);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    onSelectionChange(next);
  };
  const toggleAll = () => {
    if (!onSelectionChange) return;
    const next = new Set(selected);
    if (allSelected) rows.forEach(r => next.delete(rowKey(r)));
    else rows.forEach(r => next.add(rowKey(r)));
    onSelectionChange(next);
  };
  const clickSort = (key: string) => {
    if (!onSortChange) return;
    onSortChange(
      sort && sort.key === key ? { key, dir: (sort.dir * -1) as 1 | -1 } : { key, dir: 1 }
    );
  };

  return (
    <div
      className={cn('w-full overflow-hidden rounded-[12px] border border-ai-line bg-ai-surface', className)}
      data-testid="ai-records-table"
    >
      <div className="overflow-auto" tabIndex={0} aria-label={ariaLabel}>
        <table className="w-full border-collapse text-left">
          <colgroup>
            {showSelection && <col className="w-8" />}
            {columns.map(c => (
              <col key={c.key} style={c.width ? { width: c.width } : undefined} />
            ))}
            {expandable && <col className="w-10" />}
          </colgroup>
          <thead>
            <tr className="border-b border-ai-line">
              {showSelection && (
                <th className="px-3 py-2">
                  <Checkbox
                    checked={allSelected}
                    mixed={partiallySelected}
                    onChange={toggleAll}
                    label="全选"
                  />
                </th>
              )}
              {columns.map(c => {
                const active = sort?.key === c.key;
                return (
                  <th
                    key={c.key}
                    className={cn(
                      'px-3 py-2 text-[11.5px] font-medium text-ai-ink-3',
                      c.align === 'right' && 'text-right',
                      c.align === 'center' && 'text-center'
                    )}
                    aria-sort={active ? (sort!.dir === 1 ? 'ascending' : 'descending') : undefined}
                  >
                    {c.sortable && onSortChange ? (
                      <button
                        type="button"
                        onClick={() => clickSort(c.key)}
                        className={cn(
                          'inline-flex items-center gap-1 transition-colors hover:text-ai-ink',
                          c.align === 'right' && 'flex-row-reverse'
                        )}
                      >
                        {c.icon}
                        <span className="truncate">{c.label}</span>
                        <ArrowDown
                          size={12}
                          strokeWidth={2.2}
                          aria-hidden
                          data-testid={`ai-records-sort-${c.key}`}
                          className={cn(
                            'transition-[transform,opacity] duration-200',
                            active ? 'opacity-100' : 'opacity-0'
                          )}
                          style={{ transform: active && sort!.dir === -1 ? 'rotate(180deg)' : undefined }}
                        />
                      </button>
                    ) : (
                      <span className="inline-flex items-center gap-1">
                        {c.icon}
                        <span className="truncate">{c.label}</span>
                      </span>
                    )}
                  </th>
                );
              })}
              {expandable && <th className="px-2 py-2" aria-label="展开列" />}
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 && (
              <tr>
                <td colSpan={colCount} className="px-3 py-8 text-center text-[12.5px] text-ai-ink-3">
                  {emptyText}
                </td>
              </tr>
            )}
            {rows.map(row => {
              const key = rowKey(row);
              const open = expandedKey === key;
              const isSelected = selected.has(key);
              const keyAttr = rowKeyAttribute ? { [`data-${rowKeyAttribute}`]: key } : {};
              return (
                <Fragment key={key}>
                  <tr
                    {...keyAttr}
                    onClick={onRowToggle ? () => onRowToggle(key) : undefined}
                    className={cn(
                      'border-b border-ai-line text-[12.5px] transition-colors duration-100',
                      onRowToggle && 'cursor-pointer',
                      isSelected ? 'bg-ai-accent-tint' : 'hover:bg-ai-hover'
                    )}
                  >
                    {showSelection && (
                      <td className="px-3 py-2">
                        <Checkbox
                          checked={isSelected}
                          onChange={() => toggleRow(key)}
                          label={`选择 ${key}`}
                        />
                      </td>
                    )}
                    {columns.map(c => (
                      <td
                        key={c.key}
                        className={cn(
                          'px-3 py-2 text-ai-ink-2',
                          c.align === 'right' && 'text-right',
                          c.align === 'center' && 'text-center'
                        )}
                      >
                        {c.render(row)}
                      </td>
                    ))}
                    {expandable && (
                      <td className="px-2 py-2">
                        <button
                          type="button"
                          aria-label={open ? '收起' : '展开'}
                          aria-expanded={open}
                          onClick={e => {
                            e.stopPropagation();
                            onRowToggle!(key);
                          }}
                          className="flex size-6 items-center justify-center rounded-full text-ai-ink-3 transition-colors hover:bg-ai-hover hover:text-ai-ink"
                        >
                          <ChevronDown
                            size={14}
                            strokeWidth={2.2}
                            aria-hidden
                            className="transition-transform duration-300"
                            style={{ transform: open ? 'rotate(180deg)' : undefined }}
                          />
                        </button>
                      </td>
                    )}
                  </tr>
                  {expandable && open && (
                    <tr>
                      <td colSpan={colCount} className="border-b border-ai-line p-0">
                        <div className="animate-ai-fade-up" data-testid="ai-records-detail">
                          {renderDetail!(row)}
                        </div>
                      </td>
                    </tr>
                  )}
                </Fragment>
              );
            })}
          </tbody>
          {footer && (
            <tfoot>
              <tr className="border-t border-ai-line bg-ai-inset">
                <td colSpan={colCount} className="px-3 py-2">
                  {footer}
                </td>
              </tr>
            </tfoot>
          )}
        </table>
      </div>
    </div>
  );
}

export default AiRecordsTable;
```

**(b) `PromptsPanel.tsx`**：import 区（Task 2 已加的 AiCodeBlock import 行后）加：

```tsx
import { AiRecordsTable } from '@/components/ui/ai/AiRecordsTable';
```

分组行列表（现状 L664-773，即每个分类 Card 内的 `divide-y` 容器与其 `list.map(entry => …)` 全部内容；外层 Card L656-657、分类头部 L658-663、CardContent 收尾 L774-775 保留）：

```tsx
              <div className="divide-y divide-cinema-700">
                {list.map(entry => {
                  const isExpanded = expandedId === entry.id;
                  const draft = edited[entry.id] ?? entry.current_content;
                  const isDirty = draft !== entry.current_content;
                  return (
                    <div key={entry.id} className="px-4 py-3" data-prompt-id={entry.id}>
                      <button …（行头 L671-699）…</button>
                      {isExpanded && (
                        <div className="mt-3 space-y-3">
                          …（变量行 L703-715 / Task2 已替换的 AiCodeBlock L717-726 /
                             textarea L729-740 / 底部按钮行 L742-767）…
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
```

替换为（展开编辑器原样移入 `renderDetail`；draft/isDirty 在 renderDetail 内重算；`data-prompt-id` 经 `rowKeyAttribute` 保留；已覆盖/未保存徽章与名称/ID 并入「名称」列；描述列 truncate；组件自带外壳，`className="rounded-none border-0"` 去除与外层 Card 的嵌套边框——`cn` 为 twMerge，冲突类后者生效）：

```tsx
              <AiRecordsTable
                className="rounded-none border-0"
                ariaLabel={`${CATEGORY_LABELS[cat] ?? category}提示词列表`}
                rowKeyAttribute="prompt-id"
                rows={list}
                rowKey={e => e.id}
                expandedKey={expandedId}
                onRowToggle={id => setExpandedId(expandedId === id ? null : id)}
                columns={[
                  {
                    key: 'name',
                    label: '名称',
                    width: '45%',
                    render: entry => (
                      <span className="flex items-center gap-2 flex-wrap">
                        <span className="text-[13px] text-ai-ink">{entry.name}</span>
                        <code className="font-mono text-[11px] text-ai-ink-3">{entry.id}</code>
                        {entry.is_overridden && (
                          <span className="rounded bg-ai-orange/10 px-2 py-0.5 text-[11px] text-ai-orange">
                            已覆盖
                          </span>
                        )}
                        {(edited[entry.id] ?? entry.current_content) !== entry.current_content && (
                          <span className="rounded bg-ai-accent-tint px-2 py-0.5 text-[11px] text-ai-accent-ink">
                            未保存
                          </span>
                        )}
                      </span>
                    ),
                  },
                  {
                    key: 'description',
                    label: '描述',
                    render: entry => (
                      <span className="block truncate text-[12px] text-ai-ink-3">
                        {entry.description}
                      </span>
                    ),
                  },
                ]}
                renderDetail={entry => {
                  const draft = edited[entry.id] ?? entry.current_content;
                  const isDirty = draft !== entry.current_content;
                  return (
                    <div className="space-y-3 bg-ai-inset px-4 py-3">
                      {entry.variables.length > 0 && (
                        <div className="flex flex-wrap gap-1 text-[12px] text-ai-ink-2">
                          <span>支持的模板变量：</span>
                          {entry.variables.map(v => (
                            <code
                              key={v}
                              className="rounded bg-ai-field px-1.5 py-0.5 font-mono text-[11px] text-ai-accent-ink"
                            >
                              {VAR_TAG_OPEN + v + VAR_TAG_CLOSE}
                            </code>
                          ))}
                        </div>
                      )}

                      {/* Task2 已替换为 AiCodeBlock，原样携带 */}
                      {entry.is_overridden && (
                        <AiCodeBlock
                          code={entry.default_content}
                          title="内置默认值（只读）"
                          maxHeight={128}
                        />
                      )}

                      {/* v0.26.38: 原生 textarea，避免 Monaco CDN 被 CSP 拦截导致永久 Loading */}
                      <textarea
                        data-testid="prompt-editor"
                        value={draft}
                        onChange={e =>
                          setEdited(prev => ({
                            ...prev,
                            [entry.id]: e.target.value,
                          }))
                        }
                        className="h-[360px] w-full resize-y rounded-[8px] border border-ai-line bg-ai-surface px-3 py-2 font-mono text-[13px] leading-relaxed text-ai-ink focus:border-ai-accent/50 focus:outline-none"
                        spellCheck={false}
                      />

                      <div className="flex items-center justify-between">
                        <span className="text-[12px] text-ai-ink-3">
                          {draft.length} 字符 · {draft.split('\n').length} 行
                        </span>
                        <div className="flex items-center gap-2">
                          {entry.is_overridden && (
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={() => handleReset(entry.id)}
                            >
                              <RotateCcw className="mr-1 h-3.5 w-3.5" />
                              重置默认
                            </Button>
                          )}
                          <Button
                            size="sm"
                            onClick={() => handleSaveOverride(entry.id)}
                            disabled={!isDirty || savingId === entry.id}
                            isLoading={savingId === entry.id}
                          >
                            <Save className="mr-1 h-3.5 w-3.5" />
                            保存覆盖
                          </Button>
                        </div>
                      </div>
                    </div>
                  );
                }}
              />
```

替换后清理：lucide `ChevronDown`/`ChevronRight`（L3-4）在行头替换后若不再使用则从 import 删除（以 tsc/eslint 未使用告警为准）。既有测试链路保持：`[data-prompt-id="writer_system"] button` 命中展开列 chevron → `onRowToggle` → `expandedId` 翻转 → renderDetail 挂载 `prompt-editor`（textarea value 语义不变）。

**(c) `AgencyEval.tsx`**：import 区（L5 行后，Task 3 已加的 AiDiffTable import 附近）加：

```tsx
import { AiRecordsTable, type AiRecordsSort } from '@/components/ui/ai/AiRecordsTable';
```

L1 改 `import { useMemo, useState } from 'react';`；L5 类型 import 加 `PurposeUsage`：

```tsx
import type { GateHistoryItem, PurposeUsage } from '@/services/api/agency';
```

文件头（parseCheckpointMetrics 助手之后）加 outcome 徽章助手（tint 用 color-mix 内联，零扩令牌）：

```tsx
/** 判定结果徽章：pass 绿 / revise 橙 / 其他红（color-mix tint，零扩令牌） */
function OutcomePill({ outcome }: { outcome: string }) {
  const color =
    outcome === 'pass'
      ? 'var(--ai-green)'
      : outcome === 'revise'
        ? 'var(--ai-orange)'
        : 'var(--ai-red)';
  return (
    <span
      className="inline-flex items-center rounded-full px-2 py-0.5 text-[11px] font-medium"
      style={{ color, background: `color-mix(in srgb, ${color} 12%, transparent)` }}
    >
      {outcome}
    </span>
  );
}
```

主组件内（`const { data, isLoading, error } = useQuery(...)` 之后）加 token 用量排序派生：

```tsx
  const [usageSort, setUsageSort] = useState<AiRecordsSort>({ key: 'total_tokens', dir: -1 });
```

（`sortedUsage` 不能放在早退 return 之后用 hook——早退 return 在 L132-135，hooks 必须在之前。**将 useState/useMemo 移到 `if (!currentStory)` 早退之前**，data 可能为 undefined，用可选链：）

```tsx
  const [usageSort, setUsageSort] = useState<AiRecordsSort>({ key: 'total_tokens', dir: -1 });
  const sortedUsage = useMemo(() => {
    const list = data?.token_usage ?? [];
    const key = usageSort.key as keyof PurposeUsage;
    return [...list].sort((a, b) => {
      const av = a[key];
      const bv = b[key];
      const cmp =
        typeof av === 'number' && typeof bv === 'number'
          ? av - bv
          : String(av).localeCompare(String(bv));
      return cmp * usageSort.dir;
    });
  }, [data?.token_usage, usageSort]);
```

判定历史表（现状 L167-195）：

```tsx
      <section>
        <h2 className="mb-2 font-medium">判定历史</h2>
        <table className="w-full text-sm">
          <thead>
            <tr className="text-left text-gray-500">
              <th>条目</th><th>结果</th><th>加权</th><th>code</th><th>rule</th><th>model</th><th>时间</th>
            </tr>
          </thead>
          <tbody>
            {data.gate_history.map(g => (
              <tr key={g.key + g.created_at} className="border-t">
                <td>{g.key}</td>
                <td>{g.outcome}</td>
                <td>{g.weighted?.toFixed(2) ?? '—'}</td>
                <td>{g.code?.toFixed(2) ?? '—'}</td>
                <td>{g.rule?.toFixed(2) ?? '—'}</td>
                <td>{g.model?.toFixed(2) ?? '—'}</td>
                <td className="text-gray-400">{g.created_at.slice(0, 16)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
```

替换为（`<section>`/`<h2>` 保留，只换 table 本体）：

```tsx
      <section>
        <h2 className="mb-2 font-medium">判定历史</h2>
        <AiRecordsTable
          ariaLabel="判定历史"
          rows={data.gate_history}
          rowKey={g => g.key + g.created_at}
          emptyText="暂无判定记录"
          columns={[
            {
              key: 'key',
              label: '条目',
              width: '30%',
              render: g => <span className="font-medium text-ai-ink">{g.key}</span>,
            },
            { key: 'outcome', label: '结果', render: g => <OutcomePill outcome={g.outcome} /> },
            {
              key: 'weighted',
              label: '加权',
              align: 'right',
              render: g => <span className="tabular-nums">{g.weighted?.toFixed(2) ?? '—'}</span>,
            },
            {
              key: 'code',
              label: 'code',
              align: 'right',
              render: g => <span className="tabular-nums">{g.code?.toFixed(2) ?? '—'}</span>,
            },
            {
              key: 'rule',
              label: 'rule',
              align: 'right',
              render: g => <span className="tabular-nums">{g.rule?.toFixed(2) ?? '—'}</span>,
            },
            {
              key: 'model',
              label: 'model',
              align: 'right',
              render: g => <span className="tabular-nums">{g.model?.toFixed(2) ?? '—'}</span>,
            },
            {
              key: 'time',
              label: '时间',
              render: g => <span className="text-ai-ink-3">{g.created_at.slice(0, 16)}</span>,
            },
          ]}
        />
      </section>
```

token 用量表（现状 L203-222）同理替换（「本故事累计（检查点）」说明行 L199-201 与 `<section>`/`<h2>` 保留——既有测试断言该文案；footer 插槽给按角色合计行，排序三列 sortable）：

```tsx
        <AiRecordsTable
          ariaLabel="Agency token 用量（按角色，全局）"
          rows={sortedUsage}
          rowKey={u => u.purpose}
          sort={usageSort}
          onSortChange={setUsageSort}
          emptyText="暂无 token 用量记录"
          columns={[
            {
              key: 'purpose',
              label: '角色',
              width: '34%',
              render: u => (
                <span className="font-medium text-ai-ink">{u.purpose.replace('agency_', '')}</span>
              ),
            },
            {
              key: 'calls',
              label: '调用',
              align: 'right',
              sortable: true,
              render: u => <span className="tabular-nums">{u.calls}</span>,
            },
            {
              key: 'total_tokens',
              label: '总 tokens',
              align: 'right',
              sortable: true,
              render: u => <span className="tabular-nums">{u.total_tokens}</span>,
            },
            {
              key: 'total_duration_ms',
              label: '总耗时(ms)',
              align: 'right',
              sortable: true,
              render: u => <span className="tabular-nums">{u.total_duration_ms}</span>,
            },
          ]}
          footer={
            <span className="text-[12px] text-ai-ink-3">
              按角色合计：{data.token_usage.reduce((s, u) => s + u.calls, 0)} 次调用 ·{' '}
              {data.token_usage.reduce((s, u) => s + u.total_tokens, 0)} tokens ·{' '}
              {data.token_usage.reduce((s, u) => s + u.total_duration_ms, 0)}ms
            </span>
          }
        />
```

主题说明：同 Task 3，AgencyEval 浅色页接深色令牌组件为已声明切口。既有测试断言 `gate-第1章-r1`（名称列 render 提供）、`writer`（purpose.replace 提供）、`本故事累计（检查点）：42000 tokens / 2 runs`（未动的 L199-201 提供），不回归。

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiRecordsTable.test.tsx src/pages/settings/__tests__/PromptsPanel.test.tsx src/pages/__tests__/AgencyEval.test.tsx && npx tsc --noEmit`
Expected: AiRecordsTable 9 passed；PromptsPanel 5 passed 不回归；AgencyEval 1 passed 不回归；tsc 干净

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiRecordsTable.tsx src-frontend/src/components/ui/ai/__tests__/AiRecordsTable.test.tsx src-frontend/src/pages/settings/PromptsPanel.tsx src-frontend/src/pages/AgencyEval.tsx
git commit -m "feat: AiRecordsTable 组件入库并替换 PromptsPanel 分组列表与 AgencyEval 双表（P3 Task5）"
```

---

### Task 6: AiInsightCards 组件 + UsageStats 四统计卡 + AgencyEval 三统计卡

**Files:**
- Create: `src-frontend/src/components/ui/ai/AiInsightCards.tsx`
- Test: `src-frontend/src/components/ui/ai/__tests__/AiInsightCards.test.tsx`
- Modify: `src-frontend/src/pages/UsageStats.tsx`（统计卡 grid L196-266；tokenSeries 派生；import 区 lucide L6-16 行后）
- Modify: `src-frontend/src/pages/AgencyEval.tsx`（统计卡 grid L140-160；import 区）

**Interfaces:**
- Consumes: P1 令牌与 `animate-ai-fade-up`；`cn`；React `useId`（SVG gradient id）
- Produces:
  - `export interface AiInsightCardItem { key: string; label: string; value: string; sub?: string; icon?: React.ReactNode; tone?: 'accent' | 'green' | 'orange' | 'red' | 'neutral'; series?: number[]; seriesLabel?: string }`
  - `export interface AiInsightCardsProps { items: AiInsightCardItem[]; columns?: 2 | 3 | 4; className?: string }`
  - `export function AiInsightCards(props): JSX.Element`；`data-testid="ai-insight-cards"`；图表 `data-testid="ai-insight-chart"`
- **形态决策（本 Task 核心，相对勘察结论落定）**：导出**受控卡片组**（grid 壳 + 统计卡），**不落 AiInsightCarousel 分页壳**——参考实现的 PAGES/autoplay/「Insights N ‹ ›」分页器/blur crossfade 占位（InsightCards.tsx L436-439 写死 `opacity:1, blur(0)`）/pill CTA 均为演示逻辑，且本批两个宿主都是固定 grid 统计卡、无分页场景；差异特性（分页 + prose + CTA）如未来需要可作组合件复用本组件。liveline 依赖 → 组件内嵌私有 `MiniLineChart`（SVG polyline + 面积渐变 + 末点，**静态快照模式**：无 hover scrub、无 tooltip、无实时动效；`insight-chart-*` 全局类随之剥离）；`useDarkMode` MutationObserver 整段删除（双窗口固定主题由 `--ai-*` 接管）；序列色 hex 映射：`#f68f3c`→`--ai-orange`、`#3d9aff`→`--ai-accent`、`#ee5c61`→`--ai-red`（组件内 tone 映射，宿主不感知 hex）。
- 受控化映射：CompareCard/AnomalyCard/AllocationCard 三演示卡与 makePoints 演示数据剥离；统计卡 = label + value + sub + 可选 icon + 可选 series（时间正序数值序列）。卡入场 `animate-ai-fade-up` 错峰。

- [ ] **Step 1: Write the failing test**

```tsx
// src-frontend/src/components/ui/ai/__tests__/AiInsightCards.test.tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Hash } from 'lucide-react';
import { AiInsightCards, type AiInsightCardItem } from '../AiInsightCards';

const items: AiInsightCardItem[] = [
  { key: 'calls', label: '总调用次数', value: '128', sub: '本故事: 40', tone: 'accent', icon: <Hash size={20} /> },
  { key: 'tokens', label: '总 Token 数', value: '12.4K', tone: 'neutral', series: [3, 5, 4, 8, 6], seriesLabel: 'token 趋势' },
  { key: 'cost', label: '预估费用', value: '$0.42', tone: 'green' },
];

describe('AiInsightCards', () => {
  it('渲染每卡的 label/value/sub', () => {
    render(<AiInsightCards items={items} />);
    expect(screen.getByText('总调用次数')).toBeInTheDocument();
    expect(screen.getByText('128')).toBeInTheDocument();
    expect(screen.getByText('本故事: 40')).toBeInTheDocument();
    expect(screen.getByText('$0.42')).toBeInTheDocument();
  });

  it('有 series 的卡渲染 SVG 折线（polyline 路径 + 末点），无 series 不渲染图表', () => {
    render(<AiInsightCards items={items} />);
    const charts = screen.getAllByTestId('ai-insight-chart');
    expect(charts).toHaveLength(1);
    expect(charts[0].querySelector('path[stroke]')).toBeTruthy();
    expect(charts[0].querySelector('circle')).toBeTruthy();
    expect(charts[0]).toHaveAttribute('aria-label', 'token 趋势');
  });

  it('series 折线颜色映射 tone（neutral → --ai-ink-3）', () => {
    render(<AiInsightCards items={items} />);
    const path = screen.getByTestId('ai-insight-chart').querySelector('path[stroke]')!;
    expect(path.getAttribute('stroke')).toBe('var(--ai-ink-3)');
  });

  it('series=[1,1] 零跨度数据不 NaN（y 坐标有效）', () => {
    render(<AiInsightCards items={[{ key: 'f', label: 'l', value: 'v', series: [1, 1] }]} />);
    const path = screen.getByTestId('ai-insight-chart').querySelector('path[stroke]')!;
    expect(path.getAttribute('d')).not.toContain('NaN');
  });

  it('columns=3 时使用三列响应式类；卡入场错峰 animationDelay 递增', () => {
    render(<AiInsightCards items={items} columns={3} />);
    const grid = screen.getByTestId('ai-insight-cards');
    expect(grid.className).toContain('md:grid-cols-3');
    const cards = grid.children;
    expect((cards[0] as HTMLElement).style.animationDelay).toBe('0ms');
    expect((cards[1] as HTMLElement).style.animationDelay).toBe('80ms');
  });

  it('sub 色调映射 tone（green → --ai-green）', () => {
    render(
      <AiInsightCards items={[{ key: 'c', label: 'l', value: 'v', sub: '改善', tone: 'green' }]} />
    );
    expect(screen.getByText('改善').style.color).toContain('var(--ai-green)');
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiInsightCards.test.tsx`
Expected: FAIL（组件不存在）

- [ ] **Step 3: Write implementation**

**(a) `src-frontend/src/components/ui/ai/AiInsightCards.tsx`** — 新建：

```tsx
/**
 * AiInsightCards — 统计洞察卡片组（适配自 beautifului InsightCards）
 *
 * 受控约定：items/columns 全部由调用方提供（label/value/sub/icon/tone/series
 * 均为 props）；剥离参考实现的 CompareCard/AnomalyCard/AllocationCard 三演示卡、
 * makePoints 演示数据、PAGES/autoplay/「Insights N ‹ ›」分页壳、blur crossfade
 * 占位（L436-439 写死 opacity:1/blur(0)，随分页壳一并剥离）与 pill CTA。
 * liveline 依赖 → 内嵌私有 MiniLineChart（SVG polyline 静态快照：无 hover
 * scrub、无 tooltip、无实时动效；insight-chart-* 全局类随之剥离）；
 * useDarkMode MutationObserver 删除（双窗口固定主题由 --ai-* 接管）；
 * 序列色 hex 映射 ai-orange/ai-accent/ai-red（tone 映射，宿主不感知 hex）。
 */
import { useId } from 'react';
import { cn } from '@/utils/cn';

export interface AiInsightCardItem {
  key: string;
  label: string;
  value: string;
  sub?: string;
  icon?: React.ReactNode;
  /** 标签/icon/sub/折线的语义色（默认 neutral） */
  tone?: 'accent' | 'green' | 'orange' | 'red' | 'neutral';
  /** 静态快照迷你折线（时间正序数值序列；不提供则不渲染图表） */
  series?: number[];
  seriesLabel?: string;
}

export interface AiInsightCardsProps {
  items: AiInsightCardItem[];
  columns?: 2 | 3 | 4;
  className?: string;
}

const TONE_COLOR: Record<NonNullable<AiInsightCardItem['tone']>, string> = {
  accent: 'var(--ai-accent)',
  green: 'var(--ai-green)',
  orange: 'var(--ai-orange)',
  red: 'var(--ai-red)',
  neutral: 'var(--ai-ink-3)',
};

/** liveline 静态快照替代：SVG polyline + 面积渐变 + 末点（无实时动效） */
function MiniLineChart({ values, color, label }: { values: number[]; color: string; label?: string }) {
  const gradientId = useId();
  const w = 260;
  const h = 64;
  const pad = 6;
  if (values.length === 0) return null;
  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min || 1;
  const x = (i: number) => pad + (i / Math.max(values.length - 1, 1)) * (w - pad * 2);
  const y = (v: number) => h - pad - ((v - min) / span) * (h - pad * 2);
  const d = values.map((v, i) => `${i === 0 ? 'M' : 'L'}${x(i).toFixed(1)},${y(v).toFixed(1)}`).join(' ');
  const area = `${d} L${x(values.length - 1).toFixed(1)},${h - pad} L${x(0).toFixed(1)},${h - pad} Z`;
  return (
    <svg
      viewBox={`0 0 ${w} ${h}`}
      className="h-16 w-full"
      role="img"
      aria-label={label ?? '趋势快照'}
      data-testid="ai-insight-chart"
    >
      <defs>
        <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.25" />
          <stop offset="100%" stopColor={color} stopOpacity="0" />
        </linearGradient>
      </defs>
      <path d={area} fill={`url(#${gradientId})`} />
      <path
        d={d}
        fill="none"
        stroke={color}
        strokeWidth="2"
        strokeLinejoin="round"
        strokeLinecap="round"
      />
      <circle cx={x(values.length - 1)} cy={y(values[values.length - 1])} r="3" fill={color} />
    </svg>
  );
}

const COLUMN_CLASS: Record<NonNullable<AiInsightCardsProps['columns']>, string> = {
  2: 'md:grid-cols-2',
  3: 'md:grid-cols-3',
  4: 'md:grid-cols-2 lg:grid-cols-4',
};

export function AiInsightCards({ items, columns = 4, className }: AiInsightCardsProps) {
  return (
    <div
      className={cn('grid grid-cols-1 gap-3', COLUMN_CLASS[columns], className)}
      data-testid="ai-insight-cards"
    >
      {items.map((item, i) => {
        const tone = TONE_COLOR[item.tone ?? 'neutral'];
        return (
          <div
            key={item.key}
            className="animate-ai-fade-up rounded-[12px] border border-ai-line bg-ai-surface p-4"
            style={{ animationDelay: `${i * 80}ms` }}
          >
            <div className="flex items-center justify-between gap-2">
              <span className="text-[11px] font-medium uppercase tracking-wider text-ai-ink-3">
                {item.label}
              </span>
              {item.icon && (
                <span className="shrink-0 opacity-40" style={{ color: tone }} aria-hidden>
                  {item.icon}
                </span>
              )}
            </div>
            <div className="mt-1 text-[22px] font-semibold tracking-[-0.01em] text-ai-ink tabular-nums">
              {item.value}
            </div>
            {item.sub && (
              <div className="mt-1 text-[11.5px]" style={{ color: tone }}>
                {item.sub}
              </div>
            )}
            {item.series && item.series.length > 0 && (
              <div className="mt-2 overflow-hidden rounded-[8px] border border-ai-line bg-ai-inset p-1.5">
                <MiniLineChart values={item.series} color={tone} label={item.seriesLabel} />
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

export default AiInsightCards;
```

**(b) `UsageStats.tsx`**：import 区（Task 4 已加的 AiFilterTable import 行后）加：

```tsx
import { AiInsightCards } from '@/components/ui/ai/AiInsightCards';
```

token 趋势序列派生（放在 Task 4 所加 `operationCounts` 之后；`getRecentLlmCalls` 后端 SQL 为 `ORDER BY created_at DESC`——`src-tauri/src/db/repositories_pipeline.rs` L1339，取前 20 条反转得时间正序）：

```tsx
  // getRecentLlmCalls 返回新→旧（repositories_pipeline.rs L1339 DESC），取 20 条反转为时间正序
  const tokenSeries = useMemo(
    () => [...recentCalls.slice(0, 20)].reverse().map(c => c.total_tokens || 0),
    [recentCalls]
  );
```

统计卡 grid（现状 L196-266，四张 Card/CardContent）整体替换为（原卡色调映射：cinema-gold→accent、blue-400→neutral（ai 调色板无蓝）、green-400→green、purple-400→orange（无紫）；仅「总 Token 数」卡带 series，其余纯统计卡；icon 尺寸统一 20）：

```tsx
      <AiInsightCards
        columns={4}
        items={[
          {
            key: 'calls',
            label: '总调用次数',
            value: String(globalStats?.count ?? 0),
            tone: 'accent',
            icon: <Hash size={20} />,
            sub: storyStats != null ? `本故事: ${storyStats.count}` : undefined,
          },
          {
            key: 'tokens',
            label: '总 Token 数',
            value: formatTokens(globalStats?.total_tokens ?? 0),
            tone: 'neutral',
            icon: <Activity size={20} />,
            sub: storyStats != null ? `本故事: ${formatTokens(storyStats.total_tokens)}` : undefined,
            series: tokenSeries,
            seriesLabel: '最近调用 token 趋势',
          },
          {
            key: 'cost',
            label: '预估费用',
            value: formatCost(globalStats?.total_cost ?? 0),
            tone: 'green',
            icon: <Coins size={20} />,
            sub: storyStats != null ? `本故事: ${formatCost(storyStats.total_cost)}` : undefined,
          },
          {
            key: 'success',
            label: '成功率',
            value:
              recentCalls.length > 0
                ? `${Math.round((recentCalls.filter(c => c.success).length / recentCalls.length) * 100)}%`
                : 'N/A',
            tone: 'orange',
            icon: <BarChart3 size={20} />,
            sub: `基于最近 ${recentCalls.length} 次调用`,
          },
        ]}
      />
```

替换后 `Card`/`CardContent` 仍被「最近调用」外壳（L269-270）使用，import 保留；lucide `Hash`/`Activity`/`Coins`/`BarChart3` 经 items 继续使用，保留。

**(c) `AgencyEval.tsx`**：import 区加 `import { AiInsightCards } from '@/components/ui/ai/AiInsightCards';`。

三统计卡 grid（现状 L140-160）：

```tsx
      <div className="grid grid-cols-3 gap-4">
        <div className="rounded border p-4">
          <div className="text-sm text-gray-500">质量门通过率</div>
          <div className="text-2xl font-bold">{(data.pass_rate * 100).toFixed(0)}%</div>
          <div className="text-xs text-gray-400">{data.gate_history.length} 次判定</div>
        </div>
        <div className="rounded border p-4">
          <div className="text-sm text-gray-500">检查点</div>
          <div className="text-2xl font-bold">{data.checkpoints.length}</div>
          <div className="text-xs text-gray-400">里程碑快照</div>
        </div>
        <div className="rounded border p-4">
          <div className="text-sm text-gray-500">Human 信号</div>
          <div className="text-2xl font-bold">
            {data.human_signals.length === 0
              ? '—'
              : `${((data.human_signals.reduce((a, s) => a + s.modification_ratio, 0) / data.human_signals.length) * 100).toFixed(0)}%`}
          </div>
          <div className="text-xs text-gray-400">平均修改率</div>
        </div>
      </div>
```

替换为（文案/数值逻辑零改动，`50%` 等既有测试断言文本由 value 原样提供；无 icon/series）：

```tsx
      <AiInsightCards
        columns={3}
        items={[
          {
            key: 'pass-rate',
            label: '质量门通过率',
            value: `${(data.pass_rate * 100).toFixed(0)}%`,
            tone: 'green',
            sub: `${data.gate_history.length} 次判定`,
          },
          {
            key: 'checkpoints',
            label: '检查点',
            value: String(data.checkpoints.length),
            tone: 'neutral',
            sub: '里程碑快照',
          },
          {
            key: 'human-signals',
            label: 'Human 信号',
            value:
              data.human_signals.length === 0
                ? '—'
                : `${((data.human_signals.reduce((a, s) => a + s.modification_ratio, 0) / data.human_signals.length) * 100).toFixed(0)}%`,
            tone: 'orange',
            sub: '平均修改率',
          },
        ]}
      />
```

主题说明：同 Task 3/5，浅色页切口已声明。

- [ ] **Step 4: Run tests**

Run: `cd src-frontend && npx vitest run src/components/ui/ai/__tests__/AiInsightCards.test.tsx src/pages/__tests__/AgencyEval.test.tsx && npx tsc --noEmit`
Expected: AiInsightCards 6 passed；AgencyEval 1 passed 不回归；tsc 干净

- [ ] **Step 5: Commit**

```bash
git add src-frontend/src/components/ui/ai/AiInsightCards.tsx src-frontend/src/components/ui/ai/__tests__/AiInsightCards.test.tsx src-frontend/src/pages/UsageStats.tsx src-frontend/src/pages/AgencyEval.tsx
git commit -m "feat: AiInsightCards 组件入库并替换 UsageStats/AgencyEval 统计卡（P3 Task6）"
```

---

### Task 7: barrel 导出 + 全量回归门 + 文档同步

**Files:**
- Modify: `src-frontend/src/components/index.ts`（barrel 导出 6 个组件，P2 分组段 L25-51 之后、L52 `export { DataLoader }` 之前）
- Modify: `CHANGELOG.md`（L4 空行后、`## v0.39.0（2026-08-12）`（L5）前插入 P3 Unreleased 段）
- Modify: `PROJECT_STATUS.md`（`## ✅ 最近完成功能`（L16）下、`### v0.39.0 - …`（L18）前插入 P3 条目）
- Modify: `AGENTS.md`（编码风格节 L30 AI 原生组件行更新）
- Modify: `docs/plans/2026-08-12-beautifului-ai-native-design.md`（§8 第 4 条 P3 行尾追加 AiChat 关闭结论）

**Interfaces:**
- Consumes: Task 1-6 全部产出
- Produces: barrel 导出 `AiSearchList` / `AiCodeBlock` / `AiDiffTable` / `AiFilterTable` / `AiRecordsTable` / `AiInsightCards`（+ 各自 Props/Item/Column/Sort 类型用 `export type`；`AiFilterChipsBar` 作为 AiFilterTable 的伴随导出一并登记）

- [ ] **Step 1: barrel 导出**

`src-frontend/src/components/index.ts` 在 L51（AiSelectionActions 类型导出块的 `} from './ui/ai/AiSelectionActions';` 行）后、L52 `export { DataLoader } from './DataLoader';` 前插入：

```ts
// P3 - AI Native Components（数据展示）
export { AiSearchList } from './ui/ai/AiSearchList';
export { AiCodeBlock } from './ui/ai/AiCodeBlock';
export { AiDiffTable } from './ui/ai/AiDiffTable';
export { AiFilterTable, AiFilterChipsBar } from './ui/ai/AiFilterTable';
export { AiRecordsTable } from './ui/ai/AiRecordsTable';
export { AiInsightCards } from './ui/ai/AiInsightCards';
export type { AiSearchListProps } from './ui/ai/AiSearchList';
export type { AiCodeBlockProps } from './ui/ai/AiCodeBlock';
export type { AiDiffTableProps, AiDiffRow } from './ui/ai/AiDiffTable';
export type {
  AiFilterTableProps,
  AiFilterChipsBarProps,
  AiFilterChipItem,
  AiFilterColumn,
} from './ui/ai/AiFilterTable';
export type {
  AiRecordsTableProps,
  AiRecordsColumn,
  AiRecordsSort,
} from './ui/ai/AiRecordsTable';
export type { AiInsightCardsProps, AiInsightCardItem } from './ui/ai/AiInsightCards';
```

- [ ] **Step 2: 全量回归门**

Run: `cd src-frontend && npx tsc --noEmit && npx vitest run 2>&1 | tail -3 && npm run format:check`
Expected: tsc 干净；vitest **预期 564 passed / 3 skipped**（基线 523 + Task1 6 + Task2 6 + Task3 7 + Task4 7 + Task5 9 + Task6 6 = 564，以实际输出为准并记录进 CHANGELOG；只允许比基线多）；format 通过

Run: `python3 scripts/architecture_guard.py`
Expected: 退出码 0（纯前端改动，应为通过）

（Rust 侧无改动：`cargo test --lib` 基线 1328 passed / 2 ignored 不变，本批不重跑。）

- [ ] **Step 3: 文档同步（版本号不动，发版另行进行）**

**(a) `CHANGELOG.md`** — 在 L4 空行后、`## v0.39.0（2026-08-12）`（L5）前插入：

```markdown
## Unreleased（P3 AI 原生组件库 · 数据展示）

### 功能：beautifului AI 原生组件第三批（设计文档 P3 范围）

将 beautifului.dev 的 6 个数据展示组件适配为受控组件入库 `src-frontend/src/components/ui/ai/`，并逐点替换幕后落点。沿用 P1 令牌桥（`--ai-*` 16 变量契约不动，tint 缺口 color-mix 内联零扩令牌），不引新依赖（liveline 以组件内嵌 SVG MiniLineChart 静态快照替代）；纯前端阶段，无后端改动。

- **AiSearchList（Task1）**：受控搜索框 + 结果计数/空态（提取参考搜索框视觉语法，下拉结果列表语义不符不落），替换 PromptsPanel 搜索+计数区。
- **AiCodeBlock（Task2）**：只读代码块（剥离逐行流式演示循环与语法着色，复制按钮带反馈），批量替换六文件七处裸 pre/JSON.stringify（TracingPanel step.details、Logs 系统日志 + 行 details、Mcp 工具结果、Skills 执行结果、IntentionGraphDiagnostics plan/result JSON、PromptsPanel 内置默认值）。
- **AiDiffTable（Task3）**：指标基准/对比/Δ 行式对比表（Δ 按 betterWhen 语义着色，color-mix tint），替换 AgencyEval CheckpointCompare 四格 delta 瓷砖；基准/对比绝对值由宿主解析 checkpoint metrics_json（key 与后端 coordinator.rs 对齐，零后端改动）。
- **AiFilterTable（Task4）**：筛选 chips 条 + 数据表（行过滤宿主侧受控完成，pill 经 column.render 插槽），替换 UsageStats 分组 tabs（新增分组计数徽章）与最近调用表；AiFilterChipsBar 独立导出，可选接入 Logs 级别筛选。
- **AiRecordsTable（Task5）**：全受控记录表格（records-* 全局类 Tailwind 自研；自研 Checkbox 含 mixed；可选 selection/sort/footer 插槽；新增受控展开行 + rowKeyAttribute），替换 PromptsPanel 分组行列表（展开编辑器移入 row detail，既有测试选择器零改动）与 AgencyEval 判定历史/token 用量双表（token 表启用受控排序 + footer 合计行）。
- **AiInsightCards（Task6）**：统计洞察卡片组 + 内嵌 MiniLineChart 静态快照（删 useDarkMode，序列色 hex 映射 ai-orange/ai-accent/ai-red；不落 carousel 分页壳与 blur crossfade 占位——无宿主分页场景），替换 UsageStats 四统计卡（Token 卡带最近调用趋势折线）与 AgencyEval 三统计卡。
- **AiChat 关闭**：设计文档 §8 P3 的「AiChat」经勘察关闭——ChatComposer 是 P1 AiPromptBar 的严格子集且应用无多轮对话场景；差异特性（分节回复 + resolving 退焦模糊）记录备选，可作未来 AiChatThread 组合复用 AiPromptBar，不入 P3。
- **已知切口**：AgencyEval 为浅色裸样式页，接入的 `--ai-*` 深色令牌组件与周边形成对比（同 P2 AgencyStudio 先例），P4 统一处理后台页令牌。

### 测试

- src-frontend `npx vitest run`：**<以 Task7 Step2 实际输出填写> passed / 3 skipped**（基线 523 + 本批新增 41）。
```

**(b) `PROJECT_STATUS.md`** — 在 `## ✅ 最近完成功能`（L16）下、`### v0.39.0 - AI 原生组件库 P1+P2（共 10 组件）+ 保存 UNIQUE 修复（2026-08-12）`（L18）前插入：

```markdown
### Unreleased - beautifului AI 原生组件 P3（数据展示六件套）（2026-08-13）

- **六组件入库** `components/ui/ai/`：AiSearchList（PromptsPanel 搜索计数区）、AiCodeBlock（六文件七处裸 pre/JSON 批量替换）、AiDiffTable（AgencyEval 检查点对比，metrics_json 解析补基准/对比列）、AiFilterTable（UsageStats 分组 tabs + 最近调用表；AiFilterChipsBar 可选接 Logs 级别筛选）、AiRecordsTable（PromptsPanel 分组行列表保留展开编辑器 + AgencyEval 判定历史/token 用量双表）、AiInsightCards（UsageStats/AgencyEval 统计卡，内嵌 MiniLineChart 静态快照替代 liveline）。
- **AiChat 关闭**：ChatComposer 为 AiPromptBar 严格子集、无多轮对话场景，设计文档 P3「AiChat」以勘察结论关闭，差异特性记录备选。
- **切口**：AgencyEval 浅色裸样式页接 `--ai-*` 深色令牌组件（同 P2 AgencyStudio 先例），P4 统一。
- **验证**：`npx tsc --noEmit` / `npx vitest run`（<实际数> passed / 3 skipped）/ `format:check` / `architecture_guard.py` 全绿；Rust 无改动（1328 passed / 2 ignored 不变）。版本号未动，发版另行进行。
```

**(c) `AGENTS.md`** — 将编码风格节 L30 的 AI 原生组件行整体替换为：

```markdown
- **AI 原生组件**: `src-frontend/src/components/ui/ai/`（P1 生成体验：AiLoading/AiThinking/AiStreamingText/AiPromptBar/AiApprovalCard；P2 代理与任务：AiContextCards/AiToolChips/AiRecommendationCard/AiTaskRows/AiSelectionActions；P3 数据展示：AiSearchList/AiCodeBlock/AiDiffTable/AiFilterTable/AiRecordsTable/AiInsightCards），只引用 `--ai-*` 语义令牌（幕后 tokens.css / 幕前 frontstage.css 各自定义），不写死颜色；tint 缺口用 color-mix 内联，零扩 16 变量契约；动画用 tailwind.config.js 注册的 ai keyframes 工具类；受控组件，禁止引入自运行演示逻辑；组件内嵌私有动效/图表（如 AiSelectionActions 的 SelectionStreamText、AiInsightCards 的 MiniLineChart）不复用为公共 API。
```

**(d) 设计文档** — `docs/plans/2026-08-12-beautifului-ai-native-design.md` §8 第 4 条（L81）行尾追加：

```markdown
（AiChat 经 P3 勘察关闭：ChatComposer 为 AiPromptBar 严格子集、无多轮对话场景，差异特性「分节回复 + resolving 退焦」记录备选，详见 P3 实施计划）
```

- [ ] **Step 4: Commit**

```bash
git add src-frontend/src/components/index.ts CHANGELOG.md PROJECT_STATUS.md AGENTS.md docs/plans/2026-08-12-beautifului-ai-native-design.md
git commit -m "docs: P3 AI 原生组件库 barrel 导出与文档同步（P3 Task7）"
```

---

## 全量回归清单（每个 Task 末尾必过；Task 7 Step2 为总闸）

| 检查项 | 命令 | 通过标准 |
| --- | --- | --- |
| 类型检查 | `cd src-frontend && npx tsc --noEmit` | 0 error |
| 组件单测（本 Task） | `npx vitest run <本 Task 测试路径>` | 全绿 |
| 全量前端测试 | `npx vitest run` | ≥523 基线，只允许增加（最终预期 564） |
| 受影响既有测试 | Task1/2/5 加跑 `src/pages/settings/__tests__/PromptsPanel.test.tsx`；Task3/5/6 加跑 `src/pages/__tests__/AgencyEval.test.tsx` | 5 passed / 1 passed 不回归 |
| 格式化 | `npm run format:check` | 通过 |
| 架构守卫 | `python3 scripts/architecture_guard.py`（仓库根） | 退出码 0 |
| Rust 测试 | 不重跑（纯前端阶段） | 基线 1328 passed / 2 ignored 不变 |
| Commit 规范 | 中文 conventional commit，不 --no-verify，不推送不打 tag | 每个 Task 独立 commit + 评审 |

## 文件清单汇总表

| 文件 | Task | 动作 |
| --- | --- | --- |
| `src-frontend/src/components/ui/ai/AiSearchList.tsx` | 1 | 新建 |
| `src-frontend/src/components/ui/ai/__tests__/AiSearchList.test.tsx` | 1 | 新建 |
| `src-frontend/src/pages/settings/PromptsPanel.tsx` | 1 / 2 / 5 | 修改（L613-650 搜索计数区 + L414-416/L779-785 清理 / L717-726 默认值块 / L664-773 分组行列表——三 Task 区域嵌套关系见依赖图） |
| `src-frontend/src/components/ui/ai/AiCodeBlock.tsx` | 2 | 新建 |
| `src-frontend/src/components/ui/ai/__tests__/AiCodeBlock.test.tsx` | 2 | 新建 |
| `src-frontend/src/pages/TracingPanel.tsx` | 2 | 修改（L140-144 step.details pre） |
| `src-frontend/src/pages/Logs.tsx` | 2 / 4（可选） | 修改（L246-248 系统日志 + L312-318 details pre / L165-182 级别筛选——区域不重叠） |
| `src-frontend/src/pages/Mcp.tsx` | 2 | 修改（L275-281 toolResult pre） |
| `src-frontend/src/pages/Skills.tsx` | 2 | 修改（L271-273 executionResult pre） |
| `src-frontend/src/pages/IntentionGraphDiagnostics.tsx` | 2 | 修改（L309-322 plan_json + L324-337 result_json pre） |
| `src-frontend/src/components/ui/ai/AiDiffTable.tsx` | 3 | 新建 |
| `src-frontend/src/components/ui/ai/__tests__/AiDiffTable.test.tsx` | 3 | 新建 |
| `src-frontend/src/pages/AgencyEval.tsx` | 3 / 5 / 6 | 修改（L86-117 diff 展示 / L167-195 + L197-223 双表 / L140-160 统计卡——三 Task 区域互不重叠） |
| `src-frontend/src/components/ui/ai/AiFilterTable.tsx` | 4 | 新建 |
| `src-frontend/src/components/ui/ai/__tests__/AiFilterTable.test.tsx` | 4 | 新建 |
| `src-frontend/src/pages/UsageStats.tsx` | 4 / 6 | 修改（L173-193 分组 tabs + L282-331 最近调用表 / L196-266 统计卡——区域相邻不重叠） |
| `src-frontend/src/components/ui/ai/AiRecordsTable.tsx` | 5 | 新建 |
| `src-frontend/src/components/ui/ai/__tests__/AiRecordsTable.test.tsx` | 5 | 新建 |
| `src-frontend/src/components/ui/ai/AiInsightCards.tsx` | 6 | 新建 |
| `src-frontend/src/components/ui/ai/__tests__/AiInsightCards.test.tsx` | 6 | 新建 |
| `src-frontend/src/components/index.ts` | 7 | 修改（L51 后 barrel） |
| `CHANGELOG.md` / `PROJECT_STATUS.md` / `AGENTS.md` / `docs/plans/2026-08-12-beautifului-ai-native-design.md` | 7 | 修改（文档同步） |

## 依赖顺序图

```
P1/P2 令牌桥与组件（--ai-* / ai-* 工具类 / keyframes，已入库，本批不动）
  │
  ├─ Task1 AiSearchList ────→ PromptsPanel 搜索计数区（L613-650）
  ├─ Task2 AiCodeBlock ─────→ TracingPanel / Logs×2 / Mcp / Skills / IGD×2 / PromptsPanel L717-726
  ├─ Task3 AiDiffTable ─────→ AgencyEval CheckpointCompare（L86-117；metrics_json 解析对齐 coordinator.rs L516-519）
  ├─ Task4 AiFilterTable ───→ UsageStats tabs（L173-193）+ 最近调用表（L282-331）
  │     └─ 可选：Logs 级别筛选（L165-182，用 AiFilterChipsBar）
  ├─ Task5 AiRecordsTable ──→ PromptsPanel 分组行列表（L664-773，renderDetail 原样携带 Task2 的 AiCodeBlock 块）
  │                          + AgencyEval 判定历史（L167-195）+ token 用量（L197-223，sortable + footer）
  ├─ Task6 AiInsightCards ──→ UsageStats 统计卡（L196-266）+ AgencyEval 统计卡（L140-160）
  │
  └─ Task7 barrel + 全量回归门 + 文档同步（依赖 Task1-6 全部）

文件冲突规避：
- PromptsPanel：Task1（L613-650）→ Task2（L717-726）→ Task5（L664-773，含 Task2 块）；Task5 在 Task2 之后，
  renderDetail 携带 AiCodeBlock 用法不回退。
- UsageStats：Task4 动外两区（tabs/表），Task6 动中区（统计卡）；行号以前置 Task 合并后漂移，以锚点代码定位。
- AgencyEval：Task3（L86-117）→ Task6（L140-160）与 Task5（L167-223）；区域互不重叠，顺序按 Task 编号。
- Logs：Task2（L246-248/L312-318）与 Task4 可选（L165-182）不重叠。
```

## Self-Review 结论

- **Spec coverage（对照任务书 + 勘察结论 p3-recon-summary.md）**：
  - 六组件全部覆盖，集成点与任务书一致；ChatComposer 跳过结论写入 CHANGELOG/PROJECT_STATUS/AGENTS/设计文档 §8（Task 7），设计文档 P3「AiChat」以此关闭。
  - 勘察结论逐条落实：tint 零扩令牌（color-mix 内联，aiTokens.test.ts 不动）；SearchList fade-in 裸 keyframes → animate-fade-in；FilterTable 状态 pill → column.render 插槽、chips 圆点 hex 收进 props；RecordsTable 全受控 + Checkbox 自研含 mixed + footer 可选插槽；InsightCards 自研 MiniLineChart 静态快照替代 liveline、删 useDarkMode、序列色 hex 映射 ai-orange/ai-accent/ai-red、不补自动播放、blur crossfade 占位随分页壳剥离；全部受控化、剥离演示状态机/计时器。
  - Task 顺序简单先行（SearchList → CodeBlock → DiffTable → FilterTable → RecordsTable → InsightCards 殿后），收尾 Task 7 统一 barrel + 回归门 + 文档同步；四文件多 Task 触及的区域排布已写入 Global Constraints 与依赖图。
- **行号核实（全部经 Read/Grep 实地核实，非照抄任务书）**：PromptsPanel.tsx L1-22 imports / L169 组件定义 / L175 searchQuery / L240-279 filteredEntries+grouped / L414-416 handleClearSearch / L613-650 搜索计数区 / L653-777 分组列表 / L717-726 默认值块 / L779-785 空态；UsageStats.tsx L122-137 filteredCalls/filteredStats / L173-193 tabs / L196-266 统计卡 / L268-333 最近调用表；AgencyEval.tsx L39-120 CheckpointCompare / L140-160 统计卡 / L167-195 判定历史 / L197-223 token 用量；Logs.tsx L26-35 类型与常量 / L53-59 深链搜索 / L133-207 筛选区 / L246-248 系统日志 pre / L312-318 details pre；TracingPanel.tsx L140-144；Mcp.tsx L275-281；Skills.tsx L271-273；IntentionGraphDiagnostics.tsx L309-337；agency.ts L20-28/L53-58 类型；coordinator.rs L503-521 metrics_json key；repositories_pipeline.rs L1339 DESC 排序；tailwind.config.js L46-62 ai 色组 / L71 shadow-float / L88-104 animation / L105-157 keyframes；tokens.css L39-54 十六变量；index.ts L25-51 P2 段 / L52 DataLoader；CHANGELOG.md L5 v0.39.0；PROJECT_STATUS.md L16/L18；AGENTS.md L30；设计文档 L81 §8 P3 行；既有测试 `pages/settings/__tests__/PromptsPanel.test.tsx`（5 例，L90 `[data-prompt-id] button` 选择器 + L95 prompt-editor）与 `pages/__tests__/AgencyEval.test.tsx`（1 例，L54-59 文本断言）。
- **新决策（相对勘察结论）**：
  1. **AiInsightCards 形态落定**：导出受控卡片组 + 内嵌 MiniLineChart，**不落 AiInsightCarousel 分页壳**（PAGES/autoplay/分页器/blur crossfade 占位/pill CTA 均为演示逻辑且无宿主分页场景），勘察遗留的「壳或卡片组（计划定）」就此关闭。
  2. **CheckpointCompare 基准/对比列数据源**：diff 仅含 delta（agency.ts L53-58），绝对值由宿主解析 metrics_json（key 已逐一对齐 coordinator.rs L516-519：words_total/chapters_done/tokens_used/gate_scores 末条 weighted），解析失败回退 `—`，零后端改动。
  3. **AiRecordsTable 新增受控展开行 + rowKeyAttribute**：参考无展开能力，PromptsPanel 需要；展开行仅 open 时挂载（防重型编辑器常驻 DOM），`data-prompt-id` 经 rowKeyAttribute 保留使既有测试零改动。selection 在 P3 无宿主，作为可选能力 + 单测覆盖保留；sort 由 AgencyEval token 用量表真实启用。
  4. **AiSearchList 语义裁剪**：参考的下拉结果列表与 PromptsPanel 主列表过滤语义不符，提取搜索框视觉语法 + 计数/空态做受控组件（同 P2 AiToolChips 的语义裁剪先例）。
  5. **AiCodeBlock 剥离语法着色**：本批七个集成点均为 JSON/文本 dump，Tok 着色系统无落点，整体剥离；复制反馈计时器属交互式 UI 状态保留（非自运行演示）。
  6. **Logs 可选 Step 落定**：勘察标注「评估工作量后决定」——级别筛选与 AiFilterChipsBar 语义 1:1、改动 ≤20 行，列为 Task4 可选 Step 6（独立 commit）；搜索框因涉深链 `logsSearchQuery` 逻辑（L53-59）保持不动。
  7. **UsageStats 色调映射**：blue-400/purple-400 在 ai 调色板无对应，映射为 neutral/orange 并写入 Task 6 说明，避免执行者临场随意选色。
- **Placeholder scan**：全文无 TBD；唯一待定值为 CHANGELOG/PROJECT_STATUS 中最终测试计数 `<以 Task7 Step2 实际输出填写>`/`<实际数>`——设计上必须由执行者填入的真实命令输出，非占位符。
- **Type consistency**：
  - `AiFilterTableProps<T>`/`AiRecordsTableProps<T>`/`AiFilterColumn<T>` 泛型与宿主行类型（`LlmCall`、`PromptEntry`、`GateHistoryItem`、`PurposeUsage`）逐一对应；`rowKey` 返回类型分别为 `number`（LlmCall.id，AiFilterTable 收 `React.Key`）与 `string`（AiRecordsTable），接口已区分。
  - `AiRecordsSort.dir` 为 `1 | -1`，`onSortChange` 翻转逻辑 `(sort.dir * -1) as 1 | -1` 与参考实现一致；`usageSort.key as keyof PurposeUsage` 排序键与列 key（calls/total_tokens/total_duration_ms/purpose）一致。
  - PromptsPanel `expandedId` 为 `string | null`（L666 `expandedId === entry.id` 用法可证），与 `expandedKey?: string | null` 一致；`onRowToggle` 内 `setExpandedId(expandedId === id ? null : id)` 与原 L672 逻辑 1:1。
  - AiCodeBlock 集成处 `selectedGraph.plan_json!` 非空断言与原 IIFE（L313-319）一致（外层 `selectedGraph.plan_json &&` 已守卫）。
  - 测试断言与组件实现逐项对齐：`ai-search-count`/`ai-search-empty`/`data-line-no`/`ai-diff-delta`/`ai-filter-chips`/`ai-records-detail`/`ai-records-sort-*`/`ai-insight-chart` 均在实现中定义；fake timers 仅在 AiCodeBlock 复制翻转测试使用，卸载清理由组件 useEffect 承担。
